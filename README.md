# 난독증 (Nandokjeung)

**정신뼝!** 기반 텍스트 난독화 프로그램.

평문을 15개 토큰(`정신` `뼝` `멘헤라` …)만으로 된 `.jp` 소스로 바꿉니다.  
읽을 수 없지만 **실행하면 원문이 나옵니다**.

## 설치

```bash
git clone https://github.com/NiSeullent/nandokjeung.git
cd nandokjeung
cargo build --release
```

바이너리: `target/release/nandokjeung`

## 웹 UI (Next.js)

```bash
# 터미널 1: API 서버 (해독은 서버 전용)
cargo run -- api --port 8787

# 터미널 2: Next.js
cd web && npm install && npm run dev
```

http://localhost:3000

- 빌드 시 `handler.source.json` → **Fortress** `public/handler.jp` (난독화된 클라이언트)
- JS는 난독화 페이로드만 `/api/relay`로 순환 (round 0→2)
- 해독 키: `NANDOKJEUNG_SECRET` 환경변수 (서버만 보유)

## 사용법

### 난독화

```bash
# 문자열 → stdout
nandokjeung obfuscate -t "비밀 메시지"

# 파일 → .jp 저장
nandokjeung obfuscate -i secret.txt -o secret.jp

# heavy 모드 (죽은 코드·분할 청크)
nandokjeung obfuscate -t "Hello" --mode heavy --chunks 2 -o hello.jp
```

### 복원 (reveal)

```bash
nandokjeung reveal secret.jp
nandokjeung reveal secret.jp -o restored.txt
```

### 정신뼝! 런타임 (내장)

```bash
nandokjeung run examples/hello.jp
nandokjeung serve examples/web.jp --port 8080
```

## 모드

| 모드 | 설명 |
|------|------|
| `print` | `행정 ! <문자열> !` 출력 프로그램 (기본) |
| `heavy` | 정병 주석·데드코드·청크 분할 |
| `literal` | `저엉신저엉신` 비트 리터럴만 |
| `fortress` | XOR + 노이즈 셔드 + jp 다중 래핑 (웹/API) |

## 예시

**입력**
```
비밀번호: admin123
```

**출력** (`secret.jp` 일부)
```
행정 ! 저엉신저엉신 정신 뼝 정신 정신 ... 저엉신저엉신 !
```

**복원**
```bash
$ nandokjeung reveal secret.jp
비밀번호: admin123
```

## 정신뼝!이란?

난독증이 사용하는 난해 프로그래밍 언어. 자세한 문법은 [docs/SPEC.md](docs/SPEC.md).

## GitHub 협업 · 배포

- PR: `main` 대상, CI (`fmt` / `clippy` / `test` / web build) 자동 실행
- **deploy 브랜치**: `main` push 시 정적 웹(`web/out`) 자동 배포 → GitHub Pages 소스
- **릴리스**: `v*` 태그 push 시 크로스 플랫폼 바이너리 GitHub Releases 업로드

자세한 설정: [docs/DEPLOY.md](docs/DEPLOY.md) · [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)

## 라이선스

MIT
