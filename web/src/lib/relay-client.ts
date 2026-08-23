const API_BASE = typeof window !== 'undefined'
  ? (process.env.NEXT_PUBLIC_API_URL || '/backend')
  : (process.env.NANDOKJEUNG_API_URL || 'http://127.0.0.1:8787');

export async function loadObfuscatedHandler(): Promise<string> {
  const res = await fetch('/handler.jp', { cache: 'no-store' });
  if (!res.ok) throw new Error('handler.jp not found — run npm run obfuscate');
  return res.text();
}

export async function obfuscateFortress(text: string): Promise<{ payload: string; nonce: string }> {
  const res = await fetch(`${API_BASE}/api/obfuscate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ text, mode: 'fortress' }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error || res.statusText);
  }
  return res.json();
}

export async function relay(
  sessionId: string | null,
  round: number,
  payload: string,
): Promise<{
  session_id: string;
  round: number;
  payload: string;
  done: boolean;
  hint?: string;
}> {
  const res = await fetch(`${API_BASE}/api/relay`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ session_id: sessionId, round, payload }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error || res.statusText);
  }
  return res.json();
}

/** JavaScript ↔ 서버 순환: 난독화 페이로드만 왕복, 해독은 서버 전용 */
export async function runCircularRelay(plainText: string, bootstrapJp: string) {
  let r = await relay(null, 0, bootstrapJp);

  const step1 = await obfuscateFortress(JSON.stringify({ text: plainText }));
  r = await relay(r.session_id, 1, step1.payload);

  const step2 = await obfuscateFortress(JSON.stringify({ confirm: true }));
  r = await relay(r.session_id, 2, step2.payload);

  return r;
}
