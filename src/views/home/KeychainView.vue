<script setup lang="ts">
import { ref } from 'vue';
import { useKeychainStore } from '@/stores/keychain';
import type { IKeyEntry } from '@/shared/interface';

const store = useKeychainStore();

const dialogOpen = ref(false);
const editing = ref<IKeyEntry | null>(null);
const form = ref({ label: '', privateKey: '', publicKey: '' });

const openCreate = () => {
  editing.value = null;
  form.value = { label: '', privateKey: '', publicKey: '' };
  dialogOpen.value = true;
};

const openEdit = (entry: IKeyEntry) => {
  editing.value = entry;
  form.value = { label: entry.label, privateKey: entry.privateKey, publicKey: entry.publicKey };
  dialogOpen.value = true;
};

const save = () => {
  if (!form.value.label.trim()) return;
  if (editing.value) {
    store.update({ ...editing.value, ...form.value });
  } else {
    store.add({ id: `${Date.now()}`, ...form.value });
  }
  dialogOpen.value = false;
};

const remove = (entry: IKeyEntry) => {
  if (confirm(`删除密钥「${entry.label}」？`)) store.remove(entry.id);
};

const importFile = async (event: Event, kind: 'privateKey' | 'publicKey') => {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  form.value[kind] = await file.text();
  if (!form.value.label) form.value.label = file.name.replace(/\.pub$/, '');
  input.value = '';
};

const typeOf = (entry: IKeyEntry) => {
  const header = entry.privateKey.split('\n')[0] ?? '';
  if (header.includes('OPENSSH')) return 'OpenSSH';
  if (header.includes('RSA')) return 'RSA';
  if (header.includes('EC')) return 'ECDSA';
  return '未知';
};
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

    <div v-if="store.entries.length" class="list">
      <div v-for="entry in store.entries" :key="entry.id" class="row">
        <div class="icon"><i class="pi pi-key" /></div>
        <div class="info">
          <div class="label">{{ entry.label }}</div>
          <div class="meta">{{ typeOf(entry) }}</div>
        </div>
        <button class="icon-btn" title="编辑" @click="openEdit(entry)"><i class="pi pi-pen-to-square" /></button>
        <button class="icon-btn danger" title="删除" @click="remove(entry)"><i class="pi pi-trash" /></button>
      </div>
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

.list {
  padding: 12px 16px;
  overflow-y: auto;
}

.row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
}

.row:hover {
  background: #fff;
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: #f1f4f8;
  color: #4b5563;
}

.info {
  flex: 1;
  min-width: 0;
}

.label {
  font-size: 13px;
  font-weight: 500;
  color: #111827;
}

.meta {
  font-size: 11px;
  color: #9ca3af;
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
</style>
