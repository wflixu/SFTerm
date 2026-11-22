<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useConfirm } from 'primevue/useconfirm';
import type { ISnippet } from '@/shared/interface';
import { useSnippetsStore } from '@/stores/snippets';
import { useHistoryStore } from '@/stores/history';

const store = useSnippetsStore();
const history = useHistoryStore();
const ask = useConfirm();

const search = ref('');
const dialogOpen = ref(false);
const editing = ref<ISnippet | null>(null);
const error = ref('');
const form = ref({ label: '', command: '', group: '' });

const filtered = computed(() => {
    const q = search.value.trim().toLowerCase();
    if (!q) return store.entries;
    return store.entries.filter((item) =>
        [item.label, item.command, item.group ?? ''].some((value) =>
            value.toLowerCase().includes(q),
        ),
    );
});

const recent = computed(() => history.entries.slice(0, 20));

const isSaved = (command: string) =>
    store.entries.some((item) => item.command.trim() === command.trim());

const openCreate = (prefill?: { label?: string; command?: string }) => {
    editing.value = null;
    error.value = '';
    form.value = {
        label: prefill?.label ?? '',
        command: prefill?.command ?? '',
        group: '',
    };
    dialogOpen.value = true;
};

const openEdit = (snippet: ISnippet) => {
    editing.value = snippet;
    error.value = '';
    form.value = {
        label: snippet.label,
        command: snippet.command,
        group: snippet.group ?? '',
    };
    dialogOpen.value = true;
};

const save = () => {
    const label = form.value.label.trim();
    const command = form.value.command.trim();
    if (!label) {
        error.value = '请填写名称';
        return;
    }
    if (!command) {
        error.value = '请填写命令';
        return;
    }
    if (editing.value) {
        store.update({ ...editing.value, label, command, group: form.value.group.trim() || undefined });
    } else {
        store.add({
            id: `${Date.now()}`,
            label,
            command,
            group: form.value.group.trim() || undefined,
        });
    }
    dialogOpen.value = false;
};

const remove = (snippet: ISnippet) => {
    ask.require({
        header: '删除片段',
        message: `删除片段「${snippet.label}」？`,
        icon: 'pi pi-exclamation-triangle',
        rejectProps: { label: '取消', severity: 'secondary', text: true },
        acceptProps: { label: '删除', severity: 'danger' },
        accept: () => store.remove(snippet.id),
    });
};

const saveFromHistory = (command: string) => {
    const label = command.length > 24 ? `${command.slice(0, 24)}…` : command;
    openCreate({ label, command });
};

const formatTime = (ms: number) => {
    const date = new Date(ms);
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
};

onMounted(() => {
    if (!history.loaded) history.load();
});
</script>

<template>
    <div class="snippets">
        <header class="toolbar">
            <div class="title">
                <h2>Snippets</h2>
                <span class="count">{{ store.entries.length }}</span>
            </div>
            <div class="actions">
                <InputText v-model="search" placeholder="搜索片段" class="search" />
                <Button label="新建片段" icon="pi pi-plus" size="small" @click="openCreate()" />
            </div>
        </header>

        <div class="body">
            <section class="section">
                <h3>片段<span class="hint">终端里输入时会自动联想</span></h3>
                <div v-if="filtered.length" class="grid">
                    <article v-for="snippet in filtered" :key="snippet.id" class="card">
                        <header class="card-head">
                            <div class="label">{{ snippet.label }}</div>
                            <div class="card-actions">
                                <button class="icon-btn" title="编辑" @click="openEdit(snippet)">
                                    <i class="pi pi-pen-to-square" />
                                </button>
                                <button class="icon-btn danger" title="删除" @click="remove(snippet)">
                                    <i class="pi pi-trash" />
                                </button>
                            </div>
                        </header>
                        <code class="command">{{ snippet.command }}</code>
                        <div v-if="snippet.group" class="group">{{ snippet.group }}</div>
                    </article>
                </div>
                <div v-else class="empty">
                    <div class="empty-icon"><i class="pi pi-code" /></div>
                    <h4>{{ store.entries.length ? '没有匹配的片段' : '还没有片段' }}</h4>
                    <p>{{ store.entries.length ? '换个关键词试试。' : '保存常用命令，在终端里输入时就能补全。' }}</p>
                    <Button v-if="!store.entries.length" label="新建片段" size="small" @click="openCreate()" />
                </div>
            </section>

            <section class="section">
                <h3>最近命令<span class="hint">保存后会出现在联想列表里</span></h3>
                <div v-if="recent.length" class="recent">
                    <div v-for="item in recent" :key="item.id" class="row">
                        <div class="row-main">
                            <div class="meta">
                                <span class="host">{{ item.hostLabel }}</span>
                                <span class="time">{{ formatTime(item.executedAt) }}</span>
                            </div>
                            <code class="command">{{ item.command }}</code>
                        </div>
                        <Button
                            :label="isSaved(item.command) ? '已保存' : '存为片段'"
                            :disabled="isSaved(item.command)"
                            size="small"
                            text
                            @click="saveFromHistory(item.command)"
                        />
                    </div>
                </div>
                <p v-else class="muted">还没有执行记录；在终端里执行几条命令后会出现在这里。</p>
            </section>
        </div>

        <Dialog
            v-model:visible="dialogOpen"
            :header="editing ? '编辑片段' : '新建片段'"
            :style="{ width: '520px' }"
            modal
        >
            <div class="dialog-body">
                <div class="field">
                    <label>名称</label>
                    <InputText v-model="form.label" placeholder="例如 部署前端" />
                </div>
                <div class="field">
                    <label>命令</label>
                    <textarea v-model="form.command" rows="4" spellcheck="false" placeholder="npm run build && rsync -av dist/ server:/var/www" />
                </div>
                <div class="field">
                    <label>分组（可选）</label>
                    <InputText v-model="form.group" placeholder="例如 部署" />
                </div>
                <p v-if="error" class="error">{{ error }}</p>
            </div>
            <template #footer>
                <Button label="取消" severity="secondary" text @click="dialogOpen = false" />
                <Button label="保存" @click="save" />
            </template>
        </Dialog>
    </div>
