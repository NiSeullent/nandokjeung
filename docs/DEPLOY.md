# 배포 (Deploy)

## 브랜치 전략

| 브랜치 | 역할 |
|--------|------|
| `main` | 개발·PR 머지 대상 (소스) |
| `deploy` | CI가 자동 갱신하는 **정적 웹 산출물** (GitHub Pages 소스 브랜치) |

`main`에 push되면 GitHub Actions가 Next.js 정적 export를 빌드하고 `deploy` 브랜치에 force-push합니다.  
수동 배포: Actions 탭 → **Deploy** → **Run workflow**.

## GitHub Pages 설정

1. Repository **Settings → Pages**
2. **Build and deployment** → Source: **Deploy from a branch**
3. Branch: `deploy` / folder: `/ (root)`
4. (선택) **Settings → Secrets and variables → Actions → Variables**
   - `NANDOKJEUNG_API_URL` — 정적 사이트가 호출할 API 베이스 URL (예: `https://your-api.example.com`)

정적 호스팅에서는 Next.js `rewrites`가 동작하지 않으므로, 브라우저는 `NEXT_PUBLIC_API_URL`로 API를 직접 호출합니다.

## API 서버 (Rust)

웹 UI의 obfuscate/relay는 별도 API 프로세스가 필요합니다.

```bash
export NANDOKJEUNG_SECRET="production-secret"
cargo run --release -- api --port 8787
```

프로덕션에서는 이 바이너리를 VM/컨테이너에 배포하고, Pages 변수 `NANDOKJEUNG_API_URL`을 해당 공개 URL로 맞춥니다.

## 릴리스 바이너리

`v*` 태그 push 시 **Release** 워크플로가 Linux / Windows / macOS 바이너리를 GitHub Releases에 업로드합니다.

```bash
git tag v1.0.1
git push origin v1.0.1
```

## 로컬에서 deploy 산출물 확인

```bash
cd web
NEXT_OUTPUT=export NEXT_PUBLIC_API_URL=http://127.0.0.1:8787 npm run build
npx serve out
```

## CI 요약

| 워크플로 | 트리거 | 내용 |
|----------|--------|------|
| CI | `main` push / PR | `cargo fmt`, `clippy`, `test`, `web` build |
| Deploy | `main` push / manual | `web/out` → `deploy` 브랜치 |
| Release | `v*` tag | 크로스 플랫폼 `nandokjeung` 바이너리 |
