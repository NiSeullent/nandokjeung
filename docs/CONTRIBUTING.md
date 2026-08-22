# CONTRIBUTING.md

## 개발 환경

- Rust 1.70+
- `cargo test` 필수 통과

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
