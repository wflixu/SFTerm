<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useSystemStore } from '@/stores/system';
import { useTabsStore } from '@/stores/tabs';
import type { IHost } from '@/shared/interface';

const system = useSystemStore();
const tabs = useTabsStore();
const router = useRouter();

const pickerOpen = ref(false);

const pick = (host: IHost) => {
  tabs.openSession(host);
  router.push(`/session/${host.id}`);
  pickerOpen.value = false;
};
</script>

<template>
  <div class="sftp">
    <header class="pane-head">
      <div class="name"><i class="pi pi-folder" /> Local</div>
      <div class="hint">本地文件浏览待接入</div>
    </header>

    <div class="local-empty">
      <i class="pi pi-desktop" />
      <span>本地文件面板</span>
    </div>

    <div class="divider" />

    <section class="remote">
      <header class="pane-head">
        <div class="name"><i class="pi pi-cloud" /> 远端</div>
      </header>

      <div class="connect">
        <div class="icon"><i class="pi pi-folder" /></div>
        <h3>连接主机</h3>
        <p>从一台已保存的主机开始，用 SFTP 管理文件</p>
        <Button label="选择主机" severity="secondary" @click="pickerOpen = !pickerOpen" />

        <div v-if="pickerOpen" class="picker">
          <div v-if="system.hosts.length" class="picker-list">
            <button v-for="host in system.hosts" :key="host.id" class="picker-item" @click="pick(host)">
              <i class="pi pi-server" />
              <span class="pick-name">{{ host.name || host.address }}</span>
              <span class="pick-addr">{{ host.address }}</span>
            </button>
          </div>
          <div v-else class="picker-empty">还没有主机，先去 Home 添加。</div>
          <div class="picker-foot">
            <button class="link" @click="router.push('/'); pickerOpen = false">前往 Hosts</button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.sftp {
  display: flex;
  height: 100%;
  background: #fff;
}

.pane-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 14px;
  border-bottom: 1px solid #e9ecf0;
}

.name {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 13px;
  font-weight: 600;
  color: #111827;
}

.name i {
  font-size: 12px;
  color: #6b7280;
}

.hint,
.picker-empty {
  font-size: 12px;
  color: #9ca3af;
}

.local-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: #b6bcc5;
  font-size: 13px;
}

.local-empty i {
  font-size: 28px;
}

.divider {
  width: 1px;
  background: #e3e6ea;
}

.remote {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
}

.connect {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.connect .icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  margin-bottom: 4px;
  border-radius: 16px;
  background: #eef1f5;
  color: #8b93a1;
  font-size: 26px;
}

.connect h3 {
  margin: 0;
  font-size: 16px;
  color: #111827;
}

.connect p {
  margin: 0 0 8px;
  max-width: 280px;
  text-align: center;
  font-size: 13px;
  color: #6b7280;
}

.picker {
  position: absolute;
  top: 50%;
  margin-top: 90px;
  width: 300px;
  padding: 6px;
  border: 1px solid #e3e6ea;
  border-radius: 10px;
  background: #fff;
  box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12);
  z-index: 5;
}

.picker-list {
  max-height: 220px;
  overflow-y: auto;
}

.picker-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.picker-item:hover {
  background: #f3f5f8;
}

.picker-item i {
  font-size: 12px;
  color: #6b7280;
}

.pick-name {
  font-size: 13px;
  color: #111827;
}

.pick-addr {
  margin-left: auto;
  font-size: 11px;
  color: #9ca3af;
}

.picker-empty {
  padding: 14px 10px;
}

.picker-foot {
  border-top: 1px solid #f0f2f5;
  padding: 6px;
}

.link {
  border: 0;
  background: transparent;
  color: #2f6feb;
  font-size: 12px;
  cursor: pointer;
  padding: 4px 6px;
}
</style>
