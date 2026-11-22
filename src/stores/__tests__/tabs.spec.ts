import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useTabsStore } from '../tabs';

describe('tabs store', () => {
    beforeEach(() => {
        setActivePinia(createPinia());
    });

    it('deduplicates terminal tabs by ip and port', () => {
        const tabs = useTabsStore();

        const first = tabs.openTerminal({ id: 'h1', name: 'web-1', address: '1.2.3.4' });
        const second = tabs.openTerminal({ id: 'h2', name: 'web-1-copy', address: '1.2.3.4' });

        expect(tabs.sessions).toHaveLength(1);
        expect(second.id).toBe(first.id);
        expect(second.hostId).toBe('h1');
    });

    it('keeps separate tabs for different ports', () => {
        const tabs = useTabsStore();

        tabs.openTerminal({ id: 'h1', name: 'a', address: '1.2.3.4', port: 22 });
        tabs.openTerminal({ id: 'h2', name: 'b', address: '1.2.3.4', port: 2222 });

        expect(tabs.sessions).toHaveLength(2);
    });
});
