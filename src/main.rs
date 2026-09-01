mod ast;
mod environment;
mod errors;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod types;
mod value;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use errors::Diagnostic;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn run_source(source: &str, file_name: &str) -> bool {
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(err) => {
            Diagnostic::report(
                source,
                file_name,
                err.span,
                "LexerError",
                &err.message,
                err.hint.as_deref(),
            );
            return false;
        }
    };

    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(stmts) => stmts,
        Err(err) => {
            Diagnostic::report(
                source,
                file_name,
                err.span,
                "SyntaxError",
                &err.message,
                err.hint.as_deref(),
            );
            return false;
        }
    };

    let mut interpreter = Interpreter::with_file(file_name);
    if let Err(err) = interpreter.interpret(&statements) {
        Diagnostic::report(
            source,
            file_name,
            err.span,
            "RuntimeError",
            &err.message,
            err.hint.as_deref(),
        );
        return false;
    }

    true
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let target_file = if args.len() > 1 {
        if args[1] == "run" && args.len() > 2 {
            args[2].clone()
        } else {
            args[1].clone()
        }
    } else if Path::new("main.sr").exists() {
        "main.sr".to_string()
    } else if Path::new("main.sher").exists() {
        "main.sher".to_string()
    } else {
        eprintln!("\x1b[1;33m[Sher CLI]\x1b[0m Usage: sher <file.sr>");
        eprintln!("File 'main.sr' was not found.");
        process::exit(1);
    };

    match fs::read_to_string(&target_file) {
        Ok(content) => {
            if !run_source(&content, &target_file) {
                process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("\x1b[1;31merror[IOError]\x1b[0m: Failed to read file '{}': {}", target_file, err);
            process::exit(1);
        }
    }
}
