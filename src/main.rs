use clap::Parser;
use std::fs;
use std::path::PathBuf;
use crate::etc::deadcode::DeadCodeAnalyzer;
use crate::parser::*;

mod utils;
mod lexer;
mod parser;
mod ast;
mod interpreter;
// mod runtime;
mod value;
mod modules;
// mod repl;
mod core;
mod etc;

#[derive(Parser)]
#[command(name = "crabby")]
#[command(author="Kazooki123")]
#[command(about = "Crabby programming language interpreter", long_about=None)]
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
        let tokens = lexer::tokenize(&source).await;
        let ast = parse(tokens.expect("Failed to parse token")).await.expect("Failed to parse AST");
        let mut interpreter = interpreter::Interpreter::new(Some(absolute_path));
        // let mut runtime = runtime::Runtime::new(Some(absolute_path));
        interpreter.interpret(&ast).await?;
        // runtime.runtime(&ast).await?;

        // Shows version of Crabby
        if cli.version {
            println!("Crabby Version: {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }

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
