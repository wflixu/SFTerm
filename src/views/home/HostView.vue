<template>
    <div class="hosts">
        <div class="body">
            <div class="searchbox">
                <InputText v-model="searchQuery" placeholder="搜索主机..." />
            </div>

            <div class="actions">
                <Button icon="pi pi-plus" label="添加" severity="success" @click="addHost" />
            </div>
            <div class="host-cards">
                <HostCard v-for="host in systemStore.hosts" :key="host.id" :host="host" @select="onHostClick"
                    @delete="onDelete" />
            </div>
        </div>

        <div class="host-edit" v-if="addVisible && curHost">
            <div class="actions">
                <div class="left"></div>
                <div class="right">
                    <i class="pi pi-arrow-right" @click="addVisible = false"></i>
                </div>
            </div>
            <HostEdit :host="curHost" @submit="saveHost" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import HostCard from './components/HostCard.vue';
import HostEdit from './components/HostEdit.vue';
import type { IHost } from '@/shared/interface';
import { useSystemStore } from '@/stores/system';

const systemStore = useSystemStore();
const addVisible = ref(false);
const curHost = ref<IHost | null>(null);

// 新主机数据
const newHost = ref({
    address: '',
    name: '',
    parentGroup: '',
    tags: '',
    description: 'ssh'
});

const addHost = () => {
    addVisible.value = true;
    curHost.value = {
        address: '',
        name: '',
        id: '',
        desc: 'ssh'
    } as IHost;
};

// 保存主机
const saveHost = (host: IHost) => {

    systemStore.addHost(host);
    // 关闭表单
    addVisible.value = false;
};

// 模拟数据
const hosts = ref<IHost[]>([
    // {
    //     id: '1',
    //     name: 'light',
    //     address: '192.168.1.1',
    //     description: 'ssh, ubuntu',
    //     port: 22,
    //     password: 'password123',
    //     keyPath: '/path/to/key'
    // },
    // {
    //     id: '2',
    //     name: 'vps-tencent',
    //     address: '192.168.1.2',
    //     description: 'ssh, ubuntu',
    //     port: 22,
    //     password: 'password456',
    //     keyPath: '/path/to/key'
    // },
    // {
    //     id: '3',
    //     name: 'vpn-server-admin',
    //     address: '192.168.1.3',
    //     description: 'ssh, vpn-server-admin',
    //     port: 22,
    //     password: 'password789',
    //     keyPath: '/path/to/key'
    // },
    // {
    //     id: '4',
    //     name: 'vps-tencent-root',
    //     address: '192.168.1.4',
    //     description: 'ssh, root',
    //     port: 22,
    //     password: 'password000',
    //     keyPath: '/path/to/key'
    // },
    // {
    //     id: '5',
    //     name: 'vps-us',
    //     address: '192.168.1.5',
    //     description: 'ssh, root',
    //     port: 22,
    //     password: 'password111',
    //     keyPath: '/path/to/key'
    // }
]);

const searchQuery = ref('');
const onDelete = (host: any) => {
    hosts.value = hosts.value.filter(h => h.id !== host.id);
}
// 过滤主机列表
const filteredHosts = computed(() => {
    if (!searchQuery.value) return hosts.value;
    const query = searchQuery.value.toLowerCase();
    console.warn(systemStore.hosts)
    return systemStore.hosts.value.filter(host =>
        host.name.toLowerCase().includes(query) ||
        host.desc.toLowerCase().includes(query)
    );
});

// 处理主机点击事件
const onHostClick = (host: any) => {
    console.log('Host clicked:', host);
    addVisible.value = true;
    curHost.value = host;
    // 这里可以添加导航到连接页面或打开连接对话框的逻辑
};
</script>

<style scoped>
.hosts {
    height: 100%;
    background-color: #EEF1F2;
    display: flex;

    .body {
        flex: 1;
    }
}

.searchbox {
    background-color: white;
    border-bottom: 1px solid #dee2e6;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.searchbox :deep(.p-inputtext) {
    width: 100%;
    max-width: 400px;
}

.actions {
    padding: 16px 24px;
    display: flex;
    justify-content: flex-end;
    align-items: center;
}

.host-cards {
    padding: 16px 24px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
    overflow-y: auto;
    flex: 1;
}

.host-edit {
    width: 360px;
    border-left: 1px solid #dee2e6;

    .actions {
        height: 48px;
        background-color: white;
        border-bottom: 1px solid #dee2e6;
    }
}
</style>