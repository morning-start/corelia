export async function loadCryptoWasm(): Promise<void> {
  console.log('WASM module ready (using native browser APIs)');
}

export function encodeBase64(input: string): string {
  return btoa(input);
}

export function decodeBase64(input: string): string {
  return atob(input);
}

export async function hashSha256(input: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(input);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}
