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

export interface IKeyEntry {
    id: string;
    label: string;
    privateKey: string;
    publicKey: string;
}

export interface IConnection {

}
