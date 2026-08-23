# CONTRIBUTING.md

## 개발 환경

- Rust stable (`rust-toolchain.toml` 참고)
- Node.js 20+ (웹 UI)
- `cargo test` 필수 통과

## Git 협업 흐름

1. `main`에서 feature 브랜치 생성
2. 변경 + 테스트
3. PR → CI 통과 후 리뷰·머지
4. `main` 머지 시 **Deploy** 워크플로가 `deploy` 브랜치 갱신 (GitHub Pages)

배포·Pages 설정은 [DEPLOY.md](DEPLOY.md) 참고.

## 변경 절차

1. `docs/SPEC.md`와 구현 일치 확인
2. 테스트 추가
3. `examples/` 업데이트 (필요 시)

## 설계 원칙

- 허용 토큰 15개 고정
- 단순 치환 금지
- 문법 변경 시 SPEC 버전 갱신

## 코딩 스타일

- `cargo fmt`
- `cargo clippy` 권장
- 웹: `cd web && npm run build` (변경 시)
