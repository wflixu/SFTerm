<script setup lang="ts">
import { computed, ref } from 'vue';
import HostView from './HostView.vue';
import KeychainView from './KeychainView.vue';
import SnippetsView from './SnippetsView.vue';
import LogsView from './LogsView.vue';
import { useSystemStore } from '@/stores/system';
import { useKeychainStore } from '@/stores/keychain';

const system = useSystemStore();
const keychain = useKeychainStore();

const active = ref('hosts');

const items = computed(() => [
  { id: 'hosts', label: 'Hosts', icon: 'pi-server', count: system.hosts.length },
  { id: 'keychain', label: 'Keychain', icon: 'pi-key', count: keychain.entries.length },
  { id: 'snippets', label: 'Snippets', icon: 'pi-code', count: 0 },
  { id: 'logs', label: 'Logs', icon: 'pi-list', count: 0 },
]);
</script>

<template>
  <div class="home">
    <aside class="sidebar">
      <nav>
        <button
          v-for="item in items"
          :key="item.id"
          class="nav-item"
          :class="{ active: active === item.id }"
          @click="active = item.id"
        >
          <i class="pi" :class="item.icon" />
          <span class="label">{{ item.label }}</span>
          <span v-if="item.count" class="count">{{ item.count }}</span>
        </button>
      </nav>
    </aside>

    <section class="panel">
      <HostView v-if="active === 'hosts'" />
      <KeychainView v-else-if="active === 'keychain'" />
      <SnippetsView v-else-if="active === 'snippets'" />
      <LogsView v-else-if="active === 'logs'" />
    </section>
  </div>
</template>

<style scoped>
.home {
  display: flex;
  height: 100%;
  background: #f7f8fa;
}

.sidebar {
  width: 208px;
  flex-shrink: 0;
  padding: 10px 8px;
  border-right: 1px solid #e3e6ea;
  background: #f0f2f5;
}

.sidebar nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 32px;
  padding: 0 10px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: #374151;
  font-size: 13px;
  text-align: left;
  cursor: default;
}

.nav-item i {
  width: 14px;
  font-size: 13px;
  color: #6b7280;
}

.nav-item:hover {
  background: rgba(0, 0, 0, 0.04);
}

.nav-item.active {
  background: #dfe3e8;
  color: #111827;
}

.nav-item.active i {
  color: #111827;
}

.nav-item .label {
  flex: 1;
}

.nav-item .count {
  font-size: 11px;
  color: #6b7280;
}

.panel {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}
</style>
