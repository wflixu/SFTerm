import { beforeEach, describe, expect, it, vi } from 'vitest';

// jsdom 在这个 vitest 版本里没有提供 localStorage，用一个内存实现顶上。
const memory = new Map<string, string>();
Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
        getItem: (key: string) => memory.get(key) ?? null,
        setItem: (key: string, value: string) => memory.set(key, String(value)),
        removeItem: (key: string) => memory.delete(key),
        clear: () => memory.clear(),
    },
});
import { mount, flushPromises } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import SFTPView from '../SFTPView.vue';
import { useSystemStore } from '@/stores/system';
import { useSftpStore } from '@/stores/sftp';

const { invoke } = vi.hoisted(() => ({
    invoke: vi.fn(async (command: string) => {
        switch (command) {
            case 'fs_home':
                return '/Users/test';
            case 'fs_list_local':
            case 'sftp_list_dir':
                return [];
            case 'ssh_connect':
                return {
                    session_id: 's-1',
                    banner: 'SSH-2.0-Test',
                    host_key_type: 'Ed25519',
                    fingerprint_sha256: 'abc',
                    kex: 'curve25519-sha256',
                    cipher: 'chacha20-poly1305',
                };
            case 'ssh_disconnect':
                return true;
            default:
                return null;
        }
    }),
}));

vi.mock('@tauri-apps/api/core', () => ({ invoke }));

const stubs = {
    Button: {
        props: ['label'],
        emits: ['click'],
        template: '<button class="btn" @click="$emit(\'click\')">{{ label }}<slot /></button>',
    },
    InputText: {
        props: ['modelValue'],
        emits: ['update:modelValue'],
        template:
            '<input :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)" />',
    },
    Dialog: {
        props: ['visible', 'header'],
        emits: ['update:visible'],
        template: '<div v-if="visible" class="dialog"><slot /><slot name="footer" /></div>',
    },
};

describe('SFTPView', () => {
    beforeEach(() => {
        setActivePinia(createPinia());
        invoke.mockClear();
        window.localStorage.clear();
    });

    it('binds the chosen host in place and connects', async () => {
        const system = useSystemStore();
        system.addHost({
            id: 'h1',
            name: 'web-1',
            address: '10.0.0.1',
            username: 'ubuntu',
            keyPath: '/tmp/key',
        });

        const sftp = useSftpStore();
        const wrapper = mount(SFTPView, {
            global: { plugins: [], stubs },
        });
        await flushPromises();

        expect(wrapper.text()).toContain('连接主机');

        const pickButton = wrapper.findAll('button').find((b) => b.text().includes('选择主机'));
        expect(pickButton, '“选择主机”按钮应存在').toBeTruthy();
        await pickButton!.trigger('click');

        const hostRow = wrapper.findAll('.row').find((r) => r.text().includes('web-1'));
        expect(hostRow, '弹窗里应列出主机').toBeTruthy();
        await hostRow!.trigger('click');
        await flushPromises();

        expect(sftp.bound?.id).toBe('h1');
        expect(wrapper.text()).toContain('web-1');
        expect(invoke).toHaveBeenCalledWith('ssh_connect', expect.anything());
    });
});
