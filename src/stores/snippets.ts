import { ref } from 'vue';
import { defineStore } from 'pinia';
import type { ISnippet } from '@/shared/interface';
import { loadState, saveState } from '@/lib/db';

const STORAGE_KEY = 'snippets';

export const useSnippetsStore = defineStore('snippets', () => {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    const entries = ref<ISnippet[]>(raw ? JSON.parse(raw) : []);

    const persist = () => {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(entries.value));
        saveState('snippets', entries.value).catch((error) =>
            console.error('[snippets] 写入 SQLite 失败', error),
        );
    };

    const hydrate = async () => {
        const stored = await loadState<ISnippet[]>('snippets');
        if (stored) {
            entries.value = stored;
            return;
        }
        if (entries.value.length) await saveState('snippets', entries.value);
    };

    const add = (snippet: ISnippet) => {
        entries.value.push({ ...snippet, updatedAt: Date.now() });
        persist();
    };

    const update = (snippet: ISnippet) => {
        const index = entries.value.findIndex((item) => item.id === snippet.id);
        if (index === -1) return;
        entries.value[index] = { ...snippet, updatedAt: Date.now() };
        persist();
    };

    const remove = (id: string) => {
        entries.value = entries.value.filter((item) => item.id !== id);
        persist();
    };

    return { entries, add, update, remove, hydrate };
});
