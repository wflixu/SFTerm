import { useSystemStore } from '@/stores/system';
import { useKeychainStore } from '@/stores/keychain';
import {
    hasKeychain,
    hostPasswordAccount,
    keyPassphraseAccount,
    secretDelete,
} from './secrets';

/**
 * 清理和当前数据对不上的钥匙串条目：
 * - 主机已经不再使用密码（hasPassword=false），却还留着密码；
 * - 密钥条目已经不再有口令，却还留着口令。
 *
 * 只处理「当前还存在的实体」，已删除实体的孤儿条目需要枚举钥匙串才能发现，
 * 目前的策略是在删除时同步清理。
 */
export async function cleanupSecrets(): Promise<void> {
    if (!hasKeychain()) return;

    const system = useSystemStore();
    const keychain = useKeychainStore();

    for (const host of system.hosts) {
        if (host.hasPassword) continue;
        await secretDelete(hostPasswordAccount(host.id)).catch((error) =>
            console.error('[secrets] 清理主机密码失败', host.id, error),
        );
    }

    for (const entry of keychain.entries) {
        if (entry.hasPassphrase) continue;
        await secretDelete(keyPassphraseAccount(entry.id)).catch((error) =>
            console.error('[secrets] 清理密钥口令失败', entry.id, error),
        );
    }
}
