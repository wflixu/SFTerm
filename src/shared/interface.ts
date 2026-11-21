export interface IHost {
    id: string;
    name: string;
    address: string;
    desc?: string;
    port?: number;
    username?: string;
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

export interface IKeyEntry {
    id: string;
    label: string;
    privateKey: string;
    publicKey: string;
}

export interface IConnection {

}
