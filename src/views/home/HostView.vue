<template>
    <div class="hosts">
        <div class="body">
            <div class="searchbox">
                <InputText v-model="searchQuery" placeholder="搜索主机..." />
            </div>

            <div class="actions">
                <Button icon="pi pi-plus" label="添加" severity="success" @click="addVisible = true" />
            </div>
            <div class="host-cards">
                <HostCard v-for="host in filteredHosts" :key="host.id" :host="host" @select="onHostClick"
                    @delete="onDelete" />
            </div>
        </div>

        <div class="host-edit" v-if="addVisible">
            <div class="actions">
                <div class="left"></div>
                <div class="right">
                    <i class="pi pi-arrow-right" @click="addVisible = false"></i>
                </div>
            </div>
            <div class="edit-container">
                <div class="edit-section">
                    <h3>Address</h3>
                    <div class="address-input">
                        <div class="address-icon">
                            <i class="pi pi-server"></i>
                        </div>
                        <InputText v-model="newHost.address" placeholder="IP or Hostname" class="address-field" />
                    </div>
                </div>

                <div class="edit-section">
                    <h3>General</h3>
                    <div class="form-field">
                        <InputText v-model="newHost.name" placeholder="Label" class="full-width" />
                    </div>
                    <div class="form-field">
                        <div class="input-with-icon">
                            <i class="pi pi-folder"></i>
                            <InputText v-model="newHost.parentGroup" placeholder="Parent Group" class="full-width" />
                        </div>
                    </div>
                    <div class="form-field">
                        <div class="input-with-icon">
                            <i class="pi pi-tag"></i>
                            <InputText v-model="newHost.tags" placeholder="Tags" class="full-width" />
                        </div>
                    </div>
                    <div class="form-field">
                        <div class="input-with-icon space-between">
                            <div>
                                <i class="pi pi-arrow-left"></i>
                                <span>Backspace</span>
                            </div>
                            <span class="default-label">Default</span>
                        </div>
                    </div>
                </div>

                <div class="form-actions">
                    <Button label="Cancel" class="p-button-text" @click="addVisible = false" />
                    <Button label="Save" severity="success" @click="saveHost" />
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import HostCard from './components/HostCard.vue';
import type { IHost } from '@/shared/interface';
import { useSystemStore } from '@/stores/system';

const systemStore = useSystemStore();
const addVisible = ref(false);
const curHost = ref(null);

// 新主机数据
const newHost = ref({
    address: '',
    name: '',
    parentGroup: '',
    tags: '',
    description: 'ssh'
});

// 保存主机
const saveHost = () => {
    // 生成唯一ID
    const id = Date.now().toString();

    // 添加到主机列表
    hosts.value.push({
        id,
        name: newHost.value.name || newHost.value.address,
        description: newHost.value.description,
        icon: 'pi-server'
    });

    // 重置表单
    newHost.value = {
        address: '',
        name: '',
        parentGroup: '',
        tags: '',
        description: 'ssh'
    };

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
    return systemStore.hosts.filter(host =>
        host.name.toLowerCase().includes(query) ||
        host.description.toLowerCase().includes(query)
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

.edit-container {
    padding: 20px;
}

.edit-section {
    margin-bottom: 20px;
    background-color: #f8f9fa;
    border-radius: 8px;
    padding: 16px;
}

.edit-section h3 {
    margin-top: 0;
    margin-bottom: 16px;
    font-size: 16px;
    font-weight: 600;
    color: #333;
}

.address-input {
    display: flex;
    align-items: center;
    gap: 12px;
}

.address-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 8px;
    background-color: #2c3e50;
}

.address-icon i {
    font-size: 20px;
    color: white;
}

.address-field {
    flex: 1;
}

.form-field {
    margin-bottom: 12px;
}

.form-field:last-child {
    margin-bottom: 0;
}

.input-with-icon {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    background-color: white;
}

.input-with-icon i {
    color: #6c757d;
    font-size: 16px;
}

.space-between {
    justify-content: space-between;
}

.default-label {
    color: #6c757d;
    font-size: 14px;
}

.full-width {
    width: 100%;
}

.form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 20px;
}
</style>