<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import FilePane from './FilePane.vue';
import type { FileEntry, IHost } from '@/shared/interface';

interface ConnectInfo {
    session_id: string;
    banner: string | null;
    host_key_type: string;
    fingerprint_sha256: string;
    kex: string | null;
    cipher: string | null;
}

const props = defineProps<{
    host: IHost | null;
    autoConnect?: boolean;
    canPickHost?: boolean;
}>();

const emit = defineEmits<{
    'request-host': [];
}>();

const localPath = ref('');
const localEntries = ref<FileEntry[]>([]);
const localLoading = ref(false);
const localError = ref('');

const info = ref<ConnectInfo | null>(null);
const remotePath = ref('/home');
const remoteEntries = ref<FileEntry[]>([]);
const remoteLoading = ref(false);
const remoteError = ref('');
const connecting = ref(false);

const split = ref(50);
const dragging = ref(false);

const loadLocal = async (path: string) => {
    localLoading.value = true;
    localError.value = '';
    try {
        localEntries.value = await invoke<FileEntry[]>('fs_list_local', { path });
        localPath.value = path;
    } catch (e) {
        localError.value = String(e);
    } finally {
        localLoading.value = false;
    }
};

const loadRemote = async (path: string) => {
    if (!info.value) return;
    remoteLoading.value = true;
    remoteError.value = '';
    try {
        remoteEntries.value = await invoke<FileEntry[]>('sftp_list_dir', {
            sessionId: info.value.session_id,
            path,
        });
        remotePath.value = path;
    } catch (e) {
        remoteError.value = String(e);
    } finally {
        remoteLoading.value = false;
    }
};

const connect = async () => {
    const host = props.host;
    if (!host) return;
    if (!host.address) {
        remoteError.value = '主机信息不完整：缺少地址，回 Home 编辑一下。';
        return;
    }
    if (!host.keyPath && !host.password) {
        remoteError.value = '这台主机还没有配置认证方式，回 Home 点铅笔补一下。';
        return;
    }

    connecting.value = true;
    remoteError.value = '';
    console.debug('[dual] connecting', host.address, host.username);
    try {
        const auth = host.keyPath
            ? { kind: 'key', path: host.keyPath }
            : { kind: 'password', password: host.password ?? '' };

        info.value = await invoke<ConnectInfo>('ssh_connect', {
            params: {
                host: host.address,
                port: host.port ?? 22,
                username: host.username || 'root',
                auth,
            },
        });
        let home = '/home';
        try {
            home = await invoke<string>('sftp_home', { sessionId: info.value.session_id });
        } catch {
            // 拿不到主目录就退回 /home
        }
        await loadRemote(home || '/home');
    } catch (e) {
        remoteError.value = String(e);
        console.debug('[dual] connect failed', e);
    } finally {
        connecting.value = false;
    }
};

const disconnect = async () => {
    if (!info.value) return;
    const id = info.value.session_id;
    info.value = null;
    remoteEntries.value = [];
    await invoke('ssh_disconnect', { sessionId: id });
};

const openLocal = (entry: FileEntry) => {
    if (entry.is_dir) loadLocal(entry.path);
};

const openRemote = (entry: FileEntry) => {
    if (entry.is_dir) loadRemote(entry.path);
};

const startDrag = () => {
    dragging.value = true;
};

const onMove = (event: MouseEvent) => {
    if (!dragging.value) return;
    const el = document.querySelector('.panes') as HTMLElement | null;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const pct = ((event.clientX - rect.left) / rect.width) * 100;
    split.value = Math.min(72, Math.max(28, pct));
};

const endDrag = () => {
    dragging.value = false;
};

onMounted(async () => {
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', endDrag);

    const home = await invoke<string>('fs_home');
    await loadLocal(home);

    if (props.host && props.autoConnect) await connect();
});

watch(
    () => props.host?.id,
    async (next, previous) => {
        if (next === previous) return;
        if (info.value) await disconnect();
        if (next && props.autoConnect) await connect();
    },
);

onBeforeUnmount(() => {
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', endDrag);
    if (info.value) invoke('ssh_disconnect', { sessionId: info.value.session_id });
});
</script>

