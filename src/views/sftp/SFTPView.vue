<script setup lang="ts">
import { ref } from 'vue';
import DualPane from '@/components/DualPane.vue';
import HostPicker from '@/components/HostPicker.vue';
import { useSftpStore } from '@/stores/sftp';
import type { IHost } from '@/shared/interface';

const sftp = useSftpStore();
const pickerOpen = ref(false);

const choose = (next: IHost) => {
    console.debug('[sftp] choose host', next);
    sftp.bind(next);
};

const requestHost = () => {
    pickerOpen.value = true;
};
</script>

<template>
    <DualPane
        :key="sftp.bound?.id ?? sftp.bound?.address ?? 'unbound'"
        :host="sftp.bound"
        :auto-connect="true"
        can-pick-host
        @request-host="requestHost"
    />
    <HostPicker v-model:visible="pickerOpen" @select="choose" />
</template>
