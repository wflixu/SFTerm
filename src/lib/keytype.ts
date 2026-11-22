/** 从私钥首行猜算法；钥匙串迁移时把结果单独存下来，避免每次读私钥。 */
export function keyTypeOf(privateKey?: string): string {
    const header = (privateKey ?? '').split('\n')[0] ?? '';
    if (header.includes('OPENSSH')) return 'OpenSSH';
    if (header.includes('RSA')) return 'RSA';
    if (header.includes('EC')) return 'ECDSA';
    return '未知';
}
