use crate::etc::deadcode::DeadCodeAnalyzer;
use crate::parser::*;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

mod ast;
mod core;
mod etc;
mod interpreter;
mod lexer;
mod modules;
mod parser;
mod repl;
mod runtime;
mod utils;
mod value;

#[derive(Parser)]
#[command(name = "crabby")]
#[command(author = "Kazooki123")]
#[command(about = "Crabby Programming Language Interpreter", long_about=None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(disable_version_flag = true)]
pub struct Cli {
    #[arg(help = "Input .crab or .cb file")]
    input: Option<PathBuf>,

    #[arg(short, long, help = "Show version information")]
    version: bool,

    #[arg(long, help = "Analyze code for unused declarations")]
    deadcodewarn: bool,
    // #[arg(help = "REPL playground to test Crabby")]
    // repl: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(input) = cli.input {
        let absolute_path = input.canonicalize().expect("Failed to get absolute path");
        let source = fs::read_to_string(&absolute_path).expect("Failed to read file");
        let tokens = lexer::TokenStream::tokenize(source);
        let ast = parse(tokens.expect("Failed to parse token")).expect("Failed to parse AST");
        let interpreter = interpreter::Interpreter::new(Some(absolute_path));
        interpreter.interpret(&ast);

        // Shows the version of Crabby
        if cli.version {
            println!("Crabby Version: {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }

        // When used, it analyzes any dead & unused code
        if cli.deadcodewarn {
            let mut analyzer = DeadCodeAnalyzer::new();
            let warnings = analyzer.analyze(&ast)?;
            if !warnings.is_empty() {
                println!("\nDead code warnings:");
                for warning in warnings {
                    println!("Warning: {:?}", warning);
                }
            }
        }
    }

    Ok(())
}
