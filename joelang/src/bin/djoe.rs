use clap::Parser;
use colored::*;
use joelang::debugger::Debugger;
use joelang::JoeLang;
use rustyline::DefaultEditor;
use std::fs;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "djoe", author = "Joe", version = "0.1.0", about = "Joe's Therapist — Interactive Step-Debugger for JoeLang")]
struct Args {
    /// Path to the .joe story script
    #[arg(value_name = "FILE")]
    file: String,
}

fn print_help() {
    println!("{}", "\nAvailable Therapy Sessions (Debugger Commands):".bold());
    println!("  {}, {}          - Allow Joe to take one step forward in life", "step".green(), "s".green());
    println!("  {}, {}          - Let Joe live his life until the next crisis or breakpoint", "continue".green(), "c".green());
    println!("  {} <line>, {} <line> - Introduce an existential crisis at <line>", "break".green(), "b".green());
    println!("  {} <line>      - Clear crisis at <line>", "clear".green());
    println!("  {}, {}         - Inspect Joe's current mental baggage (local variables)", "mind".green(), "locals".green());
    println!("  {} <expr>, {} <expr> - Ask Joe what he thinks about an expression", "examine".green(), "p".green());
    println!("  {}, {}        - Inspect the traumatic call stack that brought Joe here", "trauma".green(), "stack".green());
    println!("  {}              - Show current source line", "list".green());
    println!("  {}, {}          - Abandon Joe to his fate (quit)\n", "quit".green(), "q".green());
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

    let stmts = match JoeLang::parse(&content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: Syntax Breakdown: {}", "Error".red().bold(), e);
            process::exit(1);
        }
    };

    let source_lines: Vec<&str> = content.lines().collect();

    println!("{}", "=== Joe's Therapy Session (djoe debugger) ===".bold().magenta());
    println!("Welcome. Joe is currently in a state of suspended animation.");
    println!("Type 'help' to review your therapeutic options.\n");

    let mut debugger = Debugger::new();
    let mut rl = match DefaultEditor::new() {
        Ok(ed) => ed,
        Err(e) => {
            eprintln!("Failed to initialize interactive console: {}", e);
            process::exit(1);
        }
    };

    loop {
        let prompt = format!("{}> ", "therapist".magenta().bold());
        let readline = rl.readline(&prompt);
        let line = match readline {
            Ok(l) => l,
            Err(_) => {
                println!("\nSession terminated.");
                break;
            }
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let _ = rl.add_history_entry(trimmed);

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0];

        match cmd {
            "help" | "h" => print_help(),
            "step" | "s" => {
                match debugger.step(&stmts) {
                    Ok(Some(line_num)) => {
                        println!("Joe advanced to line {}:", line_num);
                        if line_num > 0 && line_num <= source_lines.len() {
                            println!("  {} {}", "->".green(), source_lines[line_num - 1].trim());
                        }
                    }
                    Ok(None) => {
                        println!("{}", "Joe has lived out his entire story. There are no more actions to take.".italic().cyan());
                        break;
                    }
                    Err(e) => eprintln!("{}: {}", "Crisis during step".red().bold(), e),
                }
            }
            "continue" | "c" => {
                match debugger.continue_exec(&stmts) {
                    Ok(Some(line_num)) => {
                        println!("{} Joe halted at breakpoint on line {}:", "⏸".yellow(), line_num);
                        if line_num > 0 && line_num <= source_lines.len() {
                            println!("  {} {}", "->".yellow(), source_lines[line_num - 1].trim());
                        }
                    }
                    Ok(None) => {
                        println!("{}", "Joe concluded his journey without further interruption.".italic().cyan());
                        break;
                    }
                    Err(e) => eprintln!("{}: {}", "Crisis during execution".red().bold(), e),
                }
            }
            "break" | "b" => {
                if parts.len() < 2 {
                    eprintln!("Usage: break <line>");
                } else if let Ok(num) = parts[1].parse::<usize>() {
                    debugger.add_breakpoint(num);
                    println!("Breakpoint set at line {}.", num);
                } else {
                    eprintln!("Invalid line number: {}", parts[1]);
                }
            }
            "clear" => {
                if parts.len() < 2 {
                    eprintln!("Usage: clear <line>");
                } else if let Ok(num) = parts[1].parse::<usize>() {
                    debugger.remove_breakpoint(num);
                    println!("Breakpoint removed from line {}.", num);
                } else {
                    eprintln!("Invalid line number: {}", parts[1]);
                }
            }
            "mind" | "locals" => {
                let locals = debugger.locals();
                if locals.is_empty() {
                    println!("{}", "Joe's mind is completely blank.".italic());
                } else {
                    println!("{}", "Joe's current mental baggage:".bold());
                    for (k, v) in locals {
                        println!("  {} = {}", k.cyan(), v);
                    }
                }
            }
            "examine" | "p" => {
                if parts.len() < 2 {
                    eprintln!("Usage: examine <expression>");
                } else {
                    let expr_to_eval = parts[1..].join(" ");
                    match debugger.examine(&expr_to_eval) {
                        Ok(v) => println!("{} = {}", expr_to_eval.bold(), v),
                        Err(e) => eprintln!("Evaluation error: {}", e),
                    }
                }
            }
            "trauma" | "stack" => {
                println!("{}", "Joe's Traumatic Call Stack:".bold());
                for (i, frame) in debugger.call_stack.iter().enumerate() {
                    println!("  #{}: {}", i, frame.cyan());
                }
            }
            "list" => {
                let current_line = if debugger.current_stmt_idx < stmts.len() {
                    stmts[debugger.current_stmt_idx].span.line
                } else {
                    source_lines.len()
                };
                let start = current_line.saturating_sub(3);
                let end = (current_line + 3).min(source_lines.len());
                for (idx, line_text) in source_lines[start..end].iter().enumerate() {
                    let actual_line = start + idx + 1;
                    if actual_line == current_line {
                        println!("{:4} {} {}", actual_line, "->".green(), line_text);
                    } else {
                        println!("{:4}    {}", actual_line, line_text);
                    }
                }
            }
            "quit" | "q" => {
                println!("Leaving Joe to face his life alone.");
                break;
            }
            other => {
                println!("Unknown therapy instruction: '{}'. Type 'help' for options.", other);
            }
        }
    }
}
