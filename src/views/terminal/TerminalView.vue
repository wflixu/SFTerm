<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { invoke } from '@tauri-apps/api/core';
import { useSystemStore } from '@/stores/system';
import { useSftpStore } from '@/stores/sftp';
import { useTerminalsStore } from '@/stores/terminals';
import { useHistoryStore } from '@/stores/history';
import { useSnippetsStore } from '@/stores/snippets';
import { resolveHostAuth } from '@/lib/auth';

const route = useRoute();
const router = useRouter();
const system = useSystemStore();
const sftp = useSftpStore();
const terminals = useTerminalsStore();
const history = useHistoryStore();
const snippets = useSnippetsStore();

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

type Suggestion = { command: string; kind: 'snippet' | 'history' };
const suggestions = ref<Suggestion[]>([]);
const activeIndex = ref(0);
const popupPos = ref({ left: 0, top: 0 });
let inputLine = '';

const hideSuggestions = () => {
    suggestions.value = [];
    activeIndex.value = 0;
};

const bufferLineBeforeCursor = () => {
    const buffer = term?.buffer.active;
    if (!buffer) return '';
    const line = buffer.getLine(buffer.baseY + buffer.cursorY);
    return line ? line.translateToString(true).slice(0, buffer.cursorX) : '';
};

const positionPopup = () => {
    if (!term || !box.value || !suggestions.value.length) return;
    const screen = box.value.querySelector('.xterm-screen') as HTMLElement | null;
    if (!screen) return;
    const screenRect = screen.getBoundingClientRect();
    const boxRect = box.value.getBoundingClientRect();
    const cellWidth = screenRect.width / term.cols;
    const cellHeight = screenRect.height / term.rows;
    const buffer = term.buffer.active;

    // 参考设计稿：候选框与「正在输入的第一个字符」左对齐，而不是跟着光标跑。
    const startColumn = Math.max(0, buffer.cursorX - inputLine.length);
    const left = screenRect.left - boxRect.left + startColumn * cellWidth;
    const top = screenRect.top - boxRect.top + (buffer.cursorY + 1) * cellHeight + 2;
    popupPos.value = {
        left: Math.max(4, Math.min(left, boxRect.width - 324)),
        top,
    };
};

const updateSuggestions = () => {
    // vim / tmux 等全屏程序在 alternate buffer 里，不做补全。
    if (term?.buffer.active.type === 'alternate') {
        hideSuggestions();
        return;
    }
    const needle = inputLine.trim().toLowerCase();
    if (needle.length < 2) {
        hideSuggestions();
        return;
    }

    const saved = snippets.entries
        .filter((item) => item.command.toLowerCase().includes(needle))
        .sort((a, b) => {
            const aPrefix = a.command.toLowerCase().startsWith(needle) ? 0 : 1;
            const bPrefix = b.command.toLowerCase().startsWith(needle) ? 0 : 1;
            return aPrefix - bPrefix || a.command.length - b.command.length;
        })
        .map((item) => ({ command: item.command, kind: 'snippet' as const }));

    const savedCommands = new Set(snippets.entries.map((item) => item.command.trim()));
    const recent = history.entries
        .filter((item) => !savedCommands.has(item.command.trim()))
        .filter((item) => item.command.toLowerCase().includes(needle))
        .map((item) => ({ command: item.command, kind: 'history' as const }));

    suggestions.value = [...saved, ...recent].slice(0, 7);
    activeIndex.value = 0;
    positionPopup();
};

