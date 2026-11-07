use dumpster::unsync::Gc;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("float".to_string(), JoeValue::NativeHabit(rand_float));
    map.insert("int".to_string(), JoeValue::NativeHabit(rand_int));
    map.insert("choice".to_string(), JoeValue::NativeHabit(rand_choice));
    map.insert("shuffle".to_string(), JoeValue::NativeHabit(rand_shuffle));
    make_bag(map)
}

fn rand_float(_args: &[JoeValue]) -> Result<JoeValue, String> {
    let mut rng = rand::thread_rng();
    Ok(JoeValue::Number(rng.r#gen::<f64>()))
}

fn rand_int(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("random.int expects (min, max)".to_string());
    }
    let min = match args[0] {
        JoeValue::Number(n) => n as i64,
        _ => return Err("random.int expects integer min".to_string()),
    };
    let max = match args[1] {
        JoeValue::Number(n) => n as i64,
        _ => return Err("random.int expects integer max".to_string()),
    };
    if min > max {
        return Err("random.int min must be <= max".to_string());
    }
    let mut rng = rand::thread_rng();
    Ok(JoeValue::Number(rng.gen_range(min..=max) as f64))
}

fn rand_choice(args: &[JoeValue]) -> Result<JoeValue, String> {
    let list_rc = match args.first() {
        Some(JoeValue::List(l)) => l,
        _ => return Err("random.choice expects a list".to_string()),
    };
    let list = list_rc.borrow();
    if list.is_empty() {
        return Ok(JoeValue::Nothing);
    }
    let mut rng = rand::thread_rng();
    let choice = list.choose(&mut rng).cloned().unwrap_or(JoeValue::Nothing);
    Ok(choice)
}

fn rand_shuffle(args: &[JoeValue]) -> Result<JoeValue, String> {
    let list_rc = match args.first() {
        Some(JoeValue::List(l)) => l,
        _ => return Err("random.shuffle expects a list".to_string()),
    };
    let mut copy: Vec<JoeValue> = list_rc.borrow().clone();
    let mut rng = rand::thread_rng();
    copy.shuffle(&mut rng);
    Ok(JoeValue::List(Gc::new(RefCell::new(copy))))
}
