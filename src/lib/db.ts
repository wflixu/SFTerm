import { invoke } from '@tauri-apps/api/core';

export const hasDb = () =>
    typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/** 读取 JSON 状态（hosts / keys 等）；数据库里没有时返回 null。 */
export async function loadState<T>(key: string): Promise<T | null> {
    if (!hasDb()) return null;
    const raw = await invoke<string | null>('db_get_state', { key });
    if (!raw) return null;
    try {
        return JSON.parse(raw) as T;
    } catch {
        console.error('[db] 状态解析失败', key);
        return null;
    }
}

export async function saveState(key: string, value: unknown): Promise<void> {
    if (!hasDb()) return;
    await invoke('db_set_state', { key, value: JSON.stringify(value) });
}

export interface HistoryEntry {
    id: number;
    hostId: string;
    hostLabel: string;
    command: string;
    executedAt: number;
}

export async function addHistory(
    entry: Omit<HistoryEntry, 'id'>,
): Promise<number | null> {
    if (!hasDb()) return null;
    return invoke<number>('history_add', {
        hostId: entry.hostId,
        hostLabel: entry.hostLabel,
        command: entry.command,
        executedAt: entry.executedAt,
    });
}

export async function listHistory(query?: string, limit = 200): Promise<HistoryEntry[]> {
    if (!hasDb()) return [];
    return invoke<HistoryEntry[]>('history_list', { query: query ?? null, limit });
}

export async function clearHistory(): Promise<void> {
    if (!hasDb()) return;
    await invoke('history_clear');
}
