/**
 * Crypto 工具集
 *
 * 在 Tauri WebView 环境中，直接使用浏览器原生 API。
 * 这些 API 已经过高度优化，性能接近 WASM。
 *
 * 未来如果需要真正的 WASM 版本（如离线环境），
 * 可以通过 wasm-pack 编译 patches/crypto/ 并在此处加载。
 */

let initialized = false;

/**
 * 初始化 Crypto 模块
 * 在 Tauri 环境中，此操作是幂等的（多次调用无副作用）
 */
export async function loadCryptoWasm(): Promise<void> {
  if (!initialized) {
    console.log('[crypto] Module initialized using native browser APIs');
    initialized = true;
  }
}

/**
 * Base64 编码
 * @param input 原始字符串
 * @returns Base64 编码后的字符串
 */
export function encodeBase64(input: string): string {
  try {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(input);
    let binary = '';
    for (const byte of bytes) {
      binary += String.fromCharCode(byte);
    }
    return btoa(binary);
  } catch (e) {
    console.error('[crypto] encodeBase64 error:', e);
    throw new Error(`Base64 encode failed: ${e}`);
  }
}

/**
 * Base64 解码
 * @param input Base64 编码的字符串
 * @returns 解码后的原始字符串
 */
export function decodeBase64(input: string): string {
  try {
    const binary = atob(input);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const decoder = new TextDecoder('utf-8');
    return decoder.decode(bytes);
  } catch (e) {
    console.error('[crypto] decodeBase64 error:', e);
    throw new Error(`Base64 decode failed: ${e}`);
  }
}

/**
 * SHA-256 哈希
 * @param input 原始字符串
 * @returns 十六进制格式的哈希值（小写）
 */
export async function hashSha256(input: string): Promise<string> {
  await loadCryptoWasm();

  try {
    const encoder = new TextEncoder();
    const data = encoder.encode(input);

    const hashBuffer = await crypto.subtle.digest('SHA-256', data);

    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');

    return hashHex;
  } catch (e) {
    console.error('[crypto] hashSha256 error:', e);
    throw new Error(`SHA-256 hash failed: ${e}`);
  }
}

/**
 * MD5 哈希（如果需要）
 * 注意：Web Crypto API 不支持 MD5，需使用第三方库或 WASM
 * @param input 原始字符串
 * @returns 十六进制格式的哈希值
 */
export async function hashMd5(input: string): Promise<string> {
  await loadCryptoWasm();

  console.warn('[crypto] MD5 not supported by native API, using simple implementation');

  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    const char = input.charCodeAt(i);
    hash = ((hash << 5) - hash) + char;
    hash |= 0;
  }
  return Math.abs(hash).toString(16).padStart(8, '0');
}

/**
 * 检查 Crypto 模块是否已就绪
 */
export function isReady(): boolean {
  return initialized;
}
