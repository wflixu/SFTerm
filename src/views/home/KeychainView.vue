<script setup lang="ts">
import { ref, watch } from 'vue';
import { useKeychainStore } from '@/stores/keychain';
import { useConfirm } from 'primevue/useconfirm';
import { keyTypeOf } from '@/lib/keytype';
import { keyPrivateAccount, secretDelete, secretGet, secretSet } from '@/lib/secrets';
import type { IKeyEntry } from '@/shared/interface';

const store = useKeychainStore();
const ask = useConfirm();

const dialogOpen = ref(false);
const editing = ref<IKeyEntry | null>(null);
const form = ref({ label: '', privateKey: '', publicKey: '' });
const error = ref('');

const openCreate = () => {
    editing.value = null;
    error.value = '';
    form.value = { label: '', privateKey: '', publicKey: '' };
    dialogOpen.value = true;
};

const openEdit = async (entry: IKeyEntry) => {
    editing.value = entry;
    error.value = '';
    form.value = { label: entry.label, privateKey: '', publicKey: entry.publicKey };
    dialogOpen.value = true;
    try {
        form.value.privateKey = (await secretGet(keyPrivateAccount(entry.id))) ?? '';
    } catch (e) {
        error.value = `读取系统钥匙串失败：${e}`;
    }
};

const save = async () => {
    if (!form.value.label.trim()) {
        error.value = '请填写 Label';
        return;
    }

    const type = keyTypeOf(form.value.privateKey);
    try {
        if (editing.value) {
            if (form.value.privateKey) {
                await secretSet(keyPrivateAccount(editing.value.id), form.value.privateKey);
            }
            store.update({
                ...editing.value,
                label: form.value.label.trim(),
                publicKey: form.value.publicKey,
                privateKey: undefined,
                type: type === '未知' ? editing.value.type : type,
            });
        } else {
            if (!form.value.privateKey) {
                error.value = '请粘贴私钥，或从文件导入';
                return;
            }
            const id = `${Date.now()}`;
            await secretSet(keyPrivateAccount(id), form.value.privateKey);
            store.add({ id, label: form.value.label.trim(), publicKey: form.value.publicKey, type });
        }
        error.value = '';
        dialogOpen.value = false;
    } catch (e) {
        error.value = `保存到系统钥匙串失败：${e}`;
    }
};

const remove = (entry: IKeyEntry) => {
    ask.require({
        header: '删除密钥',
        message: `删除密钥「${entry.label}」？私钥会同时从系统钥匙串移除。`,
        icon: 'pi pi-exclamation-triangle',
        rejectProps: { label: '取消', severity: 'secondary', text: true },
        acceptProps: { label: '删除', severity: 'danger' },
        accept: async () => {
            try {
                await secretDelete(keyPrivateAccount(entry.id));
            } catch (e) {
                console.error('[keychain] 删除钥匙串条目失败', entry.id, e);
            }
            store.remove(entry.id);
        },
    });
};

const importFile = async (event: Event, kind: 'privateKey' | 'publicKey') => {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    form.value[kind] = await file.text();
    if (!form.value.label) form.value.label = file.name.replace(/\.pub$/, '');
    input.value = '';
};

const typeOf = (entry: IKeyEntry) => entry.type ?? keyTypeOf(entry.privateKey);

const selected = ref('');

const publicKeyPreview = (publicKey: string) => {
    const parts = publicKey.trim().split(/\s+/);
    if (!parts[0]) return '未保存公钥';
    if (parts.length < 2) return parts[0];
    const body = parts[1];
    const short = body.length > 18 ? `${body.slice(0, 12)}…${body.slice(-4)}` : body;
    return `${parts[0]} ${short}`;
};

// 关闭对话框后不把私钥文本留在内存里。
watch(dialogOpen, (open) => {
    if (!open) form.value.privateKey = '';
});
</script>

