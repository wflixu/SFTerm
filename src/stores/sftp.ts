import { ref } from 'vue';
import { defineStore } from 'pinia';

/**
 * SFTP 常驻页当前绑定的主机。
 * 只存在内存里：应用重启后回到「未绑定」状态，不自动重连。
 */
export const useSftpStore = defineStore('sftp', () => {
    const hostId = ref<string | null>(null);

    const bind = (id: string | null) => {
        hostId.value = id;
    };

    return { hostId, bind };
});
