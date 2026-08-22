#!/usr/bin/env python3
"""Generate examples/web.jp from HTML template."""
from pathlib import Path

HTML = """<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="utf-8">
<title>정신뼝?</title>
<style>
body{font-family:system-ui,sans-serif;max-width:720px;margin:2rem auto;padding:0 1rem;background:#0f0f1a;color:#e8e8f0;line-height:1.6}
h1{color:#ff6b9d;font-size:2.5rem}
h2{color:#c77dff;margin-top:2rem}
.token{display:inline-block;background:#1e1e3f;padding:.2rem .5rem;margin:.15rem;border-radius:6px;font-size:.9rem;border:1px solid #3d3d6b}
code{background:#1e1e3f;padding:.2rem .4rem;border-radius:4px}
a{color:#7dd3fc}
</style>
</head>
<body>
<h1>정신뼝?</h1>
<p><strong>Jeongsin-Ppyung!</strong> — 15개 토큰만으로 돌아가는 난해 프로그래밍 언어입니다.</p>
<h2>허용 토큰</h2>
<p>
<span class="token">정신</span> <span class="token">뼝</span> <span class="token">멘헤라</span>
<span class="token">입원</span> <span class="token">자의</span> <span class="token">동의</span>
<span class="token">보호</span> <span class="token">행정</span> <span class="token">!</span>
<span class="token">?</span> <span class="token">우범</span> <span class="token">소년</span>
<span class="token">원</span> <span class="token">정병</span> <span class="token">저엉신저엉신</span>
</p>
<h2>Hello World</h2>
<p><code>ppyung run examples/hello.jp</code> → Hello World</p>
<h2>이 페이지는?</h2>
<p>정신뼝! 인터프리터가 <code>examples/web.jp</code>를 실행해서 만든 HTML입니다. 웹서버도 정신뼝!으로 띄웁니다 ㅋㅋㅋ</p>
<p><a href="/">새로고침</a></p>
</body>
</html>"""


def encode(s: str) -> str:
    bits = []
    for b in s.encode("utf-8"):
        bits.extend("뼝" if c == "1" else "정신" for c in format(b, "08b"))
    return "저엉신저엉신 " + " ".join(bits) + " 저엉신저엉신"


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    out = root / "examples" / "web.jp"
    bits = encode(HTML)
    out.write_text(f"행정 ! {bits} !\n", encoding="utf-8")
    print(f"wrote {out} ({len(HTML)} chars HTML)")


if __name__ == "__main__":
    main()
