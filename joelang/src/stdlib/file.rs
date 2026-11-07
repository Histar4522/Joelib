use dumpster::unsync::Gc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("read".to_string(), JoeValue::NativeHabit(file_read));
    map.insert("read_lines".to_string(), JoeValue::NativeHabit(file_read_lines));
    map.insert("write".to_string(), JoeValue::NativeHabit(file_write));
    map.insert("append".to_string(), JoeValue::NativeHabit(file_append));
    map.insert("exists".to_string(), JoeValue::NativeHabit(file_exists));
    map.insert("remove".to_string(), JoeValue::NativeHabit(file_remove));
    map.insert("list_dir".to_string(), JoeValue::NativeHabit(file_list_dir));
    map.insert("make_dir".to_string(), JoeValue::NativeHabit(file_make_dir));
    make_bag(map)
}

fn file_read(args: &[JoeValue]) -> Result<JoeValue, String> {
    let path = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("file.read expects string path".to_string()),
    };
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
    Ok(JoeValue::Text(content))
}

fn file_read_lines(args: &[JoeValue]) -> Result<JoeValue, String> {
    let path = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("file.read_lines expects string path".to_string()),
    };
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
    let lines: Vec<JoeValue> = content.lines().map(|l| JoeValue::Text(l.to_string())).collect();
    Ok(JoeValue::List(Gc::new(RefCell::new(lines))))
}

fn file_write(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("file.write expects (path, content)".to_string());
    }
    let path = match &args[0] {
        JoeValue::Text(s) => s,
        _ => return Err("file.write expects string path as first argument".to_string()),
    };
    let content = format!("{}", args[1]);
    fs::write(path, content).map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
    Ok(JoeValue::Nothing)
}

fn file_append(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("file.append expects (path, content)".to_string());
    }
    let path = match &args[0] {
        JoeValue::Text(s) => s,
        _ => return Err("file.append expects string path as first argument".to_string()),
    };
    let content = format!("{}", args[1]);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open file '{}' for append: {}", path, e))?;
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(JoeValue::Nothing)
}

fn file_exists(args: &[JoeValue]) -> Result<JoeValue, String> {
    let path = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("file.exists expects string path".to_string()),
    };
    Ok(JoeValue::Agree(Path::new(path).exists()))
}

fn file_remove(args: &[JoeValue]) -> Result<JoeValue, String> {
    let path = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("file.remove expects string path".to_string()),
    };
    fs::remove_file(path).map_err(|e| format!("Failed to remove file '{}': {}", path, e))?;
    Ok(JoeValue::Nothing)
}

fn file_list_dir(args: &[JoeValue]) -> Result<JoeValue, String> {
    let path = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("file.list_dir expects string directory path".to_string()),
    };
    let entries = fs::read_dir(path).map_err(|e| format!("Failed to read dir '{}': {}", path, e))?;
    let mut list = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        list.push(JoeValue::Text(entry.file_name().to_string_lossy().to_string()));
    }
    Ok(JoeValue::List(Gc::new(RefCell::new(list))))
}

fn file_make_dir(args: &[JoeValue]) -> Result<JoeValue, String> {
    let path = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("file.make_dir expects string directory path".to_string()),
    };
    fs::create_dir_all(path).map_err(|e| format!("Failed to create directory '{}': {}", path, e))?;
    Ok(JoeValue::Nothing)
}
