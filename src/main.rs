use clap::Parser;
use std::fs;
use std::path::PathBuf;
use crate::compile::parse;

mod utils;
mod lexer;
mod parser;
mod compile;

#[derive(Parser)]
#[command(name = "crabby")]
#[command(about = "Crabby programming language compiler")]
#[command(version = include_str!(".././version.txt"))]
#[command(disable_version_flag = true)]
struct Cli {
    #[arg(help = "Input .crab or .cb file")]
    input: Option<PathBuf>,

    #[arg(short, long, help = "Show version information")]
    version: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(input) = cli.input {
        if !input.exists() {
            return Err("Input file does not exist".into());
        }

        let ext = input.extension().unwrap_or_default();
        if ext != "crab" && ext != "cb" {
            return Err("Input file must have .crab or .cb extension".into());
        }

        // Get the absolute path of the input file
        let absolute_path = input.canonicalize()?;
        let source = fs::read_to_string(&absolute_path)?;
        // Lexical analysis
        let tokens = lexer::tokenize(&source)?;

        // Parsing
        let ast = parse(tokens)?;

        // Create compiler with the current file path
        let mut compiler = compile::Compiler::new(Some(absolute_path));

        // Compilation
        compiler.compile(&ast)?;
    }

    Ok(())
}
