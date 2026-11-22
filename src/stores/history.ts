import { ref } from 'vue';
import { defineStore } from 'pinia';
import {
    addHistory,
    clearHistory,
    hasDb,
    listHistory,
    type HistoryEntry,
} from '@/lib/db';

export const useHistoryStore = defineStore('history', () => {
    const entries = ref<HistoryEntry[]>([]);
    const query = ref('');
    const loading = ref(false);
    const loaded = ref(false);

    const load = async () => {
        loading.value = true;
        try {
            entries.value = await listHistory(query.value.trim() || undefined);
        } catch (error) {
            console.error('[history] 读取命令历史失败', error);
        } finally {
            loading.value = false;
            loaded.value = true;
        }
    };

    const record = async (hostId: string, hostLabel: string, command: string) => {
        const entry = { hostId, hostLabel, command, executedAt: Date.now() };
        try {
            const id = await addHistory(entry);
            // 浏览器里没有 SQLite，保留一份内存记录方便调试。
            if (id === null && hasDb()) return;
            if (!query.value) entries.value.unshift({ id: id ?? Date.now(), ...entry });
        } catch (error) {
            console.error('[history] 写入命令历史失败', error);
        }
    };

    const clear = async () => {
        await clearHistory();
        entries.value = [];
    };

    return { entries, query, loading, loaded, load, record, clear };
});
