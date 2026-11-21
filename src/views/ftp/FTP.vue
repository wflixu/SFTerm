<template>
    <div class="probe">
        <h2>连接测试</h2>
        <p class="hint">临时页面：验证 Rust 引擎 → 前端命令链路，后续被双栏文件页替换。</p>

        <div class="row">
            <InputText v-model="form.host" placeholder="主机" />
            <InputText v-model="form.port" placeholder="端口" class="port" />
            <InputText v-model="form.user" placeholder="用户名" />
        </div>
        <div class="row">
            <InputText v-model="form.keyPath" placeholder="私钥路径" class="grow" />
            <InputText v-model="form.path" placeholder="远端路径" />
        </div>
        <div class="row">
            <Button label="连接" :loading="connecting" @click="connect" />
            <Button label="列目录" :disabled="!info" @click="list" />
            <Button label="断开" severity="secondary" :disabled="!info" @click="disconnect" />
        </div>

        <pre v-if="error" class="error">{{ error }}</pre>

        <div v-if="info" class="info">
            <div><b>服务器</b>{{ info.banner }}</div>
            <div><b>指纹</b>{{ info.fingerprint_sha256 }}</div>
            <div><b>算法</b>{{ info.kex }} · {{ info.cipher }}</div>
            <div><b>会话</b>{{ info.session_id }}</div>
        </div>

        <ul v-if="entries.length" class="entries">
            <li v-for="entry in entries" :key="entry.path">
                <span class="kind">{{ entry.is_dir ? 'DIR' : entry.is_symlink ? 'LNK' : 'FILE' }}</span>
                <span class="name">{{ entry.name }}</span>
                <span class="size">{{ entry.is_dir ? '' : entry.size }}</span>
            </li>
        </ul>
    </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

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
}

const form = reactive({
    host: '',
    port: '22',
    user: '',
    keyPath: '',
    path: '/home',
});

const info = ref<ConnectInfo | null>(null);
const entries = ref<RemoteEntry[]>([]);
const error = ref('');
const connecting = ref(false);

const connect = async () => {
    error.value = '';
    entries.value = [];
    connecting.value = true;
    try {
        info.value = await invoke<ConnectInfo>('ssh_connect', {
            params: {
                host: form.host,
                port: Number(form.port),
                username: form.user,
                auth: { kind: 'key', path: form.keyPath },
            },
        });
    } catch (e) {
        error.value = String(e);
    } finally {
        connecting.value = false;
    }
};

const list = async () => {
    if (!info.value) return;
    error.value = '';
    try {
        entries.value = await invoke<RemoteEntry[]>('sftp_list_dir', {
            sessionId: info.value.session_id,
            path: form.path,
        });
    } catch (e) {
        error.value = String(e);
    }
};

const disconnect = async () => {
    if (!info.value) return;
    await invoke('ssh_disconnect', { sessionId: info.value.session_id });
    info.value = null;
    entries.value = [];
};
</script>

<style scoped>
.probe {
    padding: 24px 32px;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.hint {
    color: #888;
    font-size: 13px;
}

.row {
    display: flex;
    gap: 8px;
    align-items: center;
}

.port {
    width: 90px;
}

.grow {
    flex: 1;
}

.error {
    color: #c0392b;
    background: #fdf2f0;
    padding: 12px;
    border-radius: 6px;
    white-space: pre-wrap;
}

.info {
    display: grid;
    gap: 4px;
    font-size: 13px;
    color: #444;
}

.info b {
    display: inline-block;
    width: 60px;
    color: #888;
    font-weight: 500;
}

.entries {
    list-style: none;
    padding: 0;
    margin: 0;
    border-top: 1px solid #eee;
}

.entries li {
    display: flex;
    gap: 12px;
    padding: 6px 4px;
    border-bottom: 1px solid #f2f2f2;
    font-size: 13px;
}

.kind {
    width: 44px;
    color: #999;
}

.name {
    flex: 1;
}

.size {
    color: #999;
}
</style>
