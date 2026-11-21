import { ref } from 'vue';
import { defineStore } from 'pinia';
import type { IKeyEntry } from '@/shared/interface';

const STORAGE_KEY = 'keys';

export const useKeychainStore = defineStore('keychain', () => {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    const entries = ref<IKeyEntry[]>(raw ? JSON.parse(raw) : []);

    const persist = () => {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(entries.value));
    };

    const add = (entry: IKeyEntry) => {
        entries.value.push(entry);
        persist();
    };

    const update = (entry: IKeyEntry) => {
        const index = entries.value.findIndex((item) => item.id === entry.id);
        if (index !== -1) {
            entries.value[index] = entry;
            persist();
        }
    };

    const remove = (id: string) => {
        entries.value = entries.value.filter((item) => item.id !== id);
        persist();
    };

    return { entries, add, update, remove };
});
