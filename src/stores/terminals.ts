import { ref } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

/** 终端输出缓冲上限，避免长时间运行的会话占满内存。 */
const MAX_BUFFER_BYTES = 512 * 1024;

export type TermStatus = 'idle' | 'connecting' | 'ready' | 'error' | 'closed';

export interface TermSession {
    id: string;
    status: TermStatus;
    error?: string;
    chunks: Uint8Array[];
    bytes: number;
}

const decode = (data: string) => {
    const binary = atob(data);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return bytes;
};

/**
 * 终端会话放在 store 里而不是组件里：
 * 组件卸载（切标签）只断开显示，Rust 的 shell 线程继续运行，
 * 输出缓冲在这里，切回来直接重放，不再重新连接。
 */
export const useTerminalsStore = defineStore('terminals', () => {
    const sessions = ref<Record<string, TermSession>>({});

    const ensure = (id: string): TermSession => {
        if (!sessions.value[id]) {
            sessions.value[id] = { id, status: 'idle', chunks: [], bytes: 0 };
        }
        return sessions.value[id];
    };

    const receive = (id: string, data: string) => {
        const session = ensure(id);
        const bytes = decode(data);
        session.chunks.push(bytes);
        session.bytes += bytes.length;
        while (session.bytes > MAX_BUFFER_BYTES && session.chunks.length > 1) {
            const removed = session.chunks.shift();
            if (removed) session.bytes -= removed.length;
        }
    };

    const open = async (
        id: string,
        params: unknown,
        cols: number,
        rows: number,
    ): Promise<boolean> => {
        const session = ensure(id);
        session.status = 'connecting';
        session.error = undefined;
        try {
            await invoke('shell_open', { params, shellId: id, cols, rows });
            session.status = 'ready';
            return true;
        } catch (e) {
            session.status = 'error';
            session.error = String(e);
            return false;
        }
    };

    const close = async (id: string) => {
        await invoke('shell_close', { shellId: id }).catch(() => {});
        delete sessions.value[id];
    };

    const clear = (id: string) => {
        const session = sessions.value[id];
        if (!session) return;
        session.chunks = [];
        session.bytes = 0;
    };

    return { sessions, ensure, receive, open, close, clear };
});
