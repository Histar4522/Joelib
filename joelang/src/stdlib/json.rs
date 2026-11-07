use dumpster::unsync::Gc;
use serde_json::Value as JsonValue;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("parse".to_string(), JoeValue::NativeHabit(json_parse));
    map.insert("stringify".to_string(), JoeValue::NativeHabit(json_stringify));
    map.insert("pretty".to_string(), JoeValue::NativeHabit(json_pretty));
    make_bag(map)
}

fn json_to_joe(json: JsonValue) -> JoeValue {
    match json {
        JsonValue::Null => JoeValue::Nothing,
        JsonValue::Bool(b) => JoeValue::Agree(b),
        JsonValue::Number(n) => JoeValue::Number(n.as_f64().unwrap_or(0.0)),
        JsonValue::String(s) => JoeValue::Text(s),
        JsonValue::Array(arr) => {
            let list: Vec<JoeValue> = arr.into_iter().map(json_to_joe).collect();
            JoeValue::List(Gc::new(RefCell::new(list)))
        }
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_joe(v));
            }
            JoeValue::Bag(Gc::new(RefCell::new(map)))
        }
    }
}

fn joe_to_json(val: &JoeValue, depth: usize) -> Result<JsonValue, String> {
    if depth > 50 {
        return Err("JSON serialization depth limit exceeded (possible cycle in Joe's thoughts)".to_string());
    }
    match val {
        JoeValue::Nothing => Ok(JsonValue::Null),
        JoeValue::Agree(b) => Ok(JsonValue::Bool(*b)),
        JoeValue::Number(n) => {
            if let Some(num) = serde_json::Number::from_f64(*n) {
                Ok(JsonValue::Number(num))
            } else {
                Ok(JsonValue::Null)
            }
        }
        JoeValue::Text(s) => Ok(JsonValue::String(s.clone())),
        JoeValue::List(l) => {
            let mut arr = Vec::new();
            for item in l.borrow().iter() {
                arr.push(joe_to_json(item, depth + 1)?);
            }
            Ok(JsonValue::Array(arr))
        }
        JoeValue::Bag(b) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in b.borrow().iter() {
                obj.insert(k.clone(), joe_to_json(v, depth + 1)?);
            }
            Ok(JsonValue::Object(obj))
        }
        JoeValue::Person(p) => {
            let mut obj = serde_json::Map::new();
            let inst = p.borrow();
            obj.insert("__class__".to_string(), JsonValue::String(inst.class_name.clone()));
            for (k, v) in inst.fields.iter() {
                obj.insert(k.clone(), joe_to_json(v, depth + 1)?);
            }
            Ok(JsonValue::Object(obj))
        }
        JoeValue::Habit(h) => Ok(JsonValue::String(format!("<habit {}>", h.borrow().name))),
        JoeValue::NativeHabit(_) => Ok(JsonValue::String("<native habit>".to_string())),
    }
}

fn json_parse(args: &[JoeValue]) -> Result<JoeValue, String> {
    let s = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("json.parse expects JSON string".to_string()),
    };
    let json: JsonValue = serde_json::from_str(s).map_err(|e| format!("Invalid JSON: {}", e))?;
    Ok(json_to_joe(json))
}

fn json_stringify(args: &[JoeValue]) -> Result<JoeValue, String> {
    let val = match args.first() {
        Some(v) => v,
        None => return Err("json.stringify expects a value".to_string()),
    };
    let json = joe_to_json(val, 0)?;
    Ok(JoeValue::Text(json.to_string()))
}

fn json_pretty(args: &[JoeValue]) -> Result<JoeValue, String> {
    let val = match args.first() {
        Some(v) => v,
        None => return Err("json.pretty expects a value".to_string()),
    };
    let json = joe_to_json(val, 0)?;
    serde_json::to_string_pretty(&json)
        .map(JoeValue::Text)
        .map_err(|e| e.to_string())
}
