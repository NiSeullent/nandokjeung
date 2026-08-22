use nandokjeung::interpreter::Interpreter;
use nandokjeung::lexer::Lexer;
use nandokjeung::obfuscator::{self, ObfuscateOptions};
use nandokjeung::parser::Parser;
use nandokjeung::semantic::SemanticAnalyzer;
use std::fs;
use std::path::PathBuf;

fn run_file(path: &str) -> String {
    let source = fs::read_to_string(path).expect("read example");
    let tokens = Lexer::new(&source).tokenize().expect("lex");
    let program = Parser::new(tokens).parse().expect("parse");
    SemanticAnalyzer::analyze(&program).expect("semantic");
    Interpreter::run(&program).expect("run")
}

fn run_file_with_input(path: &str, input: Vec<String>) -> String {
    let source = fs::read_to_string(path).expect("read example");
    let tokens = Lexer::new(&source).tokenize().expect("lex");
    let program = Parser::new(tokens).parse().expect("parse");
    SemanticAnalyzer::analyze(&program).expect("semantic");
    Interpreter::run_with_input(&program, input).expect("run")
}

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples").join(name)
}

#[test]
fn hello_world() {
    assert_eq!(run_file(example_path("hello.jp").to_str().unwrap()), "Hello World");
}

#[test]
fn number_output() {
    assert_eq!(run_file(example_path("number.jp").to_str().unwrap()), "127");
}

#[test]
fn addition() {
    assert_eq!(run_file(example_path("add.jp").to_str().unwrap()), "22");
}

#[test]
fn variables() {
    assert_eq!(run_file(example_path("variables.jp").to_str().unwrap()), "127");
}

#[test]
fn conditional_true_branch() {
    assert_eq!(
        run_file(example_path("conditional.jp").to_str().unwrap()),
        "멘헤라 입원 멘헤라"
    );
}

#[test]
fn loop_counting() {
    assert_eq!(run_file(example_path("loop.jp").to_str().unwrap()), "0123456");
}

#[test]
fn function_double() {
    assert_eq!(run_file(example_path("function.jp").to_str().unwrap()), "14");
}

#[test]
fn factorial_loop() {
    assert_eq!(run_file(example_path("recursive.jp").to_str().unwrap()), "24");
}

#[test]
fn user_input_echo() {
    assert_eq!(
        run_file_with_input(example_path("input.jp").to_str().unwrap(), vec!["정신뼝".into()]),
        "정신뼝"
    );
}

#[test]
fn fizzbuzz_partial() {
    assert_eq!(
        run_file(example_path("fizzbuzz.jp").to_str().unwrap()),
        "12Fizz45"
    );
}

#[test]
fn guessing_game_win() {
    assert_eq!(
        run_file_with_input(example_path("game.jp").to_str().unwrap(), vec!["7".into()]),
        "멘헤라 입원 멘헤라"
    );
}

#[test]
fn error_handling() {
    assert_eq!(run_file(example_path("error.jp").to_str().unwrap()), "Boom");
}

#[test]
fn lexer_rejects_invalid() {
    let err = Lexer::new("hello").tokenize().unwrap_err();
    assert_eq!(err.code, "E000");
}

#[test]
fn semantic_undefined_var() {
    let source = "뼝 정신 뼝 원 !";
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse().unwrap();
    assert!(SemanticAnalyzer::analyze(&program).is_ok());
}

#[test]
fn semantic_type_error() {
    let source = "행정 정신 뼝 ! 뼝 정신 뼝 멘헤라 보호 멘헤라 뼝 원 !";
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse().unwrap();
    let err = SemanticAnalyzer::analyze(&program).unwrap_err();
    assert_eq!(err.code, "E003");
}

#[test]
fn obfuscate_reveal_roundtrip() {
    let src = obfuscator::obfuscate_text(
        "secret payload",
        &ObfuscateOptions::default(),
    );
    let out = obfuscator::reveal_source(&src).expect("reveal");
    assert_eq!(out, "secret payload");
}

#[test]
fn web_page_html() {
    let html = run_file(example_path("web.jp").to_str().unwrap());
    assert!(html.contains("<title>정신뼝?</title>"));
    assert!(html.contains("Jeongsin-Ppyung!"));
    assert!(html.contains("저엉신저엉신"));
}
