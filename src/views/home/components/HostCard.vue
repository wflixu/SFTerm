<template>
    <div class="host-card" @click="onClick"  @contextmenu="onRightClick">
        <div class="icon">
            <i class="pi pi-server"></i>
        </div>
        <div class="info">
            <div class="name">{{ host.name }}</div>
            <div class="description">{{ host.description }}</div>
        </div>
        <div class="actions">
            <i class="pi pi-trash" @click.stop="onDelete"></i>
            <i class="pi pi-pen-to-square" @click.stop="onEdit"></i>
        </div>
    </div>
    <ContextMenu ref="menuRef" :model="menuItems" />
</template>

<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
    host: {
        name: string;
        description: string;
        [key: string]: any;
    }
}>();

const emit = defineEmits<{
    (e: 'select', host: any): void;
    (e: 'edit', host: any): void;
    (e: 'delete', host: any): void;
}>();

const menuRef = ref<any>(null);
const menuItems = [
    {
        label: 'Connect',
        icon: 'pi pi-link',
        command: () => onConnect()
    },
    {
        label: 'Delete',
        icon: 'pi pi-trash',
        command: () => onDelete()
    }
];

const onRightClick = (e: MouseEvent) => {
    e.preventDefault();
    menuRef.value.show(e);
};

const onClick = () => {
    emit('select', props.host);
};
const onEdit = () => {
    emit('edit', props.host);
}
const onDelete = () => {
    emit('delete', props.host);
}
const onConnect = () => {
    console.log('connect', props.host);
}
</script>

<style scoped>
.host-card {
    display: flex;
    align-items: center;
    padding: 16px;
    border-radius: 8px;
    background-color: #fff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    cursor: pointer;
    transition: all 0.2s ease;
}

.host-card:hover {
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 8px;
    background-color: #f5f5f5;
    margin-right: 16px;
}

.icon i {
    font-size: 20px;
    color: #e76f51;
}

.info {
    flex: 1;
}

.actions {
    display: flex;
    align-items: center;
    gap: 8px;
    i:hover {
        color:#66C464;
    }
}

.name {
    font-weight: 600;
    font-size: 16px;
    margin-bottom: 4px;
}

.description {
    font-size: 14px;
    color: #666;
}
</style>