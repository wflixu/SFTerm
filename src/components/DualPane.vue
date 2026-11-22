<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import FilePane from './FilePane.vue';
import { useTransfersStore, type TransferTask } from '@/stores/transfers';
import type { FileEntry, IHost } from '@/shared/interface';

interface ConnectInfo {
    session_id: string;
    banner: string | null;
    host_key_type: string;
    fingerprint_sha256: string;
    kex: string | null;
    cipher: string | null;
}

type Side = 'local' | 'remote';

const props = defineProps<{
    host: IHost | null;
    autoConnect?: boolean;
    canPickHost?: boolean;
}>();

const emit = defineEmits<{
    'request-host': [];
    'open-terminal': [];
}>();

const transfers = useTransfersStore();

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

const menu = ref<{ x: number; y: number; entry: FileEntry; side: Side } | null>(null);
const conflict = ref<{ entry: FileEntry; target: string; direction: 'upload' | 'download' } | null>(null);
const transferError = ref('');

const nameOpen = ref(false);
const nameMode = ref<'mkdir' | 'rename'>('mkdir');
const nameValue = ref('');
const renameFrom = ref('');
const opError = ref('');

const deleteOpen = ref(false);
const deleteEntry = ref<FileEntry | null>(null);

const joinPath = (dir: string, name: string) => (dir.endsWith('/') ? `${dir}${name}` : `${dir}/${name}`);

const basename = (path: string) => path.split('/').pop() ?? path;

const existsIn = (side: Side, name: string) => {
    const list = side === 'local' ? localEntries.value : remoteEntries.value;
    return list.some((entry) => entry.name === name);
};

const uniqueName = (name: string, side: Side) => {
    const dot = name.lastIndexOf('.');
    const hasExt = dot > 0;
    const stem = hasExt ? name.slice(0, dot) : name;
    const ext = hasExt ? name.slice(dot) : '';
    let index = 2;
    let candidate = `${stem} ${index}${ext}`;
    while (existsIn(side, candidate)) {
        index += 1;
        candidate = `${stem} ${index}${ext}`;
    }
    return candidate;
};

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

const percent = (task: TransferTask) => {
    if (task.total <= 0) return task.status === 'done' ? 100 : 0;
    return Math.min(100, Math.round((task.done / task.total) * 100));
};

const newId = (prefix: string) =>
    `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 6)}`;

const startUpload = async (entry: FileEntry, target: string, overwrite: boolean) => {
    if (!info.value) return;
    const id = newId('u');
    transfers.add({
        id,
        name: entry.name,
        direction: 'upload',
        done: 0,
        total: entry.size,
        status: 'running',
    });
    try {
        await invoke('sftp_upload', {
            id,
            sessionId: info.value.session_id,
            localPath: entry.path,
            remotePath: target,
            overwrite,
        });
        transfers.finish(id);
        await loadRemote(remotePath.value);
    } catch (e) {
        transfers.fail(id, String(e));
        transferError.value = String(e);
    }
};

const startDownload = async (entry: FileEntry, target: string, overwrite: boolean) => {
    if (!info.value) return;
    const id = newId('d');
    transfers.add({
        id,
        name: entry.name,
        direction: 'download',
        done: 0,
        total: entry.size,
        status: 'running',
    });
    try {
        await invoke('sftp_download', {
            id,
            sessionId: info.value.session_id,
            remotePath: entry.path,
            localPath: target,
            overwrite,
        });
        transfers.finish(id);
        await loadLocal(localPath.value);
    } catch (e) {
        transfers.fail(id, String(e));
        transferError.value = String(e);
    }
};

const upload = (entry: FileEntry) => {
    transferError.value = '';
    if (!info.value) {
        transferError.value = '尚未连接';
        return;
    }
    if (entry.is_dir) {
        transferError.value = '文件夹传输暂不支持';
        return;
    }
    const target = joinPath(remotePath.value, entry.name);
    if (existsIn('remote', entry.name)) {
        conflict.value = { entry, target, direction: 'upload' };
        return;
    }
    startUpload(entry, target, false);
};

const download = (entry: FileEntry) => {
    transferError.value = '';
    if (!info.value) {
        transferError.value = '尚未连接';
        return;
    }
    if (entry.is_dir) {
        transferError.value = '文件夹传输暂不支持';
        return;
    }
    const target = joinPath(localPath.value, entry.name);
    if (existsIn('local', entry.name)) {
        conflict.value = { entry, target, direction: 'download' };
        return;
    }
    startDownload(entry, target, false);
};

const closeMenu = () => {
    menu.value = null;
};

const openMenu = (entry: FileEntry, event: MouseEvent, side: Side) => {
    menu.value = {
        x: Math.min(event.clientX, window.innerWidth - 200),
        y: Math.min(event.clientY, window.innerHeight - 140),
        entry,
        side,
    };
};

const menuUpload = () => {
    const current = menu.value;
    menu.value = null;
    if (current) upload(current.entry);
};

const menuDownload = () => {
    const current = menu.value;
    menu.value = null;
    if (current) download(current.entry);
};

