<script setup lang="ts">
import type { IHost } from '@/shared/interface';

defineProps<{ host: IHost }>();

const emit = defineEmits<{
  connect: [host: IHost];
  edit: [host: IHost];
  delete: [host: IHost];
}>();

const addressLine = (host: IHost) => {
  const user = host.username ? `${host.username}@` : '';
  const port = host.port && host.port !== 22 ? `:${host.port}` : '';
  return `${user}${host.address}${port}`;
};
</script>

<template>
  <div class="card" @dblclick="emit('connect', host)">
    <div class="icon"><i class="pi pi-server" /></div>
    <div class="info">
      <div class="name">{{ host.name || '未命名主机' }}</div>
      <div class="addr">{{ addressLine(host) }}</div>
      <div v-if="host.desc" class="desc">{{ host.desc }}</div>
    </div>
    <div class="actions">
      <button class="icon-btn" title="编辑" @click.stop="emit('edit', host)">
        <i class="pi pi-pen-to-square" />
      </button>
      <button class="icon-btn danger" title="删除" @click.stop="emit('delete', host)">
        <i class="pi pi-trash" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px;
  border: 1px solid #e6e9ed;
  border-radius: 10px;
  background: #fff;
  transition: border-color 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease;
  user-select: none;
}

.card:hover {
  border-color: #cfd6de;
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.06);
  transform: translateY(-1px);
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  border-radius: 8px;
  background: #f1f4f8;
  color: #4b5563;
  font-size: 15px;
}

.info {
  flex: 1;
  min-width: 0;
}

.name {
  font-size: 13px;
  font-weight: 600;
  color: #111827;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.addr {
  margin-top: 3px;
  font-size: 12px;
  color: #6b7280;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.desc {
  margin-top: 4px;
  font-size: 12px;
  color: #9ca3af;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.card:hover .actions {
  opacity: 1;
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
</style>
