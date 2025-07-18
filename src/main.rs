use clap::Parser;
use std::fs;
use std::path::PathBuf;
use crate::etc::deadcode::DeadCodeAnalyzer;
use crate::parser::parse;

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
#[command(about = "Crabby programming language interpreter")]
#[command(version = include_str!(".././version.txt"))]
#[command(disable_version_flag = true)]
pub struct Cli {
    #[arg(help = "Input .crab or .cb file")]
    input: Option<PathBuf>,

    #[arg(short, long, help = "Show version information")]
    version: bool,

    #[arg(long, help = "Analyze code for unused declarations")]
    deadcode_warn: bool,

    // #[arg(help = "REPL playground to test Crabby")]
    // repl: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(input) = cli.input {
        let absolute_path = input.canonicalize()?;
        let source = fs::read_to_string(&absolute_path)?;
        let tokens = lexer::tokenize(&source).await?;
        let ast = parse(tokens).await?;
        let mut interpreter = interpreter::Interpreter::new(Some(absolute_path));
        // let mut runtime = runtime::Runtime::new(Some(absolute_path));
        interpreter.interpret(&ast).await?;
        // runtime.runtime(&ast).await?;

        if cli.deadcode_warn {
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
