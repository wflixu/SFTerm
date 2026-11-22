import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useSystemStore } from '../system';

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

describe('system store', () => {
    beforeEach(() => {
        memory.clear();
        setActivePinia(createPinia());
    });

    it('repairs hosts that have no id and persists them', () => {
        memory.set('hosts', JSON.stringify([{ name: 'web-1', address: '10.0.0.1' }]));

        const store = useSystemStore();

        expect(store.hosts).toHaveLength(1);
        expect(store.hosts[0].id).toBeTruthy();

        const persisted = JSON.parse(memory.get('hosts') ?? '[]');
        expect(persisted[0].id).toBe(store.hosts[0].id);
    });

    it('deduplicates repeated ids', () => {
        memory.set(
            'hosts',
            JSON.stringify([
                { id: 'same', name: 'a', address: '1.1.1.1' },
                { id: 'same', name: 'b', address: '2.2.2.2' },
            ]),
        );

        const store = useSystemStore();
        const ids = new Set(store.hosts.map((host) => host.id));

        expect(ids.size).toBe(2);
    });
});
