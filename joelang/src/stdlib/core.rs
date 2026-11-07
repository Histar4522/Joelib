use crate::value::JoeValue;

pub fn native_len(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(JoeValue::Text(s)) => Ok(JoeValue::Number(s.len() as f64)),
        Some(JoeValue::List(l)) => Ok(JoeValue::Number(l.borrow().len() as f64)),
        Some(JoeValue::Bag(b)) => Ok(JoeValue::Number(b.borrow().len() as f64)),
        Some(other) => Err(format!("'len' cannot be calculated for type '{}'", other.type_name())),
        None => Err("Expected 1 argument for 'len'".to_string()),
    }
}

pub fn native_type(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(val) => Ok(JoeValue::Text(val.type_name().to_string())),
        None => Err("Expected 1 argument for 'type'".to_string()),
    }
}

pub fn native_str(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(val) => Ok(JoeValue::Text(format!("{}", val))),
        None => Err("Expected 1 argument for 'str'".to_string()),
    }
}

pub fn native_num(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(JoeValue::Number(n)) => Ok(JoeValue::Number(*n)),
        Some(JoeValue::Agree(b)) => Ok(JoeValue::Number(if *b { 1.0 } else { 0.0 })),
        Some(JoeValue::Text(s)) => s
            .trim()
            .parse::<f64>()
            .map(JoeValue::Number)
            .map_err(|e| format!("Cannot parse '{}' to number: {}", s, e)),
        Some(other) => Err(format!("Cannot convert '{}' to number", other.type_name())),
        None => Err("Expected 1 argument for 'num'".to_string()),
    }
}

pub fn native_bool(args: &[JoeValue]) -> Result<JoeValue, String> {
    match args.first() {
        Some(val) => Ok(JoeValue::Agree(val.is_truthy())),
        None => Err("Expected 1 argument for 'bool'".to_string()),
    }
}

pub fn native_panic(args: &[JoeValue]) -> Result<JoeValue, String> {
    let msg = match args.first() {
        Some(v) => format!("{}", v),
        None => "Joe panicked with no explanation.".to_string(),
    };
    Err(format!("Joe panicked: {}", msg))
}
