use dumpster::unsync::Gc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("push".to_string(), JoeValue::NativeHabit(coll_push));
    map.insert("pop".to_string(), JoeValue::NativeHabit(coll_pop));
    map.insert("insert".to_string(), JoeValue::NativeHabit(coll_insert));
    map.insert("remove_at".to_string(), JoeValue::NativeHabit(coll_remove_at));
    map.insert("slice".to_string(), JoeValue::NativeHabit(coll_slice));
    map.insert("reverse".to_string(), JoeValue::NativeHabit(coll_reverse));
    map.insert("sort".to_string(), JoeValue::NativeHabit(coll_sort));

    map.insert("keys".to_string(), JoeValue::NativeHabit(coll_keys));
    map.insert("values".to_string(), JoeValue::NativeHabit(coll_values));
    map.insert("entries".to_string(), JoeValue::NativeHabit(coll_entries));
    map.insert("has_key".to_string(), JoeValue::NativeHabit(coll_has_key));
    map.insert("remove_key".to_string(), JoeValue::NativeHabit(coll_remove_key));
    map.insert("merge".to_string(), JoeValue::NativeHabit(coll_merge));
    make_bag(map)
}

fn coll_push(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("collections.push expects (list, item)".to_string());
    }
    match &args[0] {
        JoeValue::List(l) => {
            l.borrow_mut().push(args[1].clone());
            Ok(JoeValue::Nothing)
        }
        other => Err(format!("collections.push expects list, got {}", other.type_name())),
    }
}

fn coll_pop(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(JoeValue::List(l)) => Ok(l.borrow_mut().pop().unwrap_or(JoeValue::Nothing)),
        _ => Err("collections.pop expects a list".to_string()),
    }
}

fn coll_insert(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 3 {
        return Err("collections.insert expects (list, index, item)".to_string());
    }
    let idx = match args[1] {
        JoeValue::Number(n) => n as usize,
        _ => return Err("collections.insert expects numeric index".to_string()),
    };
    match &args[0] {
        JoeValue::List(l) => {
            let mut list = l.borrow_mut();
            let safe_idx = idx.min(list.len());
            list.insert(safe_idx, args[2].clone());
            Ok(JoeValue::Nothing)
        }
        _ => Err("collections.insert expects a list".to_string()),
    }
}

fn coll_remove_at(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("collections.remove_at expects (list, index)".to_string());
    }
    let idx = match args[1] {
        JoeValue::Number(n) => n as usize,
        _ => return Err("collections.remove_at expects numeric index".to_string()),
    };
    match &args[0] {
        JoeValue::List(l) => {
            let mut list = l.borrow_mut();
            if idx < list.len() {
                Ok(list.remove(idx))
            } else {
                Ok(JoeValue::Nothing)
            }
        }
        _ => Err("collections.remove_at expects a list".to_string()),
    }
}

fn coll_slice(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 3 {
        return Err("collections.slice expects (list, start, end)".to_string());
    }
    let start = match args[1] {
        JoeValue::Number(n) => n as usize,
        _ => return Err("collections.slice expects numeric start index".to_string()),
    };
    let end = match args[2] {
        JoeValue::Number(n) => n as usize,
        _ => return Err("collections.slice expects numeric end index".to_string()),
    };
    match &args[0] {
        JoeValue::List(l) => {
            let list = l.borrow();
            let s = start.min(list.len());
            let e = end.min(list.len()).max(s);
            let sub: Vec<JoeValue> = list[s..e].to_vec();
            Ok(JoeValue::List(Gc::new(RefCell::new(sub))))
        }
        _ => Err("collections.slice expects a list".to_string()),
    }
}

fn coll_reverse(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(JoeValue::List(l)) => {
            let mut sub = l.borrow().clone();
            sub.reverse();
            Ok(JoeValue::List(Gc::new(RefCell::new(sub))))
        }
        _ => Err("collections.reverse expects a list".to_string()),
    }
}

fn coll_sort(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(JoeValue::List(l)) => {
            let mut sub = l.borrow().clone();
            sub.sort_by(|a, b| match (a, b) {
                (JoeValue::Number(x), JoeValue::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                (JoeValue::Text(x), JoeValue::Text(y)) => x.cmp(y),
                _ => format!("{}", a).cmp(&format!("{}", b)),
            });
            Ok(JoeValue::List(Gc::new(RefCell::new(sub))))
        }
        _ => Err("collections.sort expects a list".to_string()),
    }
}

fn coll_keys(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(JoeValue::Bag(b)) => {
            let keys: Vec<JoeValue> = b.borrow().keys().map(|k| JoeValue::Text(k.clone())).collect();
            Ok(JoeValue::List(Gc::new(RefCell::new(keys))))
        }
        _ => Err("collections.keys expects a bag".to_string()),
    }
}

fn coll_values(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(JoeValue::Bag(b)) => {
            let values: Vec<JoeValue> = b.borrow().values().cloned().collect();
            Ok(JoeValue::List(Gc::new(RefCell::new(values))))
        }
        _ => Err("collections.values expects a bag".to_string()),
    }
}

fn coll_entries(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(JoeValue::Bag(b)) => {
            let mut entries = Vec::new();
            for (k, v) in b.borrow().iter() {
                let pair = vec![JoeValue::Text(k.clone()), v.clone()];
                entries.push(JoeValue::List(Gc::new(RefCell::new(pair))));
            }
            Ok(JoeValue::List(Gc::new(RefCell::new(entries))))
        }
        _ => Err("collections.entries expects a bag".to_string()),
    }
}

fn coll_has_key(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("collections.has_key expects (bag, key)".to_string());
    }
    let key = format!("{}", args[1]);
    match &args[0] {
        JoeValue::Bag(b) => Ok(JoeValue::Agree(b.borrow().contains_key(&key))),
        _ => Err("collections.has_key expects a bag".to_string()),
    }
}

fn coll_remove_key(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("collections.remove_key expects (bag, key)".to_string());
    }
    let key = format!("{}", args[1]);
    match &args[0] {
        JoeValue::Bag(b) => Ok(b.borrow_mut().remove(&key).unwrap_or(JoeValue::Nothing)),
        _ => Err("collections.remove_key expects a bag".to_string()),
    }
}

fn coll_merge(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("collections.merge expects (bag1, bag2)".to_string());
    }
    match (&args[0], &args[1]) {
        (JoeValue::Bag(b1), JoeValue::Bag(b2)) => {
            let mut merged = b1.borrow().clone();
            for (k, v) in b2.borrow().iter() {
                merged.insert(k.clone(), v.clone());
            }
            Ok(JoeValue::Bag(Gc::new(RefCell::new(merged))))
        }
        _ => Err("collections.merge expects two bags".to_string()),
    }
}
