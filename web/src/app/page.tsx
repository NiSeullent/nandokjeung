'use client';

import { useCallback, useEffect, useState } from 'react';
import { loadObfuscatedHandler, runCircularRelay, obfuscateFortress } from '@/lib/relay-client';

export default function HomePage() {
  const [input, setInput] = useState('비밀 메시지를 입력하세요');
  const [obfuscated, setObfuscated] = useState('');
  const [status, setStatus] = useState('핸들러 로딩 중…');
  const [handlerJp, setHandlerJp] = useState('');
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    loadObfuscatedHandler()
      .then((jp) => {
        setHandlerJp(jp);
        setStatus('난독화된 클라이언트 핸들러 로드 완료 (handler.jp)');
      })
      .catch((e) => setStatus(`핸들러 로드 실패: ${e}`));
  }, []);

  const onObfuscate = useCallback(async () => {
    setBusy(true);
    setStatus('Fortress 난독화 중…');
    try {
      const res = await obfuscateFortress(input);
      setObfuscated(res.payload);
      setStatus(`난독화 완료 (nonce: ${res.nonce.slice(0, 8)}…)`);
    } catch (e) {
      setStatus(`오류: ${e}`);
    } finally {
      setBusy(false);
    }
  }, [input]);

  const onRelay = useCallback(async () => {
    if (!handlerJp) return;
    setBusy(true);
    setStatus('순환 해독 프로토콜 시작 (round 0→2)…');
    try {
      const result = await runCircularRelay(input, handlerJp);
      setStatus(
        `순환 완료 · session=${result.session_id.slice(0, 8)}… round=${result.round} done=${String(result.done)}`,
      );
      if (result.payload) {
        setObfuscated(result.payload.slice(0, 2000) + (result.payload.length > 2000 ? '…' : ''));
      }
    } catch (e) {
      setStatus(`릴레이 오류: ${e}`);
    } finally {
      setBusy(false);
    }
  }, [input, handlerJp]);

  return (
    <main>
      <h1>난독증</h1>
      <p className="sub">
        정신뼝! Fortress 난독화 · 프론트는 <span className="badge">handler.jp</span>로 난독화됨 ·
        서버에서만 해독
      </p>

      <div className="panel">
        <label htmlFor="input">평문 입력</label>
        <textarea id="input" value={input} onChange={(e) => setInput(e.target.value)} />
        <button type="button" onClick={onObfuscate} disabled={busy}>
          Fortress 난독화
        </button>
        <button type="button" className="secondary" onClick={onRelay} disabled={busy || !handlerJp}>
          순환 해독 (relay)
        </button>
        <p className="status">{status}</p>
      </div>

      <div className="panel">
        <label>난독화 출력 (정신뼝! — 해독 불가)</label>
        <pre className="output">{obfuscated || '(아직 없음)'}</pre>
      </div>

      <div className="panel">
        <label>클라이언트 핸들러 (난독화됨)</label>
        <pre className="output">{handlerJp ? `${handlerJp.slice(0, 480)}…` : '로딩…'}</pre>
      </div>
    </main>
  );
}
