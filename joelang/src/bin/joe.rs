use clap::Parser;
use colored::*;
use joelang::JoeLang;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "joe", author = "Joe", version = "0.1.0", about = "The Chronicles of Joe — Official JoeLang Runner")]
struct Args {
    /// Path to the .joe story script
    #[arg(value_name = "FILE")]
    file: Option<String>,

    /// Run with the Narrator documentary voiceover
    #[arg(short, long)]
    narrate: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(path) = args.file {
        run_file(&path, args.narrate);
    } else {
        run_repl();
    }
}

fn run_file(path: &str, narrate: bool) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}: Failed to read story '{}': {}", "Error".red().bold(), path, e);
            process::exit(1);
        }
    };

    let mut joe = JoeLang::new();

    if narrate {
        match joe.explain(&content) {
            Ok(narration) => println!("{}\n", narration.cyan()),
            Err(e) => eprintln!("{}: {}", "Narration Error".yellow().bold(), e),
        }
    }

    match joe.eval(&content) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}: {}", "Joe had a crisis".red().bold(), e);
            process::exit(1);
        }
    }
}

fn run_repl() {
    println!("{}", "=== The Chronicles of Joe (Interactive Mode) ===".bold().green());
    println!("Type your story or 'Joe rests.' to leave him alone.\n");

    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Failed to initialize interactive prompt: {}", e);
            return;
        }
    };

    let mut joe = JoeLang::new();

    loop {
        let readline = rl.readline("joe> ");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "Joe rests." || trimmed == "exit" || trimmed == "quit" {
                    println!("{}", "Joe goes to sleep. Farewell.".italic());
                    break;
                }
                let _ = rl.add_history_entry(trimmed);

                match joe.eval(trimmed) {
                    Ok(val) => {
                        println!("=> {}", format!("{}", val).bright_black());
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Error".red(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("\nJoe quietly leaves.");
                break;
            }
            Err(err) => {
                eprintln!("Interactive error: {:?}", err);
                break;
            }
        }
    }
}
