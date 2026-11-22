<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { useSystemStore } from '@/stores/system';
import { useSftpStore } from '@/stores/sftp';
import { useTerminalsStore } from '@/stores/terminals';
import { resolveHostAuth } from '@/lib/auth';

const route = useRoute();
const router = useRouter();
const system = useSystemStore();
const sftp = useSftpStore();
const terminals = useTerminalsStore();

const host = computed(() => system.getHost(route.params.hostId as string) ?? null);
const shellId = computed(() =>
    host.value
        ? `term-${host.value.address}:${host.value.port ?? 22}`
        : `term-${route.params.hostId}`,
);

const session = computed(() => terminals.sessions[shellId.value]);
const status = computed(() => session.value?.status ?? 'idle');
const error = computed(() => session.value?.error ?? '');

const box = ref<HTMLElement | null>(null);

let term: Terminal | null = null;
let fit: FitAddon | null = null;
let observer: ResizeObserver | null = null;
let stopWatch: (() => void) | null = null;
let written = 0;

const encode = (text: string) => {
    const bytes = new TextEncoder().encode(text);
    let binary = '';
    bytes.forEach((byte) => (binary += String.fromCharCode(byte)));
    return btoa(binary);
};

const writeError = (message: string) => {
    term?.writeln('');
    term?.writeln(`\x1b[31m${message}\x1b[0m`);
};

const writeNewChunks = () => {
    const current = terminals.sessions[shellId.value];
    if (!current || !term) return;
    for (; written < current.chunks.length; written += 1) {
        term.write(current.chunks[written]);
    }
};

const start = async () => {
    const current = host.value;
    if (!current) return;

    const target = terminals.ensure(shellId.value);

    let auth;
    try {
        auth = await resolveHostAuth(current);
    } catch (e) {
        target.status = 'error';
        target.error = String(e);
        writeError(target.error);
        return;
    }

    const ok = await terminals.open(
        shellId.value,
        {
            host: current.address,
            port: current.port ?? 22,
            username: current.username || 'root',
            auth,
        },
        term?.cols ?? 80,
        term?.rows ?? 24,
    );

    if (ok) {
        term?.focus();
    } else {
        writeError(terminals.sessions[shellId.value]?.error ?? '连接失败');
    }
};

const reconnect = async () => {
    if (!term) return;
    term.reset();
    written = 0;
    await terminals.close(shellId.value);
    await start();
};

const openFiles = () => {
    if (!host.value) return;
    sftp.bind(host.value);
    router.push('/sftp');
};

onMounted(() => {
    if (!box.value || !host.value) return;

    term = new Terminal({
        fontFamily: 'Menlo, Monaco, "SF Mono", monospace',
        fontSize: 13,
        cursorBlink: true,
        scrollback: 10000,
        theme: {
            background: '#1e1f22',
            foreground: '#d6d9df',
            cursor: '#d6d9df',
            selectionBackground: '#3a4150',
        },
    });
    fit = new FitAddon();
    term.loadAddon(fit);
    term.open(box.value);
    fit.fit();

    term.onData((data) => {
        invoke('shell_write', { shellId: shellId.value, data: encode(data) }).catch(() => {});
    });

    term.onResize(({ cols, rows }) => {
        invoke('shell_resize', { shellId: shellId.value, cols, rows }).catch(() => {});
    });

    observer = new ResizeObserver(() => {
        fit?.fit();
    });
    observer.observe(box.value);

    const existing = terminals.ensure(shellId.value);
    written = 0;
    writeNewChunks();
    stopWatch = watch(
        () => terminals.sessions[shellId.value]?.chunks.length ?? 0,
        () => writeNewChunks(),
    );

    if (existing.status === 'ready') {
        term.focus();
    } else if (existing.status !== 'connecting') {
        start();
    }
});

onBeforeUnmount(() => {
    stopWatch?.();
    observer?.disconnect();
    // 注意：不关闭 shell。会话留在 store 里，切回标签时直接重放缓冲。
    term?.dispose();
    term = null;
});

const statusText = computed(() => {
    if (status.value === 'ready') return '已连接';
    if (status.value === 'error') return '连接失败';
    if (status.value === 'connecting') return '连接中…';
    return '未连接';
});
</script>

<template>
    <div class="terminal-view">
        <header class="head">
            <i class="pi pi-desktop" />
            <span class="name">{{ host?.name || host?.address || '终端' }}</span>
            <span class="pill" :class="status">{{ statusText }}</span>
            <span class="addr" v-if="host">{{ host.username || 'root' }}@{{ host.address }}</span>
            <span class="err" v-if="error">{{ error }}</span>
            <span class="spacer" />
            <Button label="重连" size="small" text @click="reconnect" />
            <Button label="文件" size="small" text @click="openFiles" />
        </header>
        <div ref="box" class="term-box" />
    </div>
</template>

<style scoped>
.terminal-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1f22;
}

.head {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 38px;
    padding: 0 14px;
    background: #26282d;
    border-bottom: 1px solid #33363d;
    color: #d6d9df;
    flex-shrink: 0;
}

.head > i {
    font-size: 13px;
    color: #9aa1ab;
}

.name {
    font-size: 13px;
    font-weight: 600;
}

.addr {
    font-size: 11px;
    color: #8b93a1;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.pill {
    padding: 2px 8px;
    border-radius: 999px;
    background: #3a3f4b;
    color: #c4cad4;
    font-size: 11px;
}

.pill.ready {
    background: #1f3a2a;
    color: #7ddba0;
}

.pill.error {
    background: #3d2426;
    color: #f2a6a6;
}

.err {
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 11px;
    color: #f2a6a6;
}

.spacer {
    flex: 1;
}

.term-box {
    flex: 1;
    min-height: 0;
    padding: 6px 8px;
    overflow: hidden;
}

.term-box :deep(.xterm) {
    height: 100%;
}
</style>
