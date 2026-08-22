use crate::error::{CompileError, Result};
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::ast::Program;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;

const MAX_REQUEST_BYTES: usize = 8192;

pub struct WebServer {
    program: Arc<Program>,
    port: u16,
}

impl WebServer {
    pub fn from_file(path: &Path, port: u16) -> Result<Self> {
        let source = fs::read_to_string(path)
            .map_err(|e| CompileError::new("E050", e.to_string(), 0, 0))?;
        let program = compile(&source)?;
        Ok(Self {
            program: Arc::new(program),
            port,
        })
    }

    pub fn run(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).map_err(|e| {
            CompileError::new(
                "E060",
                format!("cannot bind {addr}: {e}"),
                0,
                0,
            )
        })?;

        eprintln!("정신뼝! 웹서버 → http://127.0.0.1:{}", self.port);
        eprintln!("Ctrl+C 로 종료");

        for conn in listener.incoming() {
            match conn {
                Ok(stream) => {
                    let program = Arc::clone(&self.program);
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, &program) {
                            eprintln!("request error: {e}");
                        }
                    });
                }
                Err(e) => eprintln!("accept error: {e}"),
            }
        }
        Ok(())
    }
}

fn compile(source: &str) -> Result<Program> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = Parser::new(tokens).parse()?;
    SemanticAnalyzer::analyze(&program)?;
    Ok(program)
}

fn handle_connection(mut stream: TcpStream, program: &Program) -> Result<()> {
    let request = read_http_request(&mut stream)?;
    let path = request.path.clone();

    let body = Interpreter::run_with_request(program, &path)?;
    let response = build_http_response(&body);
    stream
        .write_all(response.as_bytes())
        .map_err(|e| CompileError::new("E061", e.to_string(), 0, 0))?;
    stream
        .flush()
        .map_err(|e| CompileError::new("E061", e.to_string(), 0, 0))?;
    Ok(())
}

struct HttpRequest {
    path: String,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest> {
    let mut buf = vec![0u8; MAX_REQUEST_BYTES];
    let n = stream
        .read(&mut buf)
        .map_err(|e| CompileError::new("E062", e.to_string(), 0, 0))?;
    let raw = String::from_utf8_lossy(&buf[..n]);
    let first_line = raw.lines().next().unwrap_or("");
    let path = first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .to_string();
    Ok(HttpRequest { path })
}

fn build_http_response(body: &str) -> String {
    let bytes = body.as_bytes();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        bytes.len(),
        body
    )
}
