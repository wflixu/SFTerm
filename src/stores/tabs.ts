import { ref } from 'vue';
import { defineStore } from 'pinia';

export type TabKind = 'files' | 'terminal';

export interface SessionTab {
    id: string;
    hostId: string;
    title: string;
    kind: TabKind;
}

/**
 * 顶层标签：Home 与 SFTP 是常驻标签（由路由固定），
 * 这里只管理 SSH 连接标签（一个主机一个）。
 */
export const useTabsStore = defineStore('tabs', () => {
    const sessions = ref<SessionTab[]>([]);

    /** 终端标签按 IP:端口 去重：同一台机器不重复开标签。 */
    const terminalId = (host: { address: string; port?: number }) =>
        `term-${host.address}:${host.port ?? 22}`;

    const openTerminal = (host: { id: string; name: string; address: string; port?: number }): SessionTab => {
        const id = terminalId(host);
        const existing = sessions.value.find((tab) => tab.id === id);
        if (existing) return existing;

        const tab: SessionTab = {
            id,
            hostId: host.id,
            title: host.name || host.address,
            kind: 'terminal',
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

    return { sessions, openTerminal, closeSession, closeAll };
});
