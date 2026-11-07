use std::collections::HashMap;
use std::io::{self, Write};
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("print".to_string(), JoeValue::NativeHabit(io_print));
    map.insert("println".to_string(), JoeValue::NativeHabit(io_println));
    map.insert("eprintln".to_string(), JoeValue::NativeHabit(io_eprintln));
    map.insert("read_line".to_string(), JoeValue::NativeHabit(io_read_line));
    map.insert("clear_screen".to_string(), JoeValue::NativeHabit(io_clear_screen));
    make_bag(map)
}

fn io_print(args: &[JoeValue]) -> Result<JoeValue, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    io::stdout().flush().map_err(|e| e.to_string())?;
    Ok(JoeValue::Nothing)
}

fn io_println(args: &[JoeValue]) -> Result<JoeValue, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(JoeValue::Nothing)
}

fn io_eprintln(args: &[JoeValue]) -> Result<JoeValue, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            eprint!(" ");
        }
        eprint!("{}", arg);
    }
    eprintln!();
    Ok(JoeValue::Nothing)
}

fn io_read_line(_args: &[JoeValue]) -> Result<JoeValue, String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }
    Ok(JoeValue::Text(input))
}

fn io_clear_screen(_args: &[JoeValue]) -> Result<JoeValue, String> {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().map_err(|e| e.to_string())?;
    Ok(JoeValue::Nothing)
}
