use crate::token::Token;
use std::fmt;

pub type Result<T> = std::result::Result<T, CompileError>;

#[derive(Debug, Clone)]
pub struct CompileError {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl CompileError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            line,
            column,
        }
    }

    pub fn lexer(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::new("E000", message, line, column)
    }

    pub fn parser(message: impl Into<String>, token: &Token) -> Self {
        Self::new("E100", message, token.line, token.column)
    }

    pub fn semantic(code: &str, message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::new(code, message, line, column)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error[{}] {}:{}: {}",
            self.code, self.line, self.column, self.message
        )
    }
}

impl std::error::Error for CompileError {}
