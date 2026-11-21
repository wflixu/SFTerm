import { ref } from 'vue';
import { defineStore } from 'pinia';

export interface SessionTab {
    id: string;
    hostId: string;
    title: string;
}

/**
 * 顶层标签：Home 与 SFTP 是常驻标签（由路由固定），
 * 这里只管理「连接标签」。同一主机最多一个文件会话标签。
 */
export const useTabsStore = defineStore('tabs', () => {
    const sessions = ref<SessionTab[]>([]);

    const openSession = (host: { id: string; name: string }): SessionTab => {
        const existing = sessions.value.find((tab) => tab.hostId === host.id);
        if (existing) return existing;

        const tab: SessionTab = {
            id: host.id,
            hostId: host.id,
            title: host.name || host.id,
        };
        sessions.value.push(tab);
        return tab;
    };

    const closeSession = (id: string) => {
        sessions.value = sessions.value.filter((tab) => tab.id !== id);
    };

    const closeAll = () => {
        sessions.value = [];
    };

    return { sessions, openSession, closeSession, closeAll };
});
