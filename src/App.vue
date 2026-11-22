<script setup lang="ts">
import { computed, onErrorCaptured, onMounted, onUnmounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useTabsStore } from '@/stores/tabs';
import { useTransfersStore } from '@/stores/transfers';
import { useTerminalsStore } from '@/stores/terminals';
import { migrateSecrets } from '@/lib/migrateSecrets';

const route = useRoute();
const router = useRouter();
const tabs = useTabsStore();
const transfers = useTransfersStore();
const terminals = useTerminalsStore();
const renderError = ref('');

onErrorCaptured((error) => {
  renderError.value = String(error);
  console.error('[app] 页面渲染失败', error);
  return false;
});

let unlistenTransfer: UnlistenFn | null = null;
let unlistenShell: UnlistenFn | null = null;
onMounted(async () => {
  // 先把旧 localStorage 里的明文凭据搬进系统钥匙串，再开始监听事件。
  await migrateSecrets().catch((error) => console.error('[secrets] 迁移失败', error));
  unlistenTransfer = await listen<{ id: string; done: number; total: number }>('transfer-progress', (event) => {
    transfers.update(event.payload.id, event.payload.done, event.payload.total);
  });
  unlistenShell = await listen<{ id: string; data: string }>('shell-output', (event) => {
    terminals.receive(event.payload.id, event.payload.data);
  });
});
onUnmounted(() => {
  unlistenTransfer?.();
  unlistenShell?.();
});

const isHome = computed(() => route.path === '/');
const isSftp = computed(() => route.path.startsWith('/sftp'));
const isTerminal = computed(() => route.path.startsWith('/terminal'));
const activeHostId = computed(() => (route.params.hostId as string) ?? '');

const open = (tab: { hostId: string }) => {
  router.push(`/terminal/${tab.hostId}`);
};

const isActive = (tab: { hostId: string }) =>
  isTerminal.value && activeHostId.value === tab.hostId;

const close = (id: string) => {
  const tab = tabs.sessions.find((item) => item.id === id);
  tabs.closeSession(id);
  terminals.close(id);
  if (tab && isActive(tab)) router.push('/');
};
</script>

<template>
  <div class="app">
    <header class="tabbar">
      <div class="traffic-space" data-tauri-drag-region />
      <button class="tab" :class="{ active: isHome }" @click="router.push('/')">
        <i class="pi pi-home" />
        <span>Home</span>
      </button>
      <button class="tab" :class="{ active: isSftp }" @click="router.push('/sftp')">
        <i class="pi pi-folder" />
        <span>SFTP</span>
      </button>
      <div class="sep" />
      <button
        v-for="tab in tabs.sessions"
        :key="tab.id"
        class="tab session"
        :class="{ active: isActive(tab) }"
        @click="open(tab)"
      >
        <i class="pi pi-desktop" />
        <span class="label">{{ tab.title }}</span>
        <i class="pi pi-times close" @click.stop="close(tab.id)" />
      </button>
    </header>
    <main class="content">
      <div v-if="renderError" class="render-error">
        <h3>页面加载失败</h3>
        <pre>{{ renderError }}</pre>
        <button @click="renderError = ''; $router.go(0)">重新加载</button>
      </div>
      <RouterView v-else />
    </main>
    <ConfirmDialog />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #f5f6f7;
}

.tabbar {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 44px;
  padding: 0 8px;
  background: #e9ebef;
  border-bottom: 1px solid #d9dce1;
  flex-shrink: 0;
}

.traffic-space {
  width: 76px;
  height: 100%;
  flex-shrink: 0;
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 10px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: #4b5563;
  font-size: 13px;
  cursor: default;
  white-space: nowrap;
}

.tab i {
  font-size: 12px;
}

.tab:hover {
  background: rgba(0, 0, 0, 0.05);
}

.tab.active {
  background: #fff;
  color: #111827;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.06);
}

.tab .label {
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab .close {
  margin-left: 4px;
  padding: 2px;
  border-radius: 4px;
  font-size: 9px;
  opacity: 0.5;
}

.tab .close:hover {
  opacity: 1;
  background: rgba(0, 0, 0, 0.08);
}

.sep {
  width: 1px;
  height: 18px;
  margin: 0 4px;
  background: #cfd3d9;
}

.content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.render-error {
  padding: 32px;
  color: #b42318;
}

.render-error h3 {
  margin: 0 0 12px;
  font-size: 15px;
}

.render-error pre {
  white-space: pre-wrap;
  font-size: 12px;
  color: #6b7280;
  background: #fdf2f0;
  padding: 12px;
  border-radius: 8px;
}

.render-error button {
  margin-top: 12px;
  padding: 6px 14px;
  border: 1px solid #d6dae0;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
}
</style>