<template>
  <div class="keychain">
    <header class="toolbar">
      <div class="title">
        <h2>Keychain</h2>
        <span class="count">{{ store.entries.length }}</span>
      </div>
      <Button label="新建密钥" icon="pi pi-plus" size="small" @click="openCreate" />
    </header>

    <div v-if="store.entries.length" class="grid">
      <article
        v-for="entry in store.entries"
        :key="entry.id"
        class="card"
        :class="{ selected: selected === entry.id }"
        @click="selected = entry.id"
        @dblclick="openEdit(entry)"
      >
        <header class="card-head">
          <div class="icon"><i class="pi pi-key" /></div>
          <div class="headings">
            <div class="label">{{ entry.label }}</div>
            <span class="type">{{ typeOf(entry) }}</span>
          </div>
          <div class="actions">
            <button class="icon-btn" title="编辑" @click.stop="openEdit(entry)">
              <i class="pi pi-pen-to-square" />
            </button>
            <button class="icon-btn danger" title="删除" @click.stop="remove(entry)">
              <i class="pi pi-trash" />
            </button>
          </div>
        </header>
        <div class="pub" :title="entry.publicKey">{{ publicKeyPreview(entry.publicKey) }}</div>
      </article>
    </div>

    <div v-else class="empty">
      <div class="empty-icon"><i class="pi pi-key" /></div>
      <h3>还没有密钥</h3>
      <p>保存常用私钥，添加主机时直接选用。</p>
    </div>

    <Dialog v-model:visible="dialogOpen" :header="editing ? '编辑密钥' : '新建密钥'" :style="{ width: '520px' }" modal>
      <div class="dialog-body">
        <div class="field">
          <label>Label</label>
          <InputText v-model="form.label" placeholder="例如 tencent-light" />
        </div>
        <div class="field">
          <label>Private Key</label>
          <textarea v-model="form.privateKey" rows="7" spellcheck="false" placeholder="-----BEGIN OPENSSH PRIVATE KEY-----" />
          <input type="file" @change="importFile($event, 'privateKey')" />
        </div>
        <div class="field">
          <label>Public Key</label>
          <textarea v-model="form.publicKey" rows="3" spellcheck="false" placeholder="ssh-ed25519 AAAA..." />
          <input type="file" @change="importFile($event, 'publicKey')" />
        </div>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
      <template #footer>
        <Button label="取消" severity="secondary" text @click="dialogOpen = false" />
        <Button label="保存" @click="save" />
      </template>
    </Dialog>
  </div>
</template>

<style scoped>
.keychain {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
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

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  align-content: start;
  gap: 12px;
  padding: 16px;
  overflow-y: auto;
}

.card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 14px;
  border: 1px solid #e3e6ea;
  border-radius: 10px;
  background: #fff;
  cursor: default;
  transition: border-color 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease;
  user-select: none;
}

.card:hover {
  border-color: #c7d2fe;
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.06);
  transform: translateY(-1px);
}

.card.selected {
  border-color: #2563eb;
  box-shadow: 0 0 0 1px #2563eb;
}

.card-head {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  border-radius: 8px;
  /* 和主机卡片的灰色图标区分开：密钥用暖色 */
  background: #fef3c7;
  color: #b45309;
}

.headings {
  flex: 1;
  min-width: 0;
}

.label {
  overflow: hidden;
  color: #111827;
  font-size: 13px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.type {
  display: inline-block;
  margin-top: 3px;
  padding: 0 6px;
  border-radius: 999px;
  background: #eef2ff;
  color: #4338ca;
  font-size: 10px;
  line-height: 16px;
}

.actions {
  display: flex;
  gap: 2px;
  opacity: 0.55;
  transition: opacity 0.15s ease;
}

.card:hover .actions,
.card.selected .actions {
  opacity: 1;
}

.pub {
  overflow: hidden;
  color: #6b7280;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: #6b7280;
}

.empty-icon {
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

.empty h3 {
  margin: 0;
  font-size: 15px;
  color: #111827;
}

.empty p {
  margin: 0;
  font-size: 13px;
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
  font-size: 12px;
  color: #6b7280;
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

.field input[type='file'] {
  font-size: 12px;
}
.error {
    margin: 0;
    font-size: 12px;
    color: #dc2626;
}</style>