</template>

<style scoped>
.snippets {
    display: flex;
    flex-direction: column;
    height: 100%;
}

.toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 24px;
    border-bottom: 1px solid #e3e6ea;
    background: #fff;
}

.title {
    display: flex;
    align-items: baseline;
    gap: 8px;
}

.title h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: #111827;
}

.count {
    font-size: 12px;
    color: #9ca3af;
}

.actions {
    display: flex;
    align-items: center;
    gap: 8px;
}

.search {
    width: 220px;
}

.body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 18px 24px 28px;
}

.section + .section {
    margin-top: 26px;
}

.section h3 {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin: 0 0 12px;
    font-size: 13px;
    font-weight: 600;
    color: #111827;
}

.hint {
    color: #9ca3af;
    font-size: 11px;
    font-weight: 400;
}

.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 12px;
}

.card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    border: 1px solid #e3e6ea;
    border-radius: 10px;
    background: #fff;
    transition: border-color 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease;
}

.card:hover {
    border-color: #c7d2fe;
    box-shadow: 0 4px 12px rgba(15, 23, 42, 0.06);
    transform: translateY(-1px);
}

.card-head {
    display: flex;
    align-items: flex-start;
    gap: 8px;
}

.label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: #111827;
    font-size: 13px;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.card-actions {
    display: flex;
    gap: 2px;
    opacity: 0.55;
}

.card:hover .card-actions {
    opacity: 1;
}

.command {
    color: #4b5563;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 11px;
    white-space: pre-wrap;
    word-break: break-all;
}

.group {
    align-self: flex-start;
    padding: 0 6px;
    border-radius: 999px;
    background: #eef2ff;
    color: #4338ca;
    font-size: 10px;
    line-height: 16px;
}

.recent {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 12px;
    border: 1px solid #e3e6ea;
    border-radius: 8px;
    background: #fff;
}

.row-main {
    flex: 1;
    min-width: 0;
}

.meta {
    display: flex;
    gap: 8px;
    margin-bottom: 2px;
    color: #9ca3af;
    font-size: 11px;
}

.host {
    color: #4b5563;
    font-weight: 500;
}

.empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 36px 0;
    text-align: center;
    color: #6b7280;
}

.empty-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 52px;
    height: 52px;
    margin-bottom: 4px;
    border-radius: 13px;
    background: #eceff3;
    color: #9ca3af;
    font-size: 20px;
}

.empty h4 {
    margin: 0;
    color: #111827;
    font-size: 14px;
}

.empty p {
    margin: 0;
    font-size: 12px;
}

.muted {
    margin: 0;
    color: #9ca3af;
    font-size: 12px;
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

.icon-btn.danger:hover {
    background: #fdecec;
    color: #dc2626;
}

.dialog-body {
    display: flex;
    flex-direction: column;
    gap: 14px;
}

.field {
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.field label {
    color: #6b7280;
    font-size: 12px;
}

.field textarea {
    width: 100%;
    padding: 8px 10px;
    border: 1px solid #d6dae0;
    border-radius: 6px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
    resize: vertical;
}

.error {
    margin: 0;
    color: #dc2626;
    font-size: 12px;
}
</style>
