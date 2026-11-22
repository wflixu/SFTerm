import { computed, ref } from 'vue';
import { defineStore } from 'pinia';

export type TransferDirection = 'upload' | 'download';
export type TransferStatus = 'running' | 'done' | 'error';

export interface TransferTask {
    id: string;
    name: string;
    direction: TransferDirection;
    done: number;
    total: number;
    status: TransferStatus;
    error?: string;
}

export const useTransfersStore = defineStore('transfers', () => {
    const tasks = ref<TransferTask[]>([]);

    const add = (task: TransferTask) => {
        tasks.value.unshift(task);
    };

    const update = (id: string, done: number, total: number) => {
        const task = tasks.value.find((item) => item.id === id);
        if (!task) return;
        task.done = done;
        task.total = total;
    };

    const finish = (id: string) => {
        const task = tasks.value.find((item) => item.id === id);
        if (!task) return;
        task.status = 'done';
        if (task.total > 0) task.done = task.total;
    };

    const fail = (id: string, error: string) => {
        const task = tasks.value.find((item) => item.id === id);
        if (!task) return;
        task.status = 'error';
        task.error = error;
    };

    const clearFinished = () => {
        tasks.value = tasks.value.filter((item) => item.status === 'running');
    };

    const running = computed(() => tasks.value.filter((item) => item.status === 'running'));

    return { tasks, running, add, update, finish, fail, clearFinished };
});
