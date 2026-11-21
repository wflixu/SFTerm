<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useSystemStore } from '@/stores/system';

interface ConnectInfo {
    session_id: string;
    banner: string | null;
    host_key_type: string;
    fingerprint_sha256: string;
    kex: string | null;
    cipher: string | null;
}

interface RemoteEntry {
    name: string;
    path: string;
    is_dir: boolean;
    is_symlink: boolean;
    size: number;
    mtime: number | null;
}

const route = useRoute();
const router = useRouter();
const system = useSystemStore();

const host = computed(() => system.getHost(route.params.hostId as string));

const info = ref<ConnectInfo | null>(null);
const entries = ref<RemoteEntry[]>([]);
const error = ref('');
const busy = ref(false);
const path = ref('/home');

const connect = async () => {
    if (!host.value) return;
    if (!host.value.keyPath && !host.value.password) {
        error.value = '这台主机还没有配置认证方式，点右上角编辑填一下。';
        return;
    }

    error.value = '';
    busy.value = true;
    try {
        const auth = host.value.keyPath
            ? { kind: 'key', path: host.value.keyPath }
            : { kind: 'password', password: host.value.password ?? '' };

        info.value = await invoke<ConnectInfo>('ssh_connect', {
            params: {
                host: host.value.address,
                port: host.value.port ?? 22,
                username: host.value.username || 'root',
                auth,
            },
        });
        await list();
    } catch (e) {
        error.value = String(e);
    } finally {
        busy.value = false;
    }
};

const list = async () => {
    if (!info.value) return;
    error.value = '';
    busy.value = true;
    try {
        entries.value = await invoke<RemoteEntry[]>('sftp_list_dir', {
            sessionId: info.value.session_id,
            path: path.value.trim() || '/',
        });
    } catch (e) {
        error.value = String(e);
    } finally {
        busy.value = false;
    }
};

const disconnect = async () => {
    if (!info.value) return;
    const id = info.value.session_id;
    info.value = null;
    entries.value = [];
    await invoke('ssh_disconnect', { sessionId: id });
};

const enter = (entry: RemoteEntry) => {
    if (!entry.is_dir) return;
    path.value = entry.path;
    list();
};

const goUp = () => {
    const trimmed = path.value.replace(/\/+$/, '');
    const index = trimmed.lastIndexOf('/');
    path.value = index <= 0 ? '/' : trimmed.slice(0, index);
    list();
};

onBeforeUnmount(() => {
    if (info.value) invoke('ssh_disconnect', { sessionId: info.value.session_id });
});

const humanSize = (entry: RemoteEntry) => {
    if (entry.is_dir) return '---';
    const units = ['B', 'KB', 'MB', 'GB'];
    let value = entry.size;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
        value /= 1024;
        unit += 1;
    }
    return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
};
</script>

<template>
    <div class="session">
        <template v-if="!host">
            <div class="missing">
                <h3>主机不存在</h3>
                <p>它可能已被删除。</p>
                <Button label="返回 Home" @click="router.push('/')" />
            </div>
        </template>

        <template v-else>
            <header class="head">
                <div class="identity">
                    <i class="pi pi-server" />
                    <div>
                        <div class="name">{{ host.name || host.address }}</div>
                        <div class="addr">{{ host.username || 'root' }}@{{ host.address }}:{{ host.port ?? 22 }}</div>
                    </div>
                </div>
                <div class="ops">
                    <Button
                        v-if="!info"
                        label="连接"
                        icon="pi pi-bolt"
                        :loading="busy"
                        @click="connect"
                    />
                    <template v-else>
                        <Button label="断开" severity="secondary" text @click="disconnect" />
                    </template>
                </div>
            </header>

            <p v-if="error" class="error">{{ error }}</p>

            <div v-if="info" class="meta">
                <span>{{ info.banner }}</span>
                <span class="fingerprint">SHA256 {{ info.fingerprint_sha256.slice(0, 24) }}…</span>
                <span>{{ info.kex }}</span>
            </div>

            <div v-if="info" class="browser">
                <div class="pathbar">
                    <button class="icon-btn" title="上级" @click="goUp"><i class="pi pi-arrow-up" /></button>
                    <InputText v-model="path" class="path" @keyup.enter="list" />
                    <button class="icon-btn" title="刷新" @click="list"><i class="pi pi-refresh" /></button>
                </div>

                <div class="list">
                    <div class="row head-row">
                        <span class="col-name">名称</span>
                        <span class="col-size">大小</span>
                    </div>
                    <div
                        v-for="entry in entries"
                        :key="entry.path"
                        class="row"
                        :class="{ dir: entry.is_dir }"
                        @dblclick="enter(entry)"
                    >
                        <span class="col-name">
                            <i class="pi" :class="entry.is_dir ? 'pi-folder' : 'pi-file'" />
                            {{ entry.name }}
                        </span>
                        <span class="col-size">{{ humanSize(entry) }}</span>
                    </div>
                    <div v-if="!entries.length" class="empty">目录为空</div>
                </div>
            </div>

            <div v-else class="idle">
                <div class="idle-icon"><i class="pi pi-bolt" /></div>
                <h3>尚未连接</h3>
                <p>点右上角「连接」，这里会显示远端目录。</p>
            </div>
        </template>
    </div>
</template>

<style scoped>
.session {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #fff;
}

.head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 20px;
    border-bottom: 1px solid #e9ecf0;
}

.identity {
    display: flex;
    align-items: center;
    gap: 10px;
}

.identity > i {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    background: #f1f4f8;
    color: #4b5563;
    font-size: 14px;
}

.name {
    font-size: 14px;
    font-weight: 600;
    color: #111827;
}

.addr {
    font-size: 12px;
    color: #6b7280;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.error {
    margin: 12px 20px 0;
    padding: 10px 12px;
    border-radius: 8px;
    background: #fdf2f0;
    color: #b42318;
    font-size: 12px;
    white-space: pre-wrap;
}

.meta {
    display: flex;
    gap: 16px;
    padding: 8px 20px;
    font-size: 11px;
    color: #9ca3af;
    border-bottom: 1px solid #f0f2f5;
}

.fingerprint {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.browser {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.pathbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border-bottom: 1px solid #f0f2f5;
}

.path {
    flex: 1;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
}

.icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: #6b7280;
    cursor: pointer;
}

.icon-btn:hover {
    background: #f0f2f5;
    color: #111827;
}

.list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
}

.row {
    display: flex;
    align-items: center;
    padding: 6px 20px;
    font-size: 13px;
    color: #374151;
}

.row:hover {
    background: #f7f8fa;
}

.head-row {
    font-size: 11px;
    color: #9ca3af;
    border-bottom: 1px solid #f0f2f5;
}

.col-name {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.col-name i {
    font-size: 12px;
    color: #8b93a1;
}

.dir .col-name {
    color: #111827;
    font-weight: 500;
}

.col-size {
    width: 90px;
    text-align: right;
    color: #9ca3af;
    font-size: 12px;
}

.empty {
    padding: 24px;
    text-align: center;
    font-size: 12px;
    color: #9ca3af;
}

.idle,
.missing {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    color: #6b7280;
}

.idle-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    margin-bottom: 6px;
    border-radius: 14px;
    background: #eceff3;
    color: #9ca3af;
    font-size: 22px;
}

.idle h3,
.missing h3 {
    margin: 0;
    font-size: 15px;
    color: #111827;
}

.idle p,
.missing p {
    margin: 0 0 10px;
    font-size: 13px;
}
</style>
