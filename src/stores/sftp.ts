import { ref } from 'vue';
import { defineStore } from 'pinia';
import type { IHost } from '@/shared/interface';

/**
 * SFTP 常驻页当前绑定的主机。
 *
 * 直接持有主机对象，而不是 id：上一版用 id 回查 hosts，
 * 遇到旧数据里 id 缺失/不规范时会静默拿到 undefined。
 * 只存在内存里：应用重启后回到「未绑定」状态，不自动重连。
 */
export const useSftpStore = defineStore('sftp', () => {
    const bound = ref<IHost | null>(null);

    const bind = (host: IHost | null) => {
        bound.value = host;
    };

    return { bound, bind };
});
