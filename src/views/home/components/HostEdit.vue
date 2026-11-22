<script setup lang="ts">
import { reactive, ref } from 'vue';
import type { IHost } from '@/shared/interface';

const props = defineProps<{ host: IHost | null }>();
const emit = defineEmits<{ save: [host: IHost]; cancel: [] }>();

const form = reactive({
  id: props.host?.id ?? `${Date.now()}`,
  name: props.host?.name ?? '',
  address: props.host?.address ?? '',
  port: `${props.host?.port ?? 22}`,
  username: props.host?.username ?? '',
  authKind: (props.host?.password ? 'password' : 'key') as 'key' | 'password',
  keyPath: props.host?.keyPath ?? '',
  password: props.host?.password ?? '',
  group: props.host?.group ?? '',
  desc: props.host?.desc ?? '',
});

const error = ref('');

const save = () => {
  if (!form.address.trim()) {
    error.value = '地址不能为空';
    return;
  }
  if (form.authKind === 'key' && !form.keyPath.trim()) {
    error.value = '请填写私钥路径';
    return;
  }
  if (form.authKind === 'password' && !form.password) {
    error.value = '请填写密码';
    return;
  }
  error.value = '';
  emit('save', {
    id: form.id,
    name: form.name.trim() || form.address.trim(),
    address: form.address.trim(),
    port: Number(form.port) || 22,
    username: form.username.trim(),
    keyPath: form.authKind === 'key' ? form.keyPath.trim() : undefined,
    password: form.authKind === 'password' ? form.password : undefined,
    group: form.group.trim() || undefined,
    desc: form.desc.trim() || undefined,
  });
};
</script>

<template>
  <div class="form">
    <div class="section">
      <label>名称</label>
      <InputText v-model="form.name" placeholder="例如 web-1" />
    </div>

    <div class="row">
      <div class="section grow">
        <label>地址</label>
        <InputText v-model="form.address" placeholder="IP 或域名" />
      </div>
      <div class="section port">
        <label>端口</label>
        <InputText v-model="form.port" placeholder="22" />
      </div>
    </div>

    <div class="section">
      <label>用户名</label>
      <InputText v-model="form.username" placeholder="ubuntu" />
    </div>

    <div class="section">
      <label>认证方式</label>
      <div class="segmented">
        <button :class="{ active: form.authKind === 'key' }" @click="form.authKind = 'key'">密钥</button>
        <button :class="{ active: form.authKind === 'password' }" @click="form.authKind = 'password'">密码</button>
      </div>
    </div>

    <div v-if="form.authKind === 'key'" class="section">
      <label>私钥路径</label>
      <InputText v-model="form.keyPath" placeholder="/Users/you/.ssh/id_ed25519" />
      <small>暂用文件路径；Keychain 集成后会改为从密钥库选择。</small>
    </div>

    <div v-else class="section">
      <label>密码</label>
      <InputText v-model="form.password" type="password" placeholder="登录密码" />
      <small>暂存本地，后续迁移到系统钥匙串。</small>
    </div>

    <div class="row">
      <div class="section grow">
        <label>分组</label>
        <InputText v-model="form.group" placeholder="可选" />
      </div>
    </div>

    <div class="section">
      <label>备注</label>
      <InputText v-model="form.desc" placeholder="可选" />
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div class="actions">
      <Button label="取消" severity="secondary" text @click="emit('cancel')" />
      <Button label="保存" @click="save" />
    </div>
  </div>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px;
  overflow-y: auto;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section label {
  font-size: 12px;
  color: #6b7280;
}

.section small {
  font-size: 11px;
  color: #9ca3af;
}

.row {
  display: flex;
  gap: 10px;
}

.grow {
  flex: 1;
}

.port {
  width: 90px;
}

.segmented {
  display: inline-flex;
  padding: 2px;
  border-radius: 7px;
  background: #eef0f3;
  width: fit-content;
}

.segmented button {
  padding: 5px 14px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: #4b5563;
  font-size: 12px;
  cursor: pointer;
}

.segmented button.active {
  background: #fff;
  color: #111827;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
}

.error {
  margin: 0;
  font-size: 12px;
  color: #dc2626;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 4px;
}
</style>
