import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn(async () => null) }));
vi.mock('@tauri-apps/api/core', () => ({ invoke }));

import { useTerminalsStore } from '../terminals';

const encode = (text: string) => btoa(text);

describe('terminals store', () => {
    beforeEach(() => {
        setActivePinia(createPinia());
        invoke.mockClear();
    });

    it('buffers output per session', () => {
        const terminals = useTerminalsStore();

        terminals.receive('term-1.2.3.4:22', encode('hello '));
        terminals.receive('term-1.2.3.4:22', encode('world'));

        const session = terminals.sessions['term-1.2.3.4:22'];
        expect(session.chunks).toHaveLength(2);
        expect(session.bytes).toBe(11);
    });

    it('closes the shell and drops the session', async () => {
        const terminals = useTerminalsStore();
        terminals.receive('term-x', encode('data'));

        await terminals.close('term-x');

        expect(invoke).toHaveBeenCalledWith('shell_close', { shellId: 'term-x' });
        expect(terminals.sessions['term-x']).toBeUndefined();
    });
});
