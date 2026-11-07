use dumpster::unsync::Gc;
use dumpster::{TraceWith, Visitor};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use crate::ast::Stmt;
use crate::environment::Environment;

pub type NativeFn = fn(&[JoeValue]) -> Result<JoeValue, String>;

#[derive(Clone)]
pub enum JoeValue {
    Nothing,
    Agree(bool),
    Number(f64),
    Text(String),
    List(Gc<RefCell<Vec<JoeValue>>>),
    Bag(Gc<RefCell<HashMap<String, JoeValue>>>),
    Person(Gc<RefCell<JoeInstance>>),
    Habit(Gc<RefCell<JoeClosure>>),
    NativeHabit(NativeFn),
}

unsafe impl<V: Visitor> TraceWith<V> for JoeValue {
    fn accept(&self, visitor: &mut V) -> Result<(), ()> {
        match self {
            JoeValue::List(l) => TraceWith::accept(l, visitor),
            JoeValue::Bag(b) => TraceWith::accept(b, visitor),
            JoeValue::Person(p) => TraceWith::accept(p, visitor),
            JoeValue::Habit(h) => TraceWith::accept(h, visitor),
            _ => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct JoeInstance {
    pub class_name: String,
    pub fields: HashMap<String, JoeValue>,
    pub methods: HashMap<String, JoeClosure>,
}

unsafe impl<V: Visitor> TraceWith<V> for JoeInstance {
    fn accept(&self, visitor: &mut V) -> Result<(), ()> {
        for v in self.fields.values() {
            TraceWith::accept(v, visitor)?;
        }
        for m in self.methods.values() {
            TraceWith::accept(m, visitor)?;
        }
        Ok(())
    }
}

impl JoeInstance {
    pub fn new(class_name: String) -> Self {
        Self {
            class_name,
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct JoeClosure {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub env: Gc<RefCell<Environment>>,
}

unsafe impl<V: Visitor> TraceWith<V> for JoeClosure {
    fn accept(&self, visitor: &mut V) -> Result<(), ()> {
        TraceWith::accept(&self.env, visitor)
    }
}

impl JoeValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            JoeValue::Nothing => false,
            JoeValue::Agree(b) => *b,
            JoeValue::Number(n) => *n != 0.0,
            JoeValue::Text(s) => !s.is_empty(),
            JoeValue::List(l) => !l.borrow().is_empty(),
            JoeValue::Bag(b) => !b.borrow().is_empty(),
            JoeValue::Person(_) => true,
            JoeValue::Habit(_) => true,
            JoeValue::NativeHabit(_) => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            JoeValue::Nothing => "nothing",
            JoeValue::Agree(_) => "agreement",
            JoeValue::Number(_) => "number",
            JoeValue::Text(_) => "text",
            JoeValue::List(_) => "list",
            JoeValue::Bag(_) => "bag",
            JoeValue::Person(_) => "person",
            JoeValue::Habit(_) => "habit",
            JoeValue::NativeHabit(_) => "native_habit",
        }
    }
}

impl fmt::Display for JoeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoeValue::Nothing => write!(f, "nothing"),
            JoeValue::Agree(true) => write!(f, "Joe agrees"),
            JoeValue::Agree(false) => write!(f, "Joe disagrees"),
            JoeValue::Number(n) => {
                if n.fract() == 0.0 && !n.is_infinite() && !n.is_nan() && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            JoeValue::Text(s) => write!(f, "{}", s),
            JoeValue::List(l) => {
                write!(f, "[")?;
                let list = l.borrow();
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JoeValue::Bag(b) => {
                write!(f, "{{")?;
                let bag = b.borrow();
                let mut first = true;
                for (k, v) in bag.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                    first = false;
                }
                write!(f, "}}")
            }
            JoeValue::Person(p) => {
                let inst = p.borrow();
                write!(f, "<{} instance>", inst.class_name)
            }
            JoeValue::Habit(h) => {
                let closure = h.borrow();
                write!(f, "<habit {}>", closure.name)
            }
            JoeValue::NativeHabit(_) => write!(f, "<native habit>"),
        }
    }
}

impl fmt::Debug for JoeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl PartialEq for JoeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JoeValue::Nothing, JoeValue::Nothing) => true,
            (JoeValue::Agree(a), JoeValue::Agree(b)) => a == b,
            (JoeValue::Number(a), JoeValue::Number(b)) => (a - b).abs() < f64::EPSILON,
            (JoeValue::Text(a), JoeValue::Text(b)) => a == b,
            (JoeValue::List(a), JoeValue::List(b)) => Gc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (JoeValue::Bag(a), JoeValue::Bag(b)) => Gc::ptr_eq(a, b),
            (JoeValue::Person(a), JoeValue::Person(b)) => Gc::ptr_eq(a, b),
            (JoeValue::Habit(a), JoeValue::Habit(b)) => Gc::ptr_eq(a, b),
            _ => false,
        }
    }
}
