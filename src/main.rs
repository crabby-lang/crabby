use clap::Parser;
use std::fs;
use std::path::PathBuf;
use crate::compile::parse;
use crate::deadcode::DeadCodeAnalyzer;

mod utils;
mod lexer;
mod parser;
mod compile;
mod deadcode;

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

    #[arg(long, help = "Analyze code for unused declarations")]
    analyze_deadcode: bool,
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
        let tokens = lexer::tokenize(&source)?;
        let ast = parse(tokens)?;
        let mut compiler = compile::Compiler::new(Some(absolute_path));
        compiler.compile(&ast)?;

        if cli.analyze_deadcode {
            let mut analyzer = DeadCodeAnalyzer::new();
            let warnings = analyzer.analyze(&ast)?;

            if !warnings.is_empty() {
                println!("\nDead code warnings:");
                for warning in warnings {
                    println!("Warning: Unused {} '{}' at line {}, column {}",
                        warning.kind,
                        warning.symbol,
                        warning.line,
                        warning.column
                    );
                }
            }
        }
    }

    Ok(())
}
