<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import DualPane from '@/components/DualPane.vue';
import HostPicker from '@/components/HostPicker.vue';
import { useSftpStore } from '@/stores/sftp';
import { useTabsStore } from '@/stores/tabs';
import type { IHost } from '@/shared/interface';

const sftp = useSftpStore();
const tabs = useTabsStore();
const router = useRouter();
const pickerOpen = ref(false);

const openTerminal = () => {
    if (!sftp.bound) return;
    const tab = tabs.openTerminal(sftp.bound);
    router.push(`/terminal/${tab.hostId}`);
};

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
        @open-terminal="openTerminal"
    />
    <HostPicker v-model:visible="pickerOpen" @select="choose" />
</template>
