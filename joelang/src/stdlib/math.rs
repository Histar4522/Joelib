use std::collections::HashMap;
use std::f64::consts::{E, PI};
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("pi".to_string(), JoeValue::Number(PI));
    map.insert("e".to_string(), JoeValue::Number(E));
    map.insert("infinity".to_string(), JoeValue::Number(f64::INFINITY));
    map.insert("nan".to_string(), JoeValue::Number(f64::NAN));

    map.insert("abs".to_string(), JoeValue::NativeHabit(math_abs));
    map.insert("floor".to_string(), JoeValue::NativeHabit(math_floor));
    map.insert("ceil".to_string(), JoeValue::NativeHabit(math_ceil));
    map.insert("round".to_string(), JoeValue::NativeHabit(math_round));
    map.insert("sqrt".to_string(), JoeValue::NativeHabit(math_sqrt));
    map.insert("pow".to_string(), JoeValue::NativeHabit(math_pow));
    map.insert("exp".to_string(), JoeValue::NativeHabit(math_exp));
    map.insert("ln".to_string(), JoeValue::NativeHabit(math_ln));
    map.insert("log10".to_string(), JoeValue::NativeHabit(math_log10));
    map.insert("log2".to_string(), JoeValue::NativeHabit(math_log2));
    map.insert("sin".to_string(), JoeValue::NativeHabit(math_sin));
    map.insert("cos".to_string(), JoeValue::NativeHabit(math_cos));
    map.insert("tan".to_string(), JoeValue::NativeHabit(math_tan));
    map.insert("asin".to_string(), JoeValue::NativeHabit(math_asin));
    map.insert("acos".to_string(), JoeValue::NativeHabit(math_acos));
    map.insert("atan".to_string(), JoeValue::NativeHabit(math_atan));
    map.insert("atan2".to_string(), JoeValue::NativeHabit(math_atan2));
    map.insert("min".to_string(), JoeValue::NativeHabit(math_min));
    map.insert("max".to_string(), JoeValue::NativeHabit(math_max));
    map.insert("clamp".to_string(), JoeValue::NativeHabit(math_clamp));
    make_bag(map)
}

fn get_num(args: &[JoeValue], idx: usize, name: &str) -> Result<f64, String> {
    match args.get(idx) {
        Some(JoeValue::Number(n)) => Ok(*n),
        Some(other) => Err(format!("{} expects number, got {}", name, other.type_name())),
        None => Err(format!("{} missing argument at index {}", name, idx)),
    }
}

fn math_abs(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.abs")?.abs()))
}

fn math_floor(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.floor")?.floor()))
}

fn math_ceil(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.ceil")?.ceil()))
}

fn math_round(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.round")?.round()))
}

fn math_sqrt(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.sqrt")?.sqrt()))
}

fn math_pow(args: &[JoeValue]) -> Result<JoeValue, String> {
    let base = get_num(args, 0, "math.pow")?;
    let exp = get_num(args, 1, "math.pow")?;
    Ok(JoeValue::Number(base.powf(exp)))
}

fn math_exp(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.exp")?.exp()))
}

fn math_ln(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.ln")?.ln()))
}

fn math_log10(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.log10")?.log10()))
}

fn math_log2(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.log2")?.log2()))
}

fn math_sin(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.sin")?.sin()))
}

fn math_cos(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.cos")?.cos()))
}

fn math_tan(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.tan")?.tan()))
}

fn math_asin(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.asin")?.asin()))
}

fn math_acos(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.acos")?.acos()))
}

fn math_atan(args: &[JoeValue]) -> Result<JoeValue, String> {
    Ok(JoeValue::Number(get_num(args, 0, "math.atan")?.atan()))
}

fn math_atan2(args: &[JoeValue]) -> Result<JoeValue, String> {
    let y = get_num(args, 0, "math.atan2")?;
    let x = get_num(args, 1, "math.atan2")?;
    Ok(JoeValue::Number(y.atan2(x)))
}

fn math_min(args: &[JoeValue]) -> Result<JoeValue, String> {
    let a = get_num(args, 0, "math.min")?;
    let b = get_num(args, 1, "math.min")?;
    Ok(JoeValue::Number(a.min(b)))
}

fn math_max(args: &[JoeValue]) -> Result<JoeValue, String> {
    let a = get_num(args, 0, "math.max")?;
    let b = get_num(args, 1, "math.max")?;
    Ok(JoeValue::Number(a.max(b)))
}

fn math_clamp(args: &[JoeValue]) -> Result<JoeValue, String> {
    let val = get_num(args, 0, "math.clamp")?;
    let min = get_num(args, 1, "math.clamp")?;
    let max = get_num(args, 2, "math.clamp")?;
    Ok(JoeValue::Number(val.clamp(min, max)))
}
