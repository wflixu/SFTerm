<script setup lang="ts">
import { computed, ref } from 'vue';
import DualPane from '@/components/DualPane.vue';
import HostPicker from '@/components/HostPicker.vue';
import { useSystemStore } from '@/stores/system';
import { useSftpStore } from '@/stores/sftp';
import type { IHost } from '@/shared/interface';

const system = useSystemStore();
const sftp = useSftpStore();

const pickerOpen = ref(false);

const host = computed(() => (sftp.hostId ? system.getHost(sftp.hostId) ?? null : null));

const choose = (next: IHost) => {
    console.debug('[sftp] choose host', next.id, next.name);
    sftp.bind(next.id);
};

const requestHost = () => {
    pickerOpen.value = true;
};
</script>

<template>
    <DualPane
        :key="host?.id ?? 'unbound'"
        :host="host"
        :auto-connect="true"
        can-pick-host
        @request-host="requestHost"
    />
    <HostPicker v-model:visible="pickerOpen" @select="choose" />
</template>
