use dumpster::unsync::Gc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::stdlib::make_bag;
use crate::value::JoeValue;

pub fn load() -> JoeValue {
    let mut map = HashMap::new();
    map.insert("fetch".to_string(), JoeValue::NativeHabit(net_fetch));
    map.insert("get".to_string(), JoeValue::NativeHabit(net_get));
    map.insert("post".to_string(), JoeValue::NativeHabit(net_post));
    map.insert("put".to_string(), JoeValue::NativeHabit(net_put));
    map.insert("delete".to_string(), JoeValue::NativeHabit(net_delete));
    make_bag(map)
}

fn create_response_bag(status: u16, body: String, headers_map: HashMap<String, JoeValue>) -> JoeValue {
    let mut map = HashMap::new();
    map.insert("status".to_string(), JoeValue::Number(status as f64));
    map.insert("ok".to_string(), JoeValue::Agree((200..=299).contains(&status)));
    map.insert("body".to_string(), JoeValue::Text(body));
    map.insert("headers".to_string(), JoeValue::Bag(Gc::new(RefCell::new(headers_map))));
    make_bag(map)
}

fn net_fetch(args: &[JoeValue]) -> Result<JoeValue, String> {
    let url = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("net.fetch expects a URL string".to_string()),
    };

    let response = match ureq::get(url).call() {
        Ok(res) => res,
        Err(ureq::Error::Status(code, res)) => {
            let body = res.into_string().unwrap_or_default();
            return Ok(create_response_bag(code, body, HashMap::new()));
        }
        Err(e) => return Err(format!("Network fetch error: {}", e)),
    };

    let status = response.status();
    let mut headers = HashMap::new();
    for name in response.headers_names() {
        if let Some(val) = response.header(&name) {
            headers.insert(name, JoeValue::Text(val.to_string()));
        }
    }
    let body = response.into_string().map_err(|e| e.to_string())?;

    Ok(create_response_bag(status, body, headers))
}

fn net_get(args: &[JoeValue]) -> Result<JoeValue, String> {
    let url = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("net.get expects a URL string".to_string()),
    };

    let mut req = ureq::get(url);
    if let Some(JoeValue::Bag(b)) = args.get(1) {
        for (k, v) in b.borrow().iter() {
            req = req.set(k, &format!("{}", v));
        }
    }

    let response = match req.call() {
        Ok(res) => res,
        Err(ureq::Error::Status(code, res)) => {
            let body = res.into_string().unwrap_or_default();
            return Ok(create_response_bag(code, body, HashMap::new()));
        }
        Err(e) => return Err(format!("Network get error: {}", e)),
    };

    let status = response.status();
    let body = response.into_string().map_err(|e| e.to_string())?;
    Ok(create_response_bag(status, body, HashMap::new()))
}

fn net_post(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("net.post expects (url, body, [headers])".to_string());
    }
    let url = match &args[0] {
        JoeValue::Text(s) => s,
        _ => return Err("net.post expects a URL string as first argument".to_string()),
    };
    let body_str = format!("{}", args[1]);

    let mut req = ureq::post(url);
    if let Some(JoeValue::Bag(b)) = args.get(2) {
        for (k, v) in b.borrow().iter() {
            req = req.set(k, &format!("{}", v));
        }
    }

    let response = match req.send_string(&body_str) {
        Ok(res) => res,
        Err(ureq::Error::Status(code, res)) => {
            let body = res.into_string().unwrap_or_default();
            return Ok(create_response_bag(code, body, HashMap::new()));
        }
        Err(e) => return Err(format!("Network post error: {}", e)),
    };

    let status = response.status();
    let body = response.into_string().map_err(|e| e.to_string())?;
    Ok(create_response_bag(status, body, HashMap::new()))
}

fn net_put(args: &[JoeValue]) -> Result<JoeValue, String> {
    if args.len() < 2 {
        return Err("net.put expects (url, body, [headers])".to_string());
    }
    let url = match &args[0] {
        JoeValue::Text(s) => s,
        _ => return Err("net.put expects a URL string as first argument".to_string()),
    };
    let body_str = format!("{}", args[1]);

    let mut req = ureq::put(url);
    if let Some(JoeValue::Bag(b)) = args.get(2) {
        for (k, v) in b.borrow().iter() {
            req = req.set(k, &format!("{}", v));
        }
    }

    let response = match req.send_string(&body_str) {
        Ok(res) => res,
        Err(ureq::Error::Status(code, res)) => {
            let body = res.into_string().unwrap_or_default();
            return Ok(create_response_bag(code, body, HashMap::new()));
        }
        Err(e) => return Err(format!("Network put error: {}", e)),
    };

    let status = response.status();
    let body = response.into_string().map_err(|e| e.to_string())?;
    Ok(create_response_bag(status, body, HashMap::new()))
}

fn net_delete(args: &[JoeValue]) -> Result<JoeValue, String> {
    let url = match args.first() {
        Some(JoeValue::Text(s)) => s,
        _ => return Err("net.delete expects a URL string".to_string()),
    };

    let mut req = ureq::delete(url);
    if let Some(JoeValue::Bag(b)) = args.get(1) {
        for (k, v) in b.borrow().iter() {
            req = req.set(k, &format!("{}", v));
        }
    }

    let response = match req.call() {
        Ok(res) => res,
        Err(ureq::Error::Status(code, res)) => {
            let body = res.into_string().unwrap_or_default();
            return Ok(create_response_bag(code, body, HashMap::new()));
        }
        Err(e) => return Err(format!("Network delete error: {}", e)),
    };

    let status = response.status();
    let body = response.into_string().map_err(|e| e.to_string())?;
    Ok(create_response_bag(status, body, HashMap::new()))
}