const menuRefresh = () => {
    const current = menu.value;
    menu.value = null;
    if (!current) return;
    if (current.side === 'local') loadLocal(localPath.value);
    else loadRemote(remotePath.value);
};

const resolveOverwrite = () => {
    const current = conflict.value;
    if (!current) return;
    conflict.value = null;
    if (current.direction === 'upload') startUpload(current.entry, current.target, true);
    else startDownload(current.entry, current.target, true);
};

const resolveRename = () => {
    const current = conflict.value;
    if (!current) return;
    conflict.value = null;
    const side: Side = current.direction === 'upload' ? 'remote' : 'local';
    const name = uniqueName(basename(current.target), side);
    const dir = current.direction === 'upload' ? remotePath.value : localPath.value;
    const target = joinPath(dir, name);
    if (current.direction === 'upload') startUpload(current.entry, target, false);
    else startDownload(current.entry, target, false);
};

const resolveSkip = () => {
    conflict.value = null;
};

const refreshSide = async (side: Side) => {
    if (side === 'local') await loadLocal(localPath.value);
    else await loadRemote(remotePath.value);
};

const openMkdir = () => {
    if (!info.value) {
        transferError.value = '尚未连接';
        return;
    }
    opError.value = '';
    nameMode.value = 'mkdir';
    nameValue.value = '未命名文件夹';
    renameFrom.value = '';
    nameOpen.value = true;
};

const openRename = (entry: FileEntry) => {
    if (!info.value) {
        transferError.value = '尚未连接';
        return;
    }
    opError.value = '';
    nameMode.value = 'rename';
    nameValue.value = entry.name;
    renameFrom.value = entry.path;
    nameOpen.value = true;
};

const submitName = async () => {
    const name = nameValue.value.trim();
    if (!name) return;
    if (nameMode.value === 'rename' && name === basename(renameFrom.value)) {
        nameOpen.value = false;
        return;
    }

    if (!info.value) {
        opError.value = '尚未连接';
        return;
    }

    const target = joinPath(remotePath.value, name);
    if (existsIn('remote', name)) {
        opError.value = '目标名称已存在';
        return;
    }

    opError.value = '';
    try {
        if (nameMode.value === 'mkdir') {
            await invoke('sftp_mkdir', { sessionId: info.value.session_id, path: target });
        } else {
            await invoke('sftp_rename', {
                sessionId: info.value.session_id,
                from: renameFrom.value,
                to: target,
            });
        }
        nameOpen.value = false;
        await refreshSide('remote');
    } catch (e) {
        opError.value = String(e);
    }
};

const openDelete = (entry: FileEntry) => {
    if (!info.value) {
        transferError.value = '尚未连接';
        return;
    }
    opError.value = '';
    deleteEntry.value = entry;
    deleteOpen.value = true;
};

const confirmDelete = async () => {
    const entry = deleteEntry.value;
    if (!entry || !info.value) return;

    opError.value = '';
    try {
        await invoke('sftp_remove', {
            sessionId: info.value.session_id,
            path: entry.path,
            isDir: entry.is_dir,
        });
        deleteOpen.value = false;
        await refreshSide('remote');
    } catch (e) {
        opError.value = String(e);
    }
};

const menuMkdir = () => {
    menu.value = null;
    openMkdir();
};

const menuRename = () => {
    const current = menu.value;
    menu.value = null;
    if (current) openRename(current.entry);
};

const menuDelete = () => {
    const current = menu.value;
    menu.value = null;
    if (current) openDelete(current.entry);
};

