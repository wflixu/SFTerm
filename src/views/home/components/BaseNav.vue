<template>
    <div class="base-nav">
        <div class="tab" v-for="item in items" :key="item.id" :class="{ active: modelValue === item.id }"
            @click="onTabClick(item.id)">
            <i class="pi" :class="item.icon"></i>
            <span>{{ item.name }}</span>
        </div>
    </div>
</template>

<script setup lang="ts">

interface IProps {
    items: Array<{ id: string, name: string, icon: string }>
    modelValue: string
    [key: string]: any
}

const props = withDefaults(defineProps<IProps>(), {
    items: () => [
        
    ],
    modelValue: ''
});

const emits = defineEmits<{
    (event: 'update:modelValue', id: string): void;
}>()

const onTabClick = (id: string) => {
    emits('update:modelValue', id);
}
</script>

<style scoped>
.base-nav {
    padding: 8px;

    .tab {
        width: 100%;
        border-radius: 4px;
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 4px 12px;
    }

    .tab.active {
        background-color: #E7EBED;
    }

    .tab:hover {
        cursor: pointer;
    }
}
</style>