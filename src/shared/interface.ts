export interface IHost {
    id: string;
    name: string;
    address: string;
    description: string;
    port?: number;
    password?: string;
    keyPath?: string;
}