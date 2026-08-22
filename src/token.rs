use std::fmt;

/// Surface tokens allowed in 정신뼝! source (v1.0 fixed set).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Jeongsin,
    Ppyung,
    Menhera,
    Ipwon,
    Jaui,
    Dongui,
    Boho,
    Haengjeong,
    Bang,
    Question,
    Ubeom,
    Sonyeon,
    Won,
    Jeongbyeong,
    Jeoeongsin,
    Eof,
}

impl TokenKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "정신" => Some(Self::Jeongsin),
            "뼝" => Some(Self::Ppyung),
            "멘헤라" => Some(Self::Menhera),
            "입원" => Some(Self::Ipwon),
            "자의" => Some(Self::Jaui),
            "동의" => Some(Self::Dongui),
            "보호" => Some(Self::Boho),
            "행정" => Some(Self::Haengjeong),
            "!" => Some(Self::Bang),
            "?" => Some(Self::Question),
            "우범" => Some(Self::Ubeom),
            "소년" => Some(Self::Sonyeon),
            "원" => Some(Self::Won),
            "정병" => Some(Self::Jeongbyeong),
            "저엉신저엉신" => Some(Self::Jeoeongsin),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Jeongsin => "정신",
            Self::Ppyung => "뼝",
            Self::Menhera => "멘헤라",
            Self::Ipwon => "입원",
            Self::Jaui => "자의",
            Self::Dongui => "동의",
            Self::Boho => "보호",
            Self::Haengjeong => "행정",
            Self::Bang => "!",
            Self::Question => "?",
            Self::Ubeom => "우범",
            Self::Sonyeon => "소년",
            Self::Won => "원",
            Self::Jeongbyeong => "정병",
            Self::Jeoeongsin => "저엉신저엉신",
            Self::Eof => "<eof>",
        }
    }

    /// Tokens that may appear inside a user identifier (not `!` or `?`).
    pub fn is_identifier_part(self) -> bool {
        !matches!(self, Self::Bang | Self::Question | Self::Eof)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}:{}", self.lexeme, self.line, self.column)
    }
}

/// Multi-token identifier (e.g. `뼝 정신`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub parts: Vec<TokenKind>,
    pub span_line: usize,
    pub span_column: usize,
}

impl Identifier {
    pub fn name(&self) -> String {
        self.parts
            .iter()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
