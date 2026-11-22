import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(async () => undefined),
}));

import { invoke } from '@tauri-apps/api/core';
import { migrateSecrets } from '../migrateSecrets';

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

const asTauri = () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} });
};

describe('migrateSecrets', () => {
    beforeEach(() => {
        memory.clear();
        setActivePinia(createPinia());
        asTauri();
        vi.mocked(invoke).mockClear();
    });

    it('moves host passwords and private keys into the keychain and strips them', async () => {
        memory.set(
            'hosts',
            JSON.stringify([
                { id: 'host-1', name: 'web-1', address: '10.0.0.1', password: 'secret' },
            ]),
        );
        memory.set(
            'keys',
            JSON.stringify([
                {
                    id: 'key-1',
                    label: 'laptop',
                    privateKey: '-----BEGIN OPENSSH PRIVATE KEY-----\nabc',
                    publicKey: 'ssh-ed25519 AAAA',
                },
            ]),
        );

        await migrateSecrets();

        const accounts = vi
            .mocked(invoke)
            .mock.calls.filter(([cmd]) => cmd === 'secret_set')
            .map(([, args]) => (args as { account: string }).account);
        expect(accounts).toContain('host:host-1:password');
        expect(accounts).toContain('key:key-1:private');

        const hosts = JSON.parse(memory.get('hosts') ?? '[]');
        expect(hosts[0].password).toBeUndefined();
        expect(hosts[0].hasPassword).toBe(true);

        const keys = JSON.parse(memory.get('keys') ?? '[]');
        expect(keys[0].privateKey).toBeUndefined();
        expect(keys[0].type).toBe('OpenSSH');
        expect(memory.get('secrets:v1')).toBe('1');
    });

    it('keeps the marker unset when the keychain rejects a write', async () => {
        memory.set(
            'hosts',
            JSON.stringify([
                { id: 'host-1', name: 'web-1', address: '10.0.0.1', password: 'secret' },
            ]),
        );
        vi.mocked(invoke).mockRejectedValueOnce(new Error('denied'));

        await migrateSecrets();

        expect(memory.get('secrets:v1')).toBeUndefined();
        const hosts = JSON.parse(memory.get('hosts') ?? '[]');
        expect(hosts[0].password).toBe('secret');
    });
});
