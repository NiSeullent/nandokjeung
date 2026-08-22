# EXAMPLES.md

모든 예제는 `cargo test`로 검증됨.

| # | 파일 | 출력/동작 |
|---|------|-----------|
| 1 | hello.jp | Hello World |
| 2 | number.jp | 127 |
| 3 | add.jp | 22 |
| 4 | variables.jp | 127 |
| 5 | conditional.jp | true 리터럴 |
| 6 | loop.jp | 0123456 |
| 7 | function.jp | 14 (7×2) |
| 8 | recursive.jp | 24 (4!) |
| 9 | input.jp | 입력 에코 |
| 10 | game.jp | 맞추기 게임 |
| 11 | fizzbuzz.jp | 12Fizz45 |
| 12 | error.jp | Boom (catch) |
| 13 | web.jp | 정신뼝? HTML 페이지 (웹서버용) |

실행:

```bash
cargo run -- run examples/hello.jp
cargo run -- serve examples/web.jp --port 8080
```
