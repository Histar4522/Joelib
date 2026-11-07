use dumpster::unsync::Gc;
use dumpster::{TraceWith, Visitor};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::value::JoeValue;

#[derive(Clone)]
pub struct Environment {
    pub values: HashMap<String, JoeValue>,
    pub enclosing: Option<Gc<RefCell<Environment>>>,
}

unsafe impl<V: Visitor> TraceWith<V> for Environment {
    fn accept(&self, visitor: &mut V) -> Result<(), ()> {
        for v in self.values.values() {
            TraceWith::accept(v, visitor)?;
        }
        if let Some(ref enc) = self.enclosing {
            TraceWith::accept(enc, visitor)?;
        }
        Ok(())
    }
}

impl Environment {
    pub fn new() -> Gc<RefCell<Self>> {
        Gc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn with_enclosing(parent: Gc<RefCell<Environment>>) -> Gc<RefCell<Self>> {
        Gc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(parent),
        }))
    }

    pub fn define(&mut self, name: &str, value: JoeValue) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<JoeValue> {
        if let Some(val) = self.values.get(name) {
            Some(val.clone())
        } else if let Some(ref parent) = self.enclosing {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: &str, value: JoeValue) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else if let Some(ref parent) = self.enclosing {
            parent.borrow_mut().assign(name, value)
        } else {
            false
        }
    }

    pub fn all_locals(&self) -> Vec<(String, JoeValue)> {
        self.values.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}
