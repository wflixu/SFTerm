import { ref } from 'vue';
import { defineStore } from 'pinia';
import type { IKeyEntry } from '@/shared/interface';
import { loadState, saveState } from '@/lib/db';

const STORAGE_KEY = 'keys';

export const useKeychainStore = defineStore('keychain', () => {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    const entries = ref<IKeyEntry[]>(raw ? JSON.parse(raw) : []);

    let persistDb = true;
    const setPersistDb = (value: boolean) => {
        persistDb = value;
    };

    const persist = () => {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(entries.value));
        if (persistDb) {
            saveState('keys', entries.value).catch((error) =>
                console.error('[keychain] 写入 SQLite 失败', error),
            );
        }
    };

    /** 首次启动时把 localStorage 里的密钥元数据导入 SQLite；之后以数据库为准。 */
    const hydrate = async () => {
        const stored = await loadState<IKeyEntry[]>('keys');
        if (stored) {
            entries.value = stored;
            return;
        }
        if (entries.value.length) await saveState('keys', entries.value);
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

    return { entries, add, update, remove, hydrate, persist, setPersistDb };
});
