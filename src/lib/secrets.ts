import { invoke } from '@tauri-apps/api/core';
import type { IHost } from '@/shared/interface';

/** 主机登录密码在钥匙串里的 account。 */
export const hostPasswordAccount = (id: string) => `host:${id}:password`;
/** 密钥条目私钥文本在钥匙串里的 account。 */
export const keyPrivateAccount = (id: string) => `key:${id}:private`;
/** 密钥条目私钥口令在钥匙串里的 account。 */
export const keyPassphraseAccount = (id: string) => `key:${id}:passphrase`;

/** WebView 开发模式（没有 Tauri IPC）下不能访问钥匙串。 */
export const hasKeychain = () =>
    typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export async function secretSet(account: string, value: string): Promise<void> {
    if (!hasKeychain()) throw new Error('钥匙串只在桌面应用中可用');
    await invoke('secret_set', { account, value });
}

export async function secretGet(account: string): Promise<string | null> {
    if (!hasKeychain()) return null;
    return invoke<string | null>('secret_get', { account });
}

export async function secretDelete(account: string): Promise<void> {
    if (!hasKeychain()) return;
    await invoke('secret_delete', { account });
}

/**
 * 解析主机的登录密码。
 * 新数据从钥匙串读；迁移前的旧数据仍可能带明文 password，作为兜底。
 */
export async function resolveHostPassword(host: IHost): Promise<string | undefined> {
    if (host.hasPassword) return (await secretGet(hostPasswordAccount(host.id))) ?? undefined;
    return host.password;
}
