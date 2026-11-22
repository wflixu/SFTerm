<script setup lang="ts">
import { computed, reactive, ref } from 'vue';
import type { IHost } from '@/shared/interface';
import { useKeychainStore } from '@/stores/keychain';
import { hostPasswordAccount, secretDelete, secretSet } from '@/lib/secrets';

const keychain = useKeychainStore();
const keyOptions = computed(() =>
  keychain.entries.map((entry) => ({
    label: entry.label || '未命名密钥',
    value: entry.id,
  })),
);

const props = defineProps<{ host: IHost | null }>();
const emit = defineEmits<{ save: [host: IHost]; cancel: []; 'open-keychain': [] }>();

const hasPassword = ref(props.host?.hasPassword ?? Boolean(props.host?.password));

const form = reactive({
  id: props.host?.id ?? `${Date.now()}`,
  name: props.host?.name ?? '',
  address: props.host?.address ?? '',
  port: `${props.host?.port ?? 22}`,
  username: props.host?.username ?? '',
  authKind: (props.host?.keyPath ? 'key' : hasPassword.value ? 'password' : 'key') as
    | 'key'
    | 'password',
  keyPath: props.host?.keyPath ?? '',
  keyId: props.host?.keyId ?? '',
  password: '',
  group: props.host?.group ?? '',
  desc: props.host?.desc ?? '',
});

const error = ref('');
// 有 Keychain 条目时默认用选择器；迁移前的旧主机可能只有路径，直接把路径展开。
const showPath = ref(Boolean(props.host?.keyPath) && !props.host?.keyId);

const save = async () => {
  if (!form.address.trim()) {
    error.value = '地址不能为空';
    return;
  }
  if (form.authKind === 'key' && !form.keyId && !(showPath.value && form.keyPath.trim())) {
    error.value = keyOptions.value.length ? '请选择密钥，或填写私钥文件路径' : '还没有密钥，请先去 Keychain 新建';
    return;
  }
  if (form.authKind === 'password' && !form.password && !hasPassword.value) {
    error.value = '请填写密码';
    return;
  }

  try {
    if (form.authKind === 'password') {
      if (form.password) {
        await secretSet(hostPasswordAccount(form.id), form.password);
        hasPassword.value = true;
      }
    } else if (hasPassword.value) {
      await secretDelete(hostPasswordAccount(form.id));
      hasPassword.value = false;
    }
  } catch (e) {
    error.value = `保存到系统钥匙串失败：${e}`;
    return;
  }

  error.value = '';
  emit('save', {
    id: form.id,
    name: form.name.trim() || form.address.trim(),
    address: form.address.trim(),
    port: Number(form.port) || 22,
    username: form.username.trim(),
    keyId: form.authKind === 'key' && form.keyId ? form.keyId : undefined,
    keyPath:
      form.authKind === 'key' && !form.keyId && showPath.value ? form.keyPath.trim() : undefined,
    hasPassword: form.authKind === 'password' ? hasPassword.value : false,
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
      <label>密钥</label>

      <template v-if="keyOptions.length">
        <Select
          v-model="form.keyId"
          :options="keyOptions"
          option-label="label"
          option-value="value"
          placeholder="从 Keychain 选择密钥"
          class="key-select"
        />
      </template>

      <div v-else class="no-keys">
        <small>Keychain 里还没有密钥，新建一个后就能直接选。</small>
        <Button
          label="去 Keychain 新建"
          size="small"
          text
          icon="pi pi-plus"
          @click="emit('open-keychain')"
        />
      </div>

      <button type="button" class="link" @click="showPath = !showPath">
        {{ showPath ? '隐藏私钥文件路径' : '或使用私钥文件路径' }}
      </button>
      <InputText
        v-if="showPath"
        v-model="form.keyPath"
        placeholder="/Users/you/.ssh/id_ed25519"
      />

      <small>私钥从系统钥匙串读取，只在连接时交给 Rust，不会落到磁盘。</small>
    </div>

    <div v-else class="section">
      <label>密码</label>
      <InputText
        v-model="form.password"
        type="password"
        :placeholder="hasPassword ? '已保存在系统钥匙串，留空则不修改' : '登录密码'"
      />
      <small>密码只写入系统钥匙串，不会出现在 localStorage 里。</small>
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

.link {
  align-self: flex-start;
  padding: 0;
  border: 0;
  background: transparent;
  color: #2563eb;
  font-size: 12px;
  cursor: pointer;
}

.no-keys {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.key-select {
  width: 100%;
}

/* PrimeVue Select 默认比 InputText 高，压到和表单其他输入框一致 */
.key-select :deep(.p-select) {
  width: 100%;
  height: 36px;
}

.key-select :deep(.p-select-label) {
  padding: 0 0.75rem;
  font-size: 14px;
  line-height: 34px;
}

.key-select :deep(.p-select-dropdown) {
  width: 2rem;
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
