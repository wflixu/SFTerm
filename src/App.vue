<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useTabsStore } from '@/stores/tabs';

const route = useRoute();
const router = useRouter();
const tabs = useTabsStore();

const isHome = computed(() => route.path === '/');
const isSftp = computed(() => route.path.startsWith('/sftp'));
const activeHostId = computed(() => (route.params.hostId as string) ?? '');

const open = (hostId: string) => router.push(`/session/${hostId}`);

const close = (id: string) => {
  tabs.closeSession(id);
  if (route.path === `/session/${id}`) router.push('/');
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
        :class="{ active: activeHostId === tab.hostId }"
        @click="open(tab.hostId)"
      >
        <i class="pi pi-server" />
        <span class="label">{{ tab.title }}</span>
        <i class="pi pi-times close" @click.stop="close(tab.id)" />
      </button>
    </header>
    <main class="content">
      <RouterView />
    </main>
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
</style>
