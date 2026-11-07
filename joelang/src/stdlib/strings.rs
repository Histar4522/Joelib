use dumpster::unsync::Gc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("split".to_string(), JoeValue::NativeHabit(str_split));
    map.insert("join".to_string(), JoeValue::NativeHabit(str_join));
    map.insert("trim".to_string(), JoeValue::NativeHabit(str_trim));
    map.insert("to_upper".to_string(), JoeValue::NativeHabit(str_to_upper));
    map.insert("to_lower".to_string(), JoeValue::NativeHabit(str_to_lower));
    map.insert("replace".to_string(), JoeValue::NativeHabit(str_replace));
    map.insert("contains".to_string(), JoeValue::NativeHabit(str_contains));
    map.insert("starts_with".to_string(), JoeValue::NativeHabit(str_starts_with));
    map.insert("ends_with".to_string(), JoeValue::NativeHabit(str_ends_with));
    map.insert("index_of".to_string(), JoeValue::NativeHabit(str_index_of));
    map.insert("substring".to_string(), JoeValue::NativeHabit(str_substring));
    map.insert("pad_left".to_string(), JoeValue::NativeHabit(str_pad_left));
    map.insert("pad_right".to_string(), JoeValue::NativeHabit(str_pad_right));
    make_bag(map)
}

fn get_str(args: &[JoeValue], idx: usize, name: &str) -> Result<String, String> {
    match args.get(idx) {
        Some(JoeValue::Text(s)) => Ok(s.clone()),
        Some(other) => Ok(format!("{}", other)),
        None => Err(format!("{} missing string argument at index {}", name, idx)),
    }
}

fn str_split(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.split")?;
    let delim = get_str(args, 1, "strings.split")?;
    let parts: Vec<JoeValue> = s.split(&delim).map(|p| JoeValue::Text(p.to_string())).collect();
    Ok(JoeValue::List(Gc::new(RefCell::new(parts))))
}

fn str_join(args: &[JoeValue]) -> Result<JoeValue, String> {
    let list_rc = match args.first() {
        Some(JoeValue::List(l)) => l,
        _ => return Err("strings.join expects a list as first argument".to_string()),
    };
    let delim = get_str(args, 1, "strings.join").unwrap_or_default();
    let strings: Vec<String> = list_rc.borrow().iter().map(|item| format!("{}", item)).collect();
    Ok(JoeValue::Text(strings.join(&delim)))
}

fn str_trim(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.trim")?;
    Ok(JoeValue::Text(s.trim().to_string()))
}

fn str_to_upper(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.to_upper")?;
    Ok(JoeValue::Text(s.to_uppercase()))
}

fn str_to_lower(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.to_lower")?;
    Ok(JoeValue::Text(s.to_lowercase()))
}

fn str_replace(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.replace")?;
    let from = get_str(args, 1, "strings.replace")?;
    let to = get_str(args, 2, "strings.replace")?;
    Ok(JoeValue::Text(s.replace(&from, &to)))
}

fn str_contains(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.contains")?;
    let sub = get_str(args, 1, "strings.contains")?;
    Ok(JoeValue::Agree(s.contains(&sub)))
}

fn str_starts_with(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.starts_with")?;
    let pfx = get_str(args, 1, "strings.starts_with")?;
    Ok(JoeValue::Agree(s.starts_with(&pfx)))
}

fn str_ends_with(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.ends_with")?;
    let sfx = get_str(args, 1, "strings.ends_with")?;
    Ok(JoeValue::Agree(s.ends_with(&sfx)))
}

fn str_index_of(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.index_of")?;
    let sub = get_str(args, 1, "strings.index_of")?;
    match s.find(&sub) {
        Some(idx) => Ok(JoeValue::Number(idx as f64)),
        None => Ok(JoeValue::Number(-1.0)),
    }
}

fn str_substring(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.substring")?;
    let start = match args.get(1) {
        Some(JoeValue::Number(n)) => *n as usize,
        _ => return Err("strings.substring expects start index".to_string()),
    };
    let end = match args.get(2) {
        Some(JoeValue::Number(n)) => *n as usize,
        _ => s.len(),
    };
    let start_clamped = start.min(s.len());
    let end_clamped = end.min(s.len()).max(start_clamped);
    Ok(JoeValue::Text(s[start_clamped..end_clamped].to_string()))
}

fn str_pad_left(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.pad_left")?;
    let len = match args.get(1) {
        Some(JoeValue::Number(n)) => *n as usize,
        _ => return Err("strings.pad_left expects target length".to_string()),
    };
    let ch = get_str(args, 2, "strings.pad_left").unwrap_or_else(|_| " ".to_string());
    if s.len() >= len {
        return Ok(JoeValue::Text(s));
    }
    let pad_char = ch.chars().next().unwrap_or(' ');
    let padding: String = std::iter::repeat(pad_char).take(len - s.len()).collect();
    Ok(JoeValue::Text(format!("{}{}", padding, s)))
}

fn str_pad_right(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = get_str(args, 0, "strings.pad_right")?;
    let len = match args.get(1) {
        Some(JoeValue::Number(n)) => *n as usize,
        _ => return Err("strings.pad_right expects target length".to_string()),
    };
    let ch = get_str(args, 2, "strings.pad_right").unwrap_or_else(|_| " ".to_string());
    if s.len() >= len {
        return Ok(JoeValue::Text(s));
    }
    let pad_char = ch.chars().next().unwrap_or(' ');
    let padding: String = std::iter::repeat(pad_char).take(len - s.len()).collect();
    Ok(JoeValue::Text(format!("{}{}", s, padding)))
}
