import { useKeychainStore } from '@/stores/keychain';
import { useSystemStore } from '@/stores/system';
import { keyTypeOf } from './keytype';
import { hasKeychain, hostPasswordAccount, keyPrivateAccount, secretSet } from './secrets';

const FLAG = 'secrets:v1';

/**
 * 把旧版 localStorage 里的明文密码 / 私钥搬进系统钥匙串，然后从 hosts/keys 里抹掉。
 *
 * - 只有全部迁移成功才写标记，否则下次启动继续尝试；
 * - 迁移失败的主机保留原明文，连接流程会回退使用它，不会把人锁在门外；
 * - 在浏览器里没有 Tauri IPC，直接跳过。
 */
export async function migrateSecrets(): Promise<void> {
    if (localStorage.getItem(FLAG) === '1' || !hasKeychain()) return;

    const system = useSystemStore();
    const keychain = useKeychainStore();
    let failed = 0;

    for (const host of [...system.hosts]) {
        if (!host.password) continue;
        try {
            await secretSet(hostPasswordAccount(host.id), host.password);
            system.updateHost({ ...host, password: undefined, hasPassword: true });
        } catch (error) {
            failed += 1;
            console.error('[secrets] 主机密码迁移失败', host.id, error);
        }
    }

    for (const entry of [...keychain.entries]) {
        if (!entry.privateKey) continue;
        try {
            await secretSet(keyPrivateAccount(entry.id), entry.privateKey);
            keychain.update({
                ...entry,
                privateKey: undefined,
                type: entry.type ?? keyTypeOf(entry.privateKey),
            });
        } catch (error) {
            failed += 1;
            console.error('[secrets] 私钥迁移失败', entry.id, error);
        }
    }

    if (failed === 0) localStorage.setItem(FLAG, '1');
}
