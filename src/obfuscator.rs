use crate::error::{CompileError, Result};
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObfuscateMode {
    /// 행정 ! <문자열> ! 한 줄 출력 프로그램
    Print,
    /// 정병 주석·죽은 코드를 섞은 출력 프로그램
    Heavy,
    /// 저엉신저엉신 비트 리터럴만
    Literal,
}

#[derive(Debug, Clone)]
pub struct ObfuscateOptions {
    pub mode: ObfuscateMode,
    pub split_chunks: usize,
}

impl Default for ObfuscateOptions {
    fn default() -> Self {
        Self {
            mode: ObfuscateMode::Print,
            split_chunks: 1,
        }
    }
}

pub fn encode_bits(bytes: &[u8]) -> String {
    let mut out = Vec::new();
    for byte in bytes {
        for bit in format!("{byte:08b}").chars() {
            out.push(if bit == '1' { "뼝" } else { "정신" });
        }
    }
    out.join(" ")
}

pub fn encode_string_literal(text: &str) -> String {
    format!(
        "저엉신저엉신 {} 저엉신저엉신",
        encode_bits(text.as_bytes())
    )
}

pub fn encode_int(value: i64) -> String {
    if value == 0 {
        return "멘헤라 정신 멘헤라".to_string();
    }
    let bits: String = if value > 0 {
        format!("{value:b}")
    } else {
        format!("{}", (value as u64).wrapping_neg())
    };
    let body = bits
        .chars()
        .map(|c| if c == '1' { "뼝" } else { "정신" })
        .collect::<Vec<_>>()
        .join(" ");
    format!("멘헤라 {body} 멘헤라")
}

pub fn obfuscate_text(text: &str, options: &ObfuscateOptions) -> String {
    match options.mode {
        ObfuscateMode::Literal => encode_string_literal(text),
        ObfuscateMode::Print => obfuscate_print_program(text, options.split_chunks),
        ObfuscateMode::Heavy => obfuscate_heavy_program(text, options.split_chunks),
    }
}

pub fn obfuscate_file(path: &Path, options: &ObfuscateOptions) -> Result<String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| CompileError::new("E050", e.to_string(), 0, 0))?;
    Ok(obfuscate_text(&raw, options))
}

pub fn reveal_source(source: &str) -> Result<String> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = Parser::new(tokens).parse()?;
    SemanticAnalyzer::analyze(&program)?;
    Interpreter::run(&program).map_err(|e| e)
}

pub fn reveal_file(path: &Path) -> Result<String> {
    let source = fs::read_to_string(path)
        .map_err(|e| CompileError::new("E050", e.to_string(), 0, 0))?;
    reveal_source(&source)
}

fn obfuscate_print_program(text: &str, chunks: usize) -> String {
    if chunks <= 1 || text.len() < 4 {
        return format!("행정 ! {} !\n", encode_string_literal(text));
    }

    let parts = split_utf8_chunks(text, chunks);
    let mut lines = Vec::new();
    lines.push("정병 멘헤라 입원 보호 행정 정신 뼝 !".to_string());

    for (i, part) in parts.iter().enumerate() {
        let name = chunk_var_name(i);
        lines.push(format!("뼝 정신 {name} {} !", encode_string_literal(part)));
    }

    let first = chunk_var_name(0);
    let mut expr = format!("정신 {first}");
    for i in 1..parts.len() {
        let name = chunk_var_name(i);
        expr = format!("{expr} 뼝 정신 {name}");
    }
    lines.push(format!("행정 ! {expr} !"));
    lines.push(String::new());
    lines.join("\n")
}

fn obfuscate_heavy_program(text: &str, chunks: usize) -> String {
    let mut lines = vec![
        "정병 멘헤라 우범 소년 정병 입원 보호 행정 !".to_string(),
        "정병 자의 동의 보호 행정 정신 뼝 !".to_string(),
        "정병 입원 멘헤라 보호 뼝 !".to_string(),
    ];

    let core = obfuscate_print_program(text, chunks.max(2));
    for line in core.lines() {
        if line.starts_with("정병") {
            continue;
        }
        lines.push(line.to_string());
    }
    lines.push(String::new());
    lines.join("\n")
}

fn split_utf8_chunks(text: &str, chunks: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return vec![String::new()];
    }
    let chunk_size = (chars.len() + chunks - 1) / chunks;
    chars
        .chunks(chunk_size.max(1))
        .map(|c| c.iter().collect())
        .collect()
}

fn chunk_var_name(index: usize) -> String {
    match index {
        0 => "우범".to_string(),
        1 => "소년".to_string(),
        2 => "보호".to_string(),
        3 => "행정".to_string(),
        _ => "멘헤라".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_obfuscate_reveal() {
        let text = "난독증 테스트";
        let src = obfuscate_text(text, &ObfuscateOptions::default());
        let out = reveal_source(&src).expect("reveal");
        assert_eq!(out, text);
    }

    #[test]
    fn heavy_mode_roundtrip() {
        let text = "Hello 난독증";
        let opts = ObfuscateOptions {
            mode: ObfuscateMode::Heavy,
            split_chunks: 2,
        };
        let src = obfuscate_text(text, &opts);
        let out = reveal_source(&src).expect("reveal heavy");
        assert_eq!(out, text);
    }

    #[test]
    fn encode_int_zero() {
        assert_eq!(encode_int(0), "멘헤라 정신 멘헤라");
    }
}