const chooseSuggestion = (index = activeIndex.value) => {
    const item = suggestions.value[index];
    if (!item || !term) return;
    // 如果远端行尾和我们跟踪的输入对不上（比如光标被移动过），宁可不补全。
    if (!bufferLineBeforeCursor().endsWith(inputLine)) {
        hideSuggestions();
        return;
    }
    const data = '\x7f'.repeat(inputLine.length) + item.command;
    void invoke('shell_write', { shellId: shellId.value, data: encode(data) }).catch(() => {});
    inputLine = item.command;
    hideSuggestions();
    term.focus();
};

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
    if (!history.loaded) void history.load();

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

    // 粗略记录用户输入的命令：遇到回车就落一条历史。
    // vim / tmux 这类全屏程序会产生噪声，后续可用 OSC 133 替代。
    const commitLine = () => {
        const command = inputLine.trim();
        const alternate = term?.buffer.active.type === 'alternate';
        inputLine = '';
        hideSuggestions();
        if (!command || alternate) return;
        const current = host.value;
        void history.record(
            current?.id ?? '',
            current?.name || current?.address || '未知主机',
            command,
        );
    };

    term.attachCustomKeyEventHandler((event) => {
        if (event.type !== 'keydown' || !suggestions.value.length) return true;
        if (event.key === 'ArrowDown') {
            event.preventDefault();
            activeIndex.value = (activeIndex.value + 1) % suggestions.value.length;
            return false;
        }
        if (event.key === 'ArrowUp') {
            event.preventDefault();
            activeIndex.value =
                (activeIndex.value - 1 + suggestions.value.length) % suggestions.value.length;
            return false;
        }
        if (event.key === 'Tab' || event.key === 'Enter') {
            event.preventDefault();
            chooseSuggestion();
            return false;
        }
        if (event.key === 'Escape') {
            event.preventDefault();
            hideSuggestions();
            return false;
        }
        return true;
    });

    term.onData((data) => {
        if (!data.startsWith('\x1b')) {
            for (const ch of data) {
                if (ch === '\r' || ch === '\n') commitLine();
                else if (ch === '\x7f') inputLine = inputLine.slice(0, -1);
                else if (ch === '\x03') {
                    inputLine = '';
                    hideSuggestions();
                } else if (ch >= ' ') inputLine += ch;
            }
            updateSuggestions();
        }
        invoke('shell_write', { shellId: shellId.value, data: encode(data) }).catch(() => {});
    });

    term.onResize(({ cols, rows }) => {
        invoke('shell_resize', { shellId: shellId.value, cols, rows }).catch(() => {});
        positionPopup();
    });
    term.onScroll(positionPopup);

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
        <div ref="box" class="term-box">
            <div
                v-if="suggestions.length"
                class="completion"
                :style="{ left: `${popupPos.left}px`, top: `${popupPos.top}px` }"
            >
                <button
                    v-for="(item, index) in suggestions"
                    :key="`${item.kind}-${item.command}`"
                    class="completion-item"
                    :class="{ active: index === activeIndex }"
                    @mouseenter="activeIndex = index"
                    @mousedown.prevent="chooseSuggestion(index)"
                >
                    <span class="completion-icon" :class="item.kind">
                        <i
                            class="pi"
                            :class="item.kind === 'snippet' ? 'pi-bookmark' : 'pi-clock'"
                        />
                    </span>
                    <span class="completion-command">{{ item.command }}</span>
                    <span v-if="index === activeIndex" class="completion-enter">↵</span>
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.completion {
    position: absolute;
    z-index: 20;
    width: 315px;
    max-width: calc(100% - 8px);
    max-height: 208px;
    overflow-y: auto;
    padding: 4px;
    border: 1px solid #2e3448;
    border-radius: 10px;
    background: #25293c;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
}

.completion-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: 28px;
    padding: 0 8px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: #c0caf5;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
    text-align: left;
    cursor: pointer;
}

.completion-item.active {
    background: #313448;
    color: #fff;
}

.completion-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    border: 1px solid #4a5270;
    border-radius: 50%;
    color: #9aa5ce;
    font-size: 10px;
}

.completion-icon.snippet {
    border-color: #4f6bb0;
    color: #7aa2f7;
}

.completion-command {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.completion-enter {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 18px;
    flex-shrink: 0;
    border-radius: 5px;
    background: #3a3f55;
    color: #c0caf5;
    font-size: 12px;
}

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
    position: relative;
    flex: 1;
    min-height: 0;
    padding: 6px 8px;
    overflow: hidden;
}

.term-box :deep(.xterm) {
    height: 100%;
}
</style>
