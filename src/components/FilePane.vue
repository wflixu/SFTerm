<script setup lang="ts">
import { ref, watch } from 'vue';
import type { FileEntry } from '@/shared/interface';

const props = defineProps<{
    label: string;
    icon: string;
    path: string;
    entries: FileEntry[];
    loading?: boolean;
    emptyText?: string;
    writable?: boolean;
}>();

const emit = defineEmits<{
    navigate: [path: string];
    open: [entry: FileEntry];
    refresh: [];
    'context-menu': [entry: FileEntry, event: MouseEvent];
    'new-folder': [];
}>();

const input = ref(props.path);
const selected = ref('');

watch(
    () => props.path,
    (value) => {
        input.value = value;
    },
);

const submit = () => {
    emit('navigate', input.value.trim() || '/');
};

const up = () => {
    const trimmed = props.path.replace(/\/+$/, '');
    const index = trimmed.lastIndexOf('/');
    emit('navigate', index <= 0 ? '/' : trimmed.slice(0, index));
};

const onOpen = (entry: FileEntry) => {
    selected.value = entry.path;
    emit('open', entry);
};

const formatSize = (entry: FileEntry) => {
    if (entry.is_dir) return '—';
    const units = ['B', 'KB', 'MB', 'GB'];
    let value = entry.size;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
        value /= 1024;
        unit += 1;
    }
    return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
};

const formatTime = (mtime?: number | null) => {
    if (!mtime) return '';
    const date = new Date(mtime * 1000);
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${date.getMonth() + 1}/${date.getDate()} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
};

const iconFor = (entry: FileEntry) => {
    if (entry.is_symlink) return 'pi-link';
    return entry.is_dir ? 'pi-folder' : 'pi-file';
};
</script>

<template>
    <div class="pane">
        <header class="head">
            <div class="label">
                <i class="pi" :class="icon" />
                <span>{{ label }}</span>
            </div>
            <div class="ops">
                <button v-if="writable" class="icon-btn" title="新建文件夹" @click="emit('new-folder')">
                    <i class="pi pi-folder-plus" />
                </button>
                <button class="icon-btn" title="上级" @click="up"><i class="pi pi-arrow-up" /></button>
                <button class="icon-btn" title="刷新" @click="emit('refresh')"><i class="pi pi-refresh" /></button>
            </div>
        </header>

        <div class="pathbar">
            <input v-model="input" spellcheck="false" @keyup.enter="submit" />
        </div>

        <div class="columns">
            <span class="col-name">名称</span>
            <span class="col-size">大小</span>
            <span class="col-time">修改</span>
        </div>

        <div class="list">
            <div
                v-for="entry in entries"
                :key="entry.path"
                class="row"
                :class="{ selected: selected === entry.path }"
                @click="selected = entry.path"
                @dblclick="onOpen(entry)"
                @contextmenu.prevent="emit('context-menu', entry, $event)"
            >
                <span class="col-name">
                    <i class="pi" :class="iconFor(entry)" />
                    <span class="text">{{ entry.name }}</span>
                </span>
                <span class="col-size">{{ formatSize(entry) }}</span>
                <span class="col-time">{{ formatTime(entry.mtime) }}</span>
            </div>

            <div v-if="loading" class="state">载入中…</div>
            <div v-else-if="!entries.length" class="state">{{ emptyText ?? '空目录' }}</div>
        </div>
    </div>
</template>

<style scoped>
.pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
    background: #fff;
}

.head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 38px;
    padding: 0 12px;
    border-bottom: 1px solid #eef0f3;
}

.label {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    font-weight: 600;
    color: #111827;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.label i {
    font-size: 12px;
    color: #6b7280;
}

.ops {
    display: flex;
    gap: 2px;
}

.icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: #9ca3af;
    cursor: pointer;
}

.icon-btn:hover {
    background: #f0f2f5;
    color: #111827;
}

.pathbar {
    padding: 6px 10px;
    border-bottom: 1px solid #f3f4f6;
}

.pathbar input {
    width: 100%;
    padding: 5px 8px;
    border: 1px solid transparent;
    border-radius: 5px;
    background: #f6f7f9;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    color: #374151;
    outline: none;
}

.pathbar input:focus {
    border-color: #cfd6de;
    background: #fff;
}

.columns {
    display: flex;
    padding: 5px 12px;
    border-bottom: 1px solid #f3f4f6;
    font-size: 11px;
    color: #9ca3af;
}

.list {
    flex: 1;
    overflow-y: auto;
    padding: 2px 0 8px;
}

.row {
    display: flex;
    align-items: center;
    padding: 5px 12px;
    font-size: 12px;
    color: #374151;
    cursor: default;
    user-select: none;
}

.row:hover {
    background: #f7f8fa;
}

.row.selected {
    background: #e8f0fe;
    color: #1a4fb4;
}

.col-name {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
}

.col-name i {
    font-size: 11px;
    color: #8b93a1;
}

.row.selected .col-name i {
    color: #1a4fb4;
}

.col-name .text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.col-size {
    width: 72px;
    text-align: right;
    color: #9ca3af;
}

.col-time {
    width: 84px;
    text-align: right;
    color: #9ca3af;
}

.row.selected .col-size,
.row.selected .col-time {
    color: #5b7fd4;
}

.state {
    padding: 20px;
    text-align: center;
    font-size: 12px;
    color: #9ca3af;
}
</style>
