# ERRORS.md

형식: `error[코드] 줄:열: 메시지`

| 코드 | 단계 | 예시 |
|------|------|------|
| E000 | Lexer | 허용되지 않은 토큰 |
| E100 | Parser | 예상 토큰 불일치 |
| E001 | Semantic | undefined variable |
| E002 | Semantic | duplicate declaration |
| E003 | Semantic | type mismatch |
| E010 | Semantic | argument count |
| E012 | Semantic | undefined function |
| E040 | Runtime | uncaught throw |

## 예시

```text
error[E001] 3:1: undefined variable '우범'
error[E003] 5:1: type mismatch: 정수 vs 뼝문자
```

## Negative 테스트

`cargo test`의 integration.rs에 포함.
