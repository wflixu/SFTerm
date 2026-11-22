import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => null) }));

import { invoke } from '@tauri-apps/api/core';
import { resolveHostAuth } from '../auth';

const memory = new Map<string, string>();
Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
        getItem: (key: string) => memory.get(key) ?? null,
        setItem: (key: string, value: string) => memory.set(key, String(value)),
        removeItem: (key: string) => memory.delete(key),
        clear: () => memory.clear(),
    },
});

const mockSecret = (values: Record<string, string>) => {
    vi.mocked(invoke).mockImplementation(async (cmd: string, args?: unknown) => {
        if (cmd !== 'secret_get') return null;
        const account = (args as { account: string }).account;
        return values[account] ?? null;
    });
};

describe('resolveHostAuth', () => {
    beforeEach(() => {
        memory.clear();
        setActivePinia(createPinia());
        Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} });
        vi.mocked(invoke).mockReset();
    });

    it('reads the private key of a keychain entry and uses in-memory auth', async () => {
        memory.set(
            'keys',
            JSON.stringify([
                { id: 'k1', label: 'laptop', publicKey: 'ssh-ed25519 AAAA', type: 'OpenSSH' },
            ]),
        );
        mockSecret({ 'key:k1:private': '-----BEGIN OPENSSH PRIVATE KEY-----\nabc' });

        const auth = await resolveHostAuth({ id: 'h1', name: 'web', address: '10.0.0.1', keyId: 'k1' });

        expect(auth).toEqual({
            kind: 'key_memory',
            private_key: '-----BEGIN OPENSSH PRIVATE KEY-----\nabc',
            public_key: 'ssh-ed25519 AAAA',
        });
    });

    it('still supports a private key file path', async () => {
        const auth = await resolveHostAuth({
            id: 'h1',
            name: 'web',
            address: '10.0.0.1',
            keyPath: '/tmp/id_ed25519',
        });

        expect(auth).toEqual({ kind: 'key', path: '/tmp/id_ed25519' });
    });

    it('reads a stored password from the keychain', async () => {
        mockSecret({ 'host:h1:password': 'secret' });

        const auth = await resolveHostAuth({
            id: 'h1',
            name: 'web',
            address: '10.0.0.1',
            hasPassword: true,
        });

        expect(auth).toEqual({ kind: 'password', password: 'secret' });
    });

    it('throws when the referenced keychain entry is gone', async () => {
        await expect(
            resolveHostAuth({ id: 'h1', name: 'web', address: '10.0.0.1', keyId: 'missing' }),
        ).rejects.toThrow('引用的密钥已不在系统钥匙串里');
    });
});
