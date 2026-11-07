use dumpster::unsync::Gc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::process;
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("args".to_string(), JoeValue::NativeHabit(os_args));
    map.insert("get_env".to_string(), JoeValue::NativeHabit(os_get_env));
    map.insert("set_env".to_string(), JoeValue::NativeHabit(os_set_env));
    map.insert("exit".to_string(), JoeValue::NativeHabit(os_exit));
    make_bag(map)
}

fn os_args(_args: &[JoeValue]) -> Result<JoeValue, String> {
    let args: Vec<JoeValue> = env::args().map(JoeValue::Text).collect();
    Ok(JoeValue::List(Gc::new(RefCell::new(args))))
}

fn os_get_env(args: &[JoeValue]) -> Result<JoeValue, String> {
    let name = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("os.get_env expects environment variable name".to_string()),
    };
    match env::var(name) {
        Ok(v) => Ok(JoeValue::Text(v)),
        Err(_) => Ok(JoeValue::Nothing),
    }
}

fn os_set_env(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("os.set_env expects (name, value)".to_string());
    }
    let name = format!("{}", args[0]);
    let val = format!("{}", args[1]);
    unsafe {
        env::set_var(name, val);
    }
    Ok(JoeValue::Nothing)
}

fn os_exit(args: &[JoeValue]) -> Result<JoeValue, String> {
    let code = match args.first() {
        Some(JoeValue::Number(n)) => *n as i32,
        _ => 0,
    };
    process::exit(code);
}
