pub mod core;
pub mod io;
pub mod file;
pub mod net;
pub mod math;
pub mod random;
pub mod strings;
pub mod collections;
pub mod json;
pub mod time;
pub mod os;

use dumpster::unsync::Gc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::value::JoeValue;

pub fn load_module(name: &str) -> Option<JoeValue> {
    match name {
        "std.io" => Some(io::load()),
        "std.file" => Some(file::load()),
        "std.net" => Some(net::load()),
        "std.math" => Some(math::load()),
        "std.random" => Some(random::load()),
        "std.strings" => Some(strings::load()),
        "std.collections" => Some(collections::load()),
        "std.json" => Some(json::load()),
        "std.time" => Some(time::load()),
        "std.os" => Some(os::load()),
        _ => None,
    }
}

pub fn make_bag(map: HashMap<String, JoeValue>) -> JoeValue {
    JoeValue::Bag(Gc::new(RefCell::new(map)))
}
