<script setup lang="ts">
import { computed, ref } from 'vue';
import { useSystemStore } from '@/stores/system';
import type { IHost } from '@/shared/interface';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{
    'update:visible': [value: boolean];
    select: [host: IHost];
}>();

const system = useSystemStore();
const search = ref('');

const filtered = computed(() => {
    const q = search.value.trim().toLowerCase();
    if (!q) return system.hosts;
    return system.hosts.filter((host) =>
        [host.name, host.address, host.username ?? ''].some((value) =>
            value.toLowerCase().includes(q),
        ),
    );
});

const choose = (host: IHost) => {
    emit('select', host);
    emit('update:visible', false);
    search.value = '';
};
</script>

<template>
    <Dialog
        :visible="props.visible"
        header="选择主机"
        :style="{ width: '440px' }"
        modal
        @update:visible="emit('update:visible', $event)"
    >
        <div class="picker">
            <InputText v-model="search" placeholder="搜索主机" class="search" />
            <div v-if="filtered.length" class="list">
                <button v-for="host in filtered" :key="host.id" class="row" @click="choose(host)">
                    <i class="pi pi-server" />
                    <span class="name">{{ host.name || host.address }}</span>
                    <span class="addr">{{ host.username || 'root' }}@{{ host.address }}</span>
                </button>
            </div>
            <div v-else class="empty">没有匹配的主机，先去 Home 添加。</div>
        </div>
    </Dialog>
</template>

<style scoped>
.picker {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.search {
    width: 100%;
}

.list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 320px;
    overflow-y: auto;
}

.row {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px 10px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    text-align: left;
    cursor: pointer;
}

.row:hover {
    background: #f3f5f8;
}

.row i {
    font-size: 12px;
    color: #6b7280;
}

.name {
    font-size: 13px;
    color: #111827;
}

.addr {
    margin-left: auto;
    font-size: 11px;
    color: #9ca3af;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.empty {
    padding: 18px 8px;
    text-align: center;
    font-size: 12px;
    color: #9ca3af;
}
</style>
