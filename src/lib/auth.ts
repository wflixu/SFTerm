import type { IHost } from '@/shared/interface';
import { useKeychainStore } from '@/stores/keychain';
import { keyPassphraseAccount, keyPrivateAccount, resolveHostPassword, secretGet } from './secrets';

/** 与 Rust 侧 `AuthMethod` 的 serde 表示一致（snake_case）。 */
export type HostAuth =
    | { kind: 'password'; password: string }
    | { kind: 'key'; path: string }
    | { kind: 'key_memory'; private_key: string; public_key?: string };

/**
 * 把主机配置解析成一次连接用的认证参数。
 *
 * 优先级：Keychain 条目（keyId）> 私钥文件路径 > 钥匙串里的密码。
 * 钥匙串条目里的私钥只在这一刻读出来，交给 Rust 用内存认证，不落盘。
 */
export async function resolveHostAuth(host: IHost): Promise<HostAuth> {
    if (host.keyId) {
        const entry = useKeychainStore().entries.find((item) => item.id === host.keyId);
        // 迁移失败时条目里还留着明文私钥，作为兜底。
        const privateKey =
            (await secretGet(keyPrivateAccount(host.keyId))) ?? entry?.privateKey;
        if (!privateKey) {
            throw new Error('这台主机引用的密钥已不在系统钥匙串里，请到 Keychain 重新选择。');
        }
        const publicKey = entry?.publicKey?.trim();
        const passphrase = entry?.hasPassphrase
            ? ((await secretGet(keyPassphraseAccount(host.keyId))) ?? undefined)
            : undefined;
        return {
            kind: 'key_memory',
            private_key: privateKey,
            ...(publicKey ? { public_key: publicKey } : {}),
            ...(passphrase ? { passphrase } : {}),
        };
    }

    if (host.keyPath) {
        return { kind: 'key', path: host.keyPath };
    }

    const password = await resolveHostPassword(host);
    if (!password) {
        throw new Error(
            host.hasPassword
                ? '系统钥匙串里找不到这台主机的密码，回 Home 重新填一下。'
                : '这台主机还没有配置认证方式，回 Home 补一下。',
        );
    }
    return { kind: 'password', password };
}
