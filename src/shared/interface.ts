export interface IHost {
    id: string;
    name: string;
    address: string;
    desc?: string;
    port?: number;
    username?: string;
    /** 密码是否已存系统钥匙串（明文不会写回 localStorage）。 */
    hasPassword?: boolean;
    /** 仅迁移前的旧数据会带明文密码，迁移后为空。 */
    password?: string;
    keyPath?: string;
    keyId?: string;
    group?: string;
}

export interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    is_symlink: boolean;
    size: number;
    perm?: number | null;
    mtime?: number | null;
}

export interface ISnippet {
    id: string;
    label: string;
    command: string;
    group?: string;
    updatedAt?: number;
}

export interface IKeyEntry {
    id: string;
    label: string;
    /** 私钥文本存在系统钥匙串；这里只在编辑时临时持有。 */
    privateKey?: string;
    publicKey: string;
    /** 解析自私钥的算法名，迁移后替代私钥用于列表展示。 */
    type?: string;
    /** 私钥口令是否已存系统钥匙串。 */
    hasPassphrase?: boolean;
}

export interface IConnection {

}
