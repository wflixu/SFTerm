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
    sftp.bind(next.id);
};

const requestHost = () => {
    pickerOpen.value = true;
};
</script>

<template>
    <DualPane
        :host="host"
        auto-connect
        can-pick-host
        @request-host="requestHost"
    />
    <HostPicker v-model:visible="pickerOpen" @select="choose" />
</template>