<template>
    <div class="dual">
        <header v-if="host" class="head">
            <div class="identity">
                <i class="pi pi-server" />
                <div>
                    <div class="name">{{ host.name || host.address }}</div>
                    <div class="addr">
                        {{ host.username || 'root' }}@{{ host.address }}:{{ host.port ?? 22 }}
                    </div>
                </div>
                <span v-if="info" class="pill connected">已连接</span>
                <span v-else-if="connecting" class="pill">连接中…</span>
                <span v-else class="pill">未连接</span>
            </div>

            <div class="ops">
                <Button
                    v-if="!info"
                    label="连接"
                    size="small"
                    :loading="connecting"
                    @click="connect"
                />
                <Button v-else label="断开" size="small" severity="secondary" text @click="disconnect" />
                <Button
                    v-if="canPickHost"
                    label="切换主机"
                    size="small"
                    text
                    @click="emit('request-host')"
                />
            </div>
        </header>

        <p v-if="remoteError" class="error">{{ remoteError }}</p>

        <div class="panes">
            <FilePane
                class="pane-left"
                :style="{ width: `${split}%` }"
                label="Local"
                icon="pi-desktop"
                :path="localPath"
                :entries="localEntries"
                :loading="localLoading"
                @navigate="loadLocal"
                @refresh="loadLocal(localPath)"
                @open="openLocal"
            />
            <div class="splitter" @mousedown.prevent="startDrag" />

            <FilePane
                v-if="host"
                class="pane-right"
                :style="{ width: `${100 - split}%` }"
                :label="host.name || host.address"
                icon="pi-server"
                :path="remotePath"
                :entries="remoteEntries"
                :loading="remoteLoading"
                :empty-text="info ? '空目录' : '尚未连接'"
                @navigate="loadRemote"
                @refresh="loadRemote(remotePath)"
                @open="openRemote"
            />

            <section v-else class="empty-remote" :style="{ width: `${100 - split}%` }">
                <div class="empty-icon"><i class="pi pi-folder" /></div>
                <h3>连接主机</h3>
                <p>从一台已保存的主机开始，用 SFTP 管理文件</p>
                <Button label="选择主机" severity="secondary" @click="emit('request-host')" />
            </section>
        </div>

        <footer class="statusbar">
            <span v-if="info">{{ info.banner }}</span>
            <span v-else-if="host">未连接</span>
            <span v-else>未选择主机</span>
            <span v-if="localError" class="warn">{{ localError }}</span>
            <span class="spacer" />
            <span>传输队列待接入</span>
        </footer>
    </div>
</template>

<style scoped>
.dual {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #fff;
}

.head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
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
    width: 30px;
    height: 30px;
    border-radius: 8px;
    background: #f1f4f8;
    color: #4b5563;
    font-size: 13px;
}

.name {
    font-size: 13px;
    font-weight: 600;
    color: #111827;
}

.addr {
    font-size: 11px;
    color: #6b7280;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.pill {
    margin-left: 4px;
    padding: 2px 8px;
    border-radius: 999px;
    background: #f1f4f8;
    color: #6b7280;
    font-size: 11px;
}

.pill.connected {
    background: #e7f6ec;
    color: #1a7f37;
}

.ops {
    display: flex;
    align-items: center;
    gap: 4px;
}

.error {
    margin: 10px 16px 0;
    padding: 8px 10px;
    border-radius: 8px;
    background: #fdf2f0;
    color: #b42318;
    font-size: 12px;
    white-space: pre-wrap;
}

.panes {
    flex: 1;
    display: flex;
    min-height: 0;
    padding: 10px;
}

.pane-left,
.pane-right {
    border: 1px solid #e6e9ed;
    border-radius: 8px;
    overflow: hidden;
    flex: none;
}

.empty-remote {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 1px solid #e6e9ed;
    border-radius: 8px;
    background: #fbfcfd;
    flex: none;
}

.empty-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 64px;
    height: 64px;
    margin-bottom: 2px;
    border-radius: 16px;
    background: #eef1f5;
    color: #8b93a1;
    font-size: 26px;
}

.empty-remote h3 {
    margin: 0;
    font-size: 16px;
    color: #111827;
}

.empty-remote p {
    margin: 0 0 8px;
    max-width: 260px;
    text-align: center;
    font-size: 13px;
    color: #6b7280;
}

.splitter {
    width: 10px;
    cursor: col-resize;
    flex: none;
    position: relative;
}

.splitter::after {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: 4px;
    width: 2px;
    border-radius: 1px;
    background: #e6e9ed;
}

.splitter:hover::after {
    background: #cfd6de;
}

.statusbar {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 30px;
    padding: 0 16px;
    border-top: 1px solid #eef0f3;
    font-size: 11px;
    color: #9ca3af;
}

.statusbar .spacer {
    flex: 1;
}

.warn {
    color: #b42318;
}
</style>
