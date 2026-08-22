# COMPILER.md

## 구조

```
src/
  lexer.rs       # 토큰화
  parser.rs      # AST 생성
  ast.rs         # AST 노드
  semantic.rs    # 타입·스코프 검사
  interpreter.rs # 트리 워크 실행
  main.rs        # CLI (ppyung)
```

## 파이프라인

```
Source → Lexer → Parser → Semantic → Interpreter → Output
```

## 빌드

```bash
cargo build --release
```

바이너리: `target/release/ppyung.exe`

## 향후 (v2+)

- IR 레이어 (`ir.rs`)
- LLVM/WASM codegen
- 다중 인자 함수 호출
- 자기 호스팅
