use clap::Parser;
use colored::*;
use joelang::JoeLang;
use std::fs;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "cjoe", author = "Joe", version = "0.1.0", about = "Joe's Conscience — Reality & Syntax Checker for JoeLang")]
struct Args {
    /// Path to the .joe story script
    #[arg(value_name = "FILE")]
    file: String,

    /// Pretty-print the abstract syntax story tree (AST)
    #[arg(long)]
    ast: bool,

    /// Output diagnostics in JSON format
    #[arg(long)]
    json: bool,
}

fn main() {
    let args = Args::parse();

    let content = match fs::read_to_string(&args.file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: Failed to read '{}': {}", "Error".red().bold(), args.file, e);
            process::exit(1);
        }
    };

    if args.ast {
        match JoeLang::parse(&content) {
            Ok(stmts) => {
                println!("{:#?}", stmts);
                return;
            }
            Err(e) => {
                eprintln!("{}: {}", "Syntax Breakdown".red().bold(), e);
                process::exit(1);
            }
        }
    }

    let joe = JoeLang::new();
    match joe.check(&content) {
        Ok(diagnostics) => {
            if args.json {
                let json_items: Vec<serde_json::Value> = diagnostics.iter().map(|d| {
                    serde_json::json!({
                        "message": d.message,
                        "line": d.span.line,
                        "col": d.span.col,
                        "is_warning": d.is_warning
                    })
                }).collect();
                println!("{}", serde_json::to_string_pretty(&json_items).unwrap());
            } else {
                if diagnostics.is_empty() {
                    println!("{}", "✓ Joe's conscience is completely clear. No existential dread detected.".green().bold());
                } else {
                    let mut error_count = 0;
                    for d in &diagnostics {
                        if d.is_warning {
                            println!("{}: {}", "Warning (Conscience)".yellow().bold(), d.message);
                        } else {
                            error_count += 1;
                            println!("{}: {}", "Error (Reality Check)".red().bold(), d.message);
                        }
                    }
                    if error_count > 0 {
                        println!("\n{} {} error(s) found.", "✖".red(), error_count);
                        process::exit(1);
                    } else {
                        println!("\n{} Clear with warnings.", "⚠".yellow());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Syntax Breakdown".red().bold(), e);
            process::exit(1);
        }
    }
}
