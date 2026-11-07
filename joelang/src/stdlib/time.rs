use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("now".to_string(), JoeValue::NativeHabit(time_now));
    map.insert("now_ms".to_string(), JoeValue::NativeHabit(time_now_ms));
    map.insert("nap".to_string(), JoeValue::NativeHabit(time_nap));
    make_bag(map)
}

fn time_now(_args: &[JoeValue]) -> Result<JoeValue, String> {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    Ok(JoeValue::Number(dur.as_secs_f64()))
}

fn time_now_ms(_args: &[JoeValue]) -> Result<JoeValue, String> {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    Ok(JoeValue::Number(dur.as_millis() as f64))
}

fn time_nap(args: &[JoeValue]) -> Result<JoeValue, String> {
    let ms = match args.first() {
        Some(JoeValue::Number(n)) => *n as u64,
        _ => return Err("time.nap expects milliseconds as number".to_string()),
    };
    thread::sleep(Duration::from_millis(ms));
    Ok(JoeValue::Nothing)
}
