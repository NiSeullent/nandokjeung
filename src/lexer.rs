use crate::error::{CompileError, Result};
use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            self.offset += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn match_keyword(&mut self, keyword: &str) -> bool {
        let start = self.pos;
        let line = self.line;
        let column = self.column;
        let offset = self.offset;

        for expected in keyword.chars() {
            match self.peek() {
                Some(ch) if ch == expected => {
                    self.advance();
                }
                _ => {
                    self.pos = start;
                    self.line = line;
                    self.column = column;
                    self.offset = offset;
                    return false;
                }
            }
        }
        true
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let line = self.line;
        let column = self.column;
        let offset = self.offset;

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::Eof,
                    lexeme: String::new(),
                    line,
                    column,
                    offset,
                });
            }
        };

        if ch == '!' {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Bang,
                lexeme: "!".into(),
                line,
                column,
                offset,
            });
        }

        if ch == '?' {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Question,
                lexeme: "?".into(),
                line,
                column,
                offset,
            });
        }

        const KEYWORDS: &[(&str, TokenKind)] = &[
            ("저엉신저엉신", TokenKind::Jeoeongsin),
            ("멘헤라", TokenKind::Menhera),
            ("정병", TokenKind::Jeongbyeong),
            ("행정", TokenKind::Haengjeong),
            ("정신", TokenKind::Jeongsin),
            ("동의", TokenKind::Dongui),
            ("자의", TokenKind::Jaui),
            ("소년", TokenKind::Sonyeon),
            ("입원", TokenKind::Ipwon),
            ("보호", TokenKind::Boho),
            ("우범", TokenKind::Ubeom),
            ("뼝", TokenKind::Ppyung),
            ("원", TokenKind::Won),
        ];

        for (kw, kind) in KEYWORDS {
            if self.match_keyword(kw) {
                return Ok(Token {
                    kind: *kind,
                    lexeme: kw.to_string(),
                    line,
                    column,
                    offset,
                });
            }
        }

        if self.is_korean_start(ch) {
            let start = self.pos;
            while let Some(c) = self.peek() {
                if c.is_whitespace() || c == '!' || c == '?' {
                    break;
                }
                if !self.is_korean_char(c) {
                    break;
                }
                self.advance();
            }
            let raw: String = self.chars[start..self.pos].iter().collect();
            return Err(CompileError::lexer(
                format!("허용되지 않은 토큰: '{raw}'"),
                line,
                column,
            ));
        }

        self.advance();
        Err(CompileError::lexer(
            format!("허용되지 않은 문자: '{ch}'"),
            line,
            column,
        ))
    }

    fn is_korean_start(&self, ch: char) -> bool {
        matches!(ch, '\u{AC00}'..='\u{D7A3}' | '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}')
    }

    fn is_korean_char(&self, ch: char) -> bool {
        self.is_korean_start(ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    fn kinds(source: &str) -> Vec<TokenKind> {
        Lexer::new(source)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn tokenize_basic() {
        assert_eq!(
            kinds("정신 뼝 멘헤라"),
            vec![
                TokenKind::Jeongsin,
                TokenKind::Ppyung,
                TokenKind::Menhera,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenize_whitespace_and_newlines() {
        assert_eq!(
            kinds("행정\n정신\n!"),
            vec![
                TokenKind::Haengjeong,
                TokenKind::Jeongsin,
                TokenKind::Bang,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn rejects_invalid_token() {
        let err = Lexer::new("print").tokenize().unwrap_err();
        assert_eq!(err.code, "E000");
    }

    #[test]
    fn tokenize_jeoeongsin() {
        assert_eq!(
            kinds("저엉신저엉신 정신 저엉신저엉신"),
            vec![
                TokenKind::Jeoeongsin,
                TokenKind::Jeongsin,
                TokenKind::Jeoeongsin,
                TokenKind::Eof,
            ]
        );
    }
}
