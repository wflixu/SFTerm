<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import HostCard from './components/HostCard.vue';
import HostEdit from './components/HostEdit.vue';
import { useSystemStore } from '@/stores/system';
import { useTabsStore } from '@/stores/tabs';
import type { IHost } from '@/shared/interface';

const system = useSystemStore();
const tabs = useTabsStore();
const router = useRouter();

const search = ref('');
const drawerOpen = ref(false);
const editing = ref<IHost | null>(null);

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return system.hosts;
  return system.hosts.filter((host) =>
    [host.name, host.address, host.desc ?? ''].some((value) => value.toLowerCase().includes(q)),
  );
});

const openCreate = () => {
  editing.value = null;
  drawerOpen.value = true;
};

const openEdit = (host: IHost) => {
  editing.value = host;
  drawerOpen.value = true;
};

const save = (host: IHost) => {
  if (system.getHost(host.id)) system.updateHost(host);
  else system.addHost(host);
  drawerOpen.value = false;
};

const remove = (host: IHost) => {
  if (confirm(`删除主机「${host.name || host.address}」？`)) system.removeHost(host);
};

const connect = (host: IHost) => {
  // 理论上 store 加载时已补齐；这里再兜一层，避免空 id 导致路由匹配失败
  const withId = host.id ? host : { ...host, id: `host-${Date.now()}` };
  if (!host.id) system.setHosts(system.hosts.map((item) => (item === host ? withId : item)));

  const tab = tabs.openTerminal(withId);
  router.push(`/terminal/${tab.hostId}`);
};
</script>

<template>
  <div class="hosts">
    <header class="toolbar">
      <div class="title">
        <h2>Hosts</h2>
        <span class="count">{{ system.hosts.length }}</span>
      </div>
      <div class="actions">
        <InputText v-model="search" placeholder="搜索主机" class="search" />
        <Button label="新建主机" icon="pi pi-plus" severity="success" size="small" @click="openCreate" />
      </div>
    </header>

    <div v-if="filtered.length" class="grid">
      <HostCard
        v-for="host in filtered"
        :key="host.id"
        :host="host"
        @connect="connect"
        @edit="openEdit"
        @delete="remove"
      />
    </div>

    <div v-else class="empty">
      <div class="empty-icon"><i class="pi pi-server" /></div>
      <h3>{{ system.hosts.length ? '没有匹配的主机' : '还没有主机' }}</h3>
      <p>{{ system.hosts.length ? '换个关键词试试。' : '添加一台服务器，双击卡片即可连接。' }}</p>
      <Button
        v-if="!system.hosts.length"
        label="添加主机"
        icon="pi pi-plus"
        @click="openCreate"
      />
    </div>

    <transition name="drawer">
      <aside v-if="drawerOpen" class="drawer">
        <header class="drawer-head">
          <h3>{{ editing ? '编辑主机' : '新建主机' }}</h3>
          <button class="icon-btn" @click="drawerOpen = false"><i class="pi pi-times" /></button>
        </header>
        <HostEdit
          :key="editing?.id ?? 'new'"
          :host="editing"
          @save="save"
          @cancel="drawerOpen = false"
        />
      </aside>
    </transition>
  </div>
</template>

<style scoped>
.hosts {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
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

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
  padding: 20px 24px 32px;
  overflow-y: auto;
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
  margin: 0 0 10px;
  font-size: 13px;
}

.drawer {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 380px;
  display: flex;
  flex-direction: column;
  background: #fff;
  border-left: 1px solid #e3e6ea;
  box-shadow: -12px 0 24px rgba(15, 23, 42, 0.08);
  z-index: 10;
}

.drawer-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid #eef0f3;
}

.drawer-head h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
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
  color: #6b7280;
  cursor: pointer;
}

.icon-btn:hover {
  background: #f0f2f5;
  color: #111827;
}

.drawer-enter-active,
.drawer-leave-active {
  transition: transform 0.18s ease, opacity 0.18s ease;
}

.drawer-enter-from,
.drawer-leave-to {
  transform: translateX(16px);
  opacity: 0;
}
</style>
