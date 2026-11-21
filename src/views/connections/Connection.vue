<script setup lang="ts">
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import DualPane from '@/components/DualPane.vue';
import { useSystemStore } from '@/stores/system';

const route = useRoute();
const router = useRouter();
const system = useSystemStore();

const host = computed(() => system.getHost(route.params.hostId as string) ?? null);
</script>

<template>
    <div v-if="!host" class="missing">
        <h3>主机不存在</h3>
        <p>它可能已被删除。</p>
        <Button label="返回 Home" @click="router.push('/')" />
    </div>

    <DualPane v-else :key="host.id" :host="host" :auto-connect="true" />
</template>

<style scoped>
.missing {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 6px;
    color: #6b7280;
}

.missing h3 {
    margin: 0;
    font-size: 15px;
    color: #111827;
}

.missing p {
    margin: 0 0 10px;
    font-size: 13px;
}
</style>