onMounted(async () => {
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', endDrag);
    window.addEventListener('click', closeMenu);

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
    window.removeEventListener('click', closeMenu);
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
                <Button v-if="!info" label="连接" size="small" :loading="connecting" @click="connect" />
                <Button v-else label="断开" size="small" severity="secondary" text @click="disconnect" />
                <Button
                    v-if="host"
                    label="终端"
                    size="small"
                    text
                    icon="pi pi-desktop"
                    @click="emit('open-terminal')"
                />
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
        <p v-if="transferError" class="error">{{ transferError }}</p>

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
                @context-menu="(entry, event) => openMenu(entry, event, 'local')"
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
                :writable="true"
                @context-menu="(entry, event) => openMenu(entry, event, 'remote')"
                @new-folder="openMkdir"
            />

            <section v-else class="empty-remote" :style="{ width: `${100 - split}%` }">
                <div class="empty-icon"><i class="pi pi-folder" /></div>
                <h3>连接主机</h3>
                <p>从一台已保存的主机开始，用 SFTP 管理文件</p>
                <Button label="选择主机" severity="secondary" @click="emit('request-host')" />
            </section>
        </div>

        <div v-if="transfers.tasks.length" class="queue">
            <div class="queue-head">
                <span>传输队列<span v-if="transfers.running.length"> · {{ transfers.running.length }} 进行中</span></span>
                <button class="link" @click="transfers.clearFinished">清除已完成</button>
            </div>
            <div class="queue-list">
                <div v-for="task in transfers.tasks" :key="task.id" class="task">
                    <i class="pi" :class="task.direction === 'upload' ? 'pi-arrow-up' : 'pi-arrow-down'" />
                    <span class="task-name">{{ task.name }}</span>
                    <div class="bar">
                        <div class="fill" :class="task.status" :style="{ width: `${percent(task)}%` }" />
                    </div>
                    <span class="task-pct">
                        {{ task.status === 'running' ? `${percent(task)}%` : task.status === 'done' ? '完成' : '失败' }}
                    </span>
                </div>
            </div>
        </div>

        <footer class="statusbar">
            <span v-if="info">{{ info.banner }}</span>
            <span v-else-if="host">未连接</span>
            <span v-else>未选择主机</span>
            <span v-if="localError" class="warn">{{ localError }}</span>
            <span class="spacer" />
            <span v-if="transfers.running.length">传输中 {{ transfers.running.length }}</span>
            <span v-else>就绪</span>
        </footer>

        <div
            v-if="menu"
            class="ctx"
            :style="{ left: `${menu.x}px`, top: `${menu.y}px` }"
        >
            <button v-if="menu.side === 'local'" @click.stop="menuUpload">上传到远端</button>
            <button v-else @click.stop="menuDownload">下载到本地</button>
            <template v-if="menu.side === 'remote'">
                <div class="ctx-sep" />
                <button @click.stop="menuMkdir">新建文件夹</button>
                <button @click.stop="menuRename">重命名</button>
                <button class="danger" @click.stop="menuDelete">删除</button>
            </template>
            <div class="ctx-sep" />
            <button @click.stop="menuRefresh">刷新</button>
        </div>

        <Dialog
            :visible="nameOpen"
            :header="nameMode === 'mkdir' ? '新建文件夹' : '重命名'"
            :style="{ width: '380px' }"
            modal
            @update:visible="nameOpen = $event"
        >
            <InputText v-model="nameValue" class="full" @keyup.enter="submitName" />
            <p v-if="opError" class="op-error">{{ opError }}</p>
            <template #footer>
                <Button label="取消" text @click="nameOpen = false" />
                <Button label="确定" @click="submitName" />
            </template>
        </Dialog>

        <Dialog
            :visible="deleteOpen"
            header="删除确认"
            :style="{ width: '400px' }"
            modal
            @update:visible="deleteOpen = $event"
        >
            <p class="conflict-text">
                将删除 <b>{{ deleteEntry?.name }}</b>
                （远端不可恢复{{ deleteEntry?.is_dir ? '，目录会被递归删除' : '' }}）
            </p>
            <p v-if="opError" class="op-error">{{ opError }}</p>
            <template #footer>
                <Button label="取消" text @click="deleteOpen = false" />
                <Button label="删除" severity="danger" @click="confirmDelete" />
            </template>
        </Dialog>

        <Dialog
            :visible="!!conflict"
            header="目标已存在"
            :style="{ width: '420px' }"
            modal
            @update:visible="conflict = null"
        >
            <p class="conflict-text">
                <b>{{ conflict?.entry.name }}</b> 已存在于目标目录，如何处理？
            </p>
            <template #footer>
                <Button label="跳过" text @click="resolveSkip" />
                <Button label="新建副本" severity="secondary" @click="resolveRename" />
                <Button label="覆盖" severity="danger" @click="resolveOverwrite" />
            </template>
        </Dialog>
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

.queue {
    border-top: 1px solid #eef0f3;
    padding: 6px 16px 4px;
    max-height: 150px;
    overflow-y: auto;
}

.queue-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11px;
    color: #6b7280;
    padding-bottom: 4px;
}

.link {
    border: 0;
    background: transparent;
    color: #2f6feb;
    font-size: 11px;
    cursor: pointer;
}

.queue-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.task {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #374151;
}

.task i {
    font-size: 10px;
    color: #9ca3af;
}

.task-name {
    width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.bar {
    flex: 1;
    height: 5px;
    border-radius: 3px;
    background: #eef0f3;
    overflow: hidden;
}

.fill {
    height: 100%;
    background: #2f6feb;
    transition: width 0.12s linear;
}

.fill.done {
    background: #1a7f37;
}

.fill.error {
    background: #dc2626;
}

.task-pct {
    width: 48px;
    text-align: right;
    font-size: 11px;
    color: #9ca3af;
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

.ctx {
    position: fixed;
    z-index: 50;
    min-width: 160px;
    padding: 4px;
    border: 1px solid #e3e6ea;
    border-radius: 8px;
    background: #fff;
    box-shadow: 0 8px 24px rgba(15, 23, 42, 0.14);
}

.ctx button {
    display: block;
    width: 100%;
    padding: 7px 10px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    text-align: left;
    font-size: 12px;
    color: #374151;
    cursor: pointer;
}

.ctx button:hover {
    background: #f3f5f8;
}

.ctx button.danger {
    color: #dc2626;
}

.ctx button.danger:hover {
    background: #fdecec;
}

.full {
    width: 100%;
}

.op-error {
    margin: 8px 0 0;
    font-size: 12px;
    color: #dc2626;
}

.ctx-sep {
    height: 1px;
    margin: 4px 6px;
    background: #eef0f3;
}

.conflict-text {
    margin: 0;
    font-size: 13px;
    color: #374151;
}
</style>
