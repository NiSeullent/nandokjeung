use clap::{Parser, Subcommand};
use nandokjeung::interpreter::Interpreter;
use nandokjeung::lexer::Lexer;
use nandokjeung::obfuscator::{self, ObfuscateMode, ObfuscateOptions};
use nandokjeung::parser::Parser as JpParser;
use nandokjeung::semantic::SemanticAnalyzer;
use nandokjeung::api_server;
use nandokjeung::fortress;
use nandokjeung::CompileError;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "nandokjeung",
    version = "1.0.0",
    about = "난독증 — 정신뼝! 기반 난독화 프로그램",
    long_about = "평문을 정신뼝!(.jp) 소스로 난독화하고, reveal로 원문을 복원합니다."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 평문·파일을 정신뼝! 소스로 난독화
    Obfuscate {
        /// 난독화할 평문 (파일/표준입력 대신)
        #[arg(short, long)]
        text: Option<String>,
        /// 입력 파일
        #[arg(short, long)]
        input: Option<PathBuf>,
        /// 출력 .jp 경로 (없으면 stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// print | heavy | literal | fortress
        #[arg(short, long, default_value = "print")]
        mode: String,
        /// 문자열 분할 청크 수 (print/heavy)
        #[arg(long, default_value_t = 1)]
        chunks: usize,
    },
    /// 난독화된 .jp 실행 결과(원문) 출력
    Reveal {
        file: PathBuf,
        /// 복원 결과 저장 경로
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 정신뼝! 소스 실행
    Run {
        file: PathBuf,
    },
    /// 문법·의미 검사
    Check {
        file: PathBuf,
    },
    /// AST 덤프
    Compile {
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// .jp 핸들러 웹서버
    Serve {
        file: PathBuf,
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// JSON API (obfuscate / reveal / relay)
    Api {
        #[arg(short, long, default_value_t = 8787)]
        port: u16,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Obfuscate {
            text,
            input,
            output,
            mode,
            chunks,
        } => obfuscate_cmd(text, input, output, &mode, chunks),
        Commands::Reveal { file, output } => reveal_cmd(&file, output.as_ref()),
        Commands::Run { file } => run_file(&file),
        Commands::Compile { file, output } => compile_file(&file, output.as_ref()),
        Commands::Check { file } => check_file(&file),
        Commands::Serve { file, port } => serve_file(&file, port),
        Commands::Api { port } => run_api(port),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn parse_mode(mode: &str) -> Result<ObfuscateMode, CompileError> {
    match mode.to_lowercase().as_str() {
        "print" => Ok(ObfuscateMode::Print),
        "heavy" => Ok(ObfuscateMode::Heavy),
        "literal" => Ok(ObfuscateMode::Literal),
        "fortress" => {
            // handled separately in obfuscate_cmd
            Ok(ObfuscateMode::Heavy)
        }
        other => Err(CompileError::new(
            "E070",
            format!("unknown mode '{other}' (use print, heavy, literal)"),
            0,
            0,
        )),
    }
}

fn obfuscate_cmd(
    text: Option<String>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    mode: &str,
    chunks: usize,
) -> Result<(), CompileError> {
    let opts = ObfuscateOptions {
        mode: parse_mode(mode)?,
        split_chunks: chunks.max(1),
    };

    let payload = if mode.eq_ignore_ascii_case("fortress") {
        let nonce = uuid::Uuid::new_v4().to_string();
        let text = if let Some(t) = text.clone() {
            t
        } else if let Some(path) = &input {
            fs::read_to_string(path).map_err(|e| CompileError::new("E050", e.to_string(), 0, 0))?
        } else if !io::stdin().is_terminal() {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| CompileError::new("E050", e.to_string(), 0, 0))?;
            buf
        } else {
            return Err(CompileError::new(
                "E071",
                "provide --text, --input, or pipe stdin".to_string(),
                0,
                0,
            ));
        };
        fortress::fortress_obfuscate(&text, &nonce)?
    } else if let Some(t) = text {
        obfuscator::obfuscate_text(&t, &opts)
    } else if let Some(path) = input {
        obfuscator::obfuscate_file(&path, &opts)?
    } else if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| CompileError::new("E050", e.to_string(), 0, 0))?;
        obfuscator::obfuscate_text(&buf, &opts)
    } else {
        return Err(CompileError::new(
            "E071",
            "provide --text, --input, or pipe stdin".to_string(),
            0,
            0,
        ));
    };

    if let Some(path) = output {
        fs::write(&path, &payload)
            .map_err(|e| CompileError::new("E051", e.to_string(), 0, 0))?;
        eprintln!("난독화 완료: {}", path.display());
    } else {
        print!("{payload}");
        io_flush();
    }
    Ok(())
}

fn reveal_cmd(file: &PathBuf, output: Option<&PathBuf>) -> Result<(), CompileError> {
    let plain = obfuscator::reveal_file(file)?;
    if let Some(path) = output {
        fs::write(path, &plain)
            .map_err(|e| CompileError::new("E051", e.to_string(), 0, 0))?;
        eprintln!("복원 완료: {}", path.display());
    } else {
        print!("{plain}");
        io_flush();
    }
    Ok(())
}

fn read_source(path: &PathBuf) -> Result<String, CompileError> {
    fs::read_to_string(path).map_err(|e| CompileError::new("E050", e.to_string(), 0, 0))
}

fn compile_pipeline(source: &str) -> Result<nandokjeung::ast::Program, CompileError> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = JpParser::new(tokens).parse()?;
    SemanticAnalyzer::analyze(&program)?;
    Ok(program)
}

fn check_file(path: &PathBuf) -> Result<(), CompileError> {
    let source = read_source(path)?;
    compile_pipeline(&source)?;
    println!("OK: {}", path.display());
    Ok(())
}

fn compile_file(path: &PathBuf, output: Option<&PathBuf>) -> Result<(), CompileError> {
    let source = read_source(path)?;
    let program = compile_pipeline(&source)?;
    let out = output.map(|p| p.display().to_string()).unwrap_or_else(|| {
        path.file_stem()
            .map(|s| format!("{}.jpc", s.to_string_lossy()))
            .unwrap_or_else(|| "out.jpc".into())
    });
    let ast_dump = format!("{program:#?}");
    fs::write(&out, ast_dump).map_err(|e| CompileError::new("E051", e.to_string(), 0, 0))?;
    println!("compiled: {}", out);
    Ok(())
}

fn run_file(path: &PathBuf) -> Result<(), CompileError> {
    let source = read_source(path)?;
    let program = compile_pipeline(&source)?;
    let output = Interpreter::run(&program)?;
    print!("{output}");
    io_flush();
    Ok(())
}

fn serve_file(path: &PathBuf, port: u16) -> Result<(), CompileError> {
    let server = nandokjeung::server::WebServer::from_file(path, port)?;
    server.run()
}

fn run_api(port: u16) -> Result<(), CompileError> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CompileError::new("E060", e.to_string(), 0, 0))?;
    rt.block_on(api_server::run_api(port))
}

fn io_flush() {
    let _ = io::stdout().flush();
}
