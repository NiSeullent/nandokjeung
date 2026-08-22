#!/usr/bin/env python3
"""Encode text as 정신뼝! string literal bits (정신=0, 뼝=1)."""
import sys

def encode(s: str) -> str:
    bits = []
    for b in s.encode("utf-8"):
        bits.extend("뼝" if c == "1" else "정신" for c in format(b, "08b"))
    return "저엉신저엉신 " + " ".join(bits) + " 저엉신저엉신"

if __name__ == "__main__":
    text = sys.argv[1] if len(sys.argv) > 1 else "Hello World"
    print(encode(text))
