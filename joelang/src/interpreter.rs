use dumpster::unsync::Gc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use crate::ast::*;
use crate::environment::Environment;
use crate::stdlib;
use crate::value::{JoeClosure, JoeInstance, JoeValue};

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    None,
    Return(Option<JoeValue>),
    Break,
    Continue,
}

pub struct Interpreter {
    pub env: Gc<RefCell<Environment>>,
    pub global_env: Gc<RefCell<Environment>>,
    pub classes: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    pub output_capture: Option<Vec<String>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Environment::new();

        // Register subconscious (prelude)
        global.borrow_mut().define("len", JoeValue::NativeHabit(stdlib::core::native_len));
        global.borrow_mut().define("type", JoeValue::NativeHabit(stdlib::core::native_type));
        global.borrow_mut().define("str", JoeValue::NativeHabit(stdlib::core::native_str));
        global.borrow_mut().define("num", JoeValue::NativeHabit(stdlib::core::native_num));
        global.borrow_mut().define("bool", JoeValue::NativeHabit(stdlib::core::native_bool));
        global.borrow_mut().define("panic", JoeValue::NativeHabit(stdlib::core::native_panic));

        // Reflect (confirms dumpster cycle-tracking GC is active)
        global.borrow_mut().define("reflect", JoeValue::NativeHabit(|_| {
            Ok(JoeValue::Agree(true))
        }));

        Self {
            env: global.clone(),
            global_env: global,
            classes: HashMap::new(),
            output_capture: None,
        }
    }

    pub fn with_capture() -> Self {
        let mut interp = Self::new();
        interp.output_capture = Some(Vec::new());
        interp
    }

    pub fn output(&self) -> String {
        if let Some(ref lines) = self.output_capture {
            lines.join("\n")
        } else {
            String::new()
        }
    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<JoeValue, String> {
        let mut last_val = JoeValue::Nothing;
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::Expr(expr) => {
                    last_val = self.evaluate_expr(expr)?;
                }
                _ => {
                    let ctrl = self.execute_stmt(stmt)?;
                    match ctrl {
                        Control::Return(Some(v)) => return Ok(v),
                        Control::Return(None) => return Ok(JoeValue::Nothing),
                        Control::Break => return Err(format!("'Joe gives up.' outside of a loop at {}", stmt.span)),
                        Control::Continue => return Err(format!("'Joe tries anyway.' outside of a loop at {}", stmt.span)),
                        Control::None => {}
                    }
                }
            }
        }
        Ok(last_val)
    }

    pub fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Control, String> {
        match &stmt.kind {
            StmtKind::Remember { name, init } => {
                let val = self.evaluate_expr(init)?;
                self.env.borrow_mut().define(name, val);
                Ok(Control::None)
            }
            StmtKind::Assign { target, value } => {
                let val = self.evaluate_expr(value)?;
                match &target.kind {
                    ExprKind::Ident(name) => {
                        if !self.env.borrow_mut().assign(name, val.clone()) {
                            return Err(format!("Joe has no memory of '{}' at {}", name, target.span));
                        }
                    }
                    ExprKind::GetField { object, name } => {
                        let obj = self.evaluate_expr(object)?;
                        match obj {
                            JoeValue::Person(p) => {
                                p.borrow_mut().fields.insert(name.clone(), val);
                            }
                            JoeValue::Bag(b) => {
                                b.borrow_mut().insert(name.clone(), val);
                            }
                            other => return Err(format!("Cannot set field '{}' on '{}' at {}", name, other.type_name(), target.span)),
                        }
                    }
                    ExprKind::GetIndex { object, index } => {
                        let obj = self.evaluate_expr(object)?;
                        let idx_val = self.evaluate_expr(index)?;
                        match obj {
                            JoeValue::List(l) => {
                                let idx = match idx_val {
                                    JoeValue::Number(n) => n as usize,
                                    _ => return Err(format!("List index must be numeric at {}", index.span)),
                                };
                                let mut list = l.borrow_mut();
                                if idx < list.len() {
                                    list[idx] = val;
                                } else {
                                    return Err(format!("List index {} out of bounds (len: {}) at {}", idx, list.len(), index.span));
                                }
                            }
                            JoeValue::Bag(b) => {
                                let key = format!("{}", idx_val);
                                b.borrow_mut().insert(key, val);
                            }
                            other => return Err(format!("Cannot index-assign into '{}' at {}", other.type_name(), target.span)),
                        }
                    }
                    _ => return Err(format!("Invalid assignment target at {}", target.span)),
                }
                Ok(Control::None)
            }
            StmtKind::Yell { expr, to } => {
                let val = self.evaluate_expr(expr)?;
                if let Some(to_expr) = to {
                    let target = self.evaluate_expr(to_expr)?;
                    self.dispatch_keyword_override(&target, "yell", &[val.clone()], &format!("{}\n", val), stmt.span)?;
                } else {
                    let line = format!("{}", val);
                    if let Some(ref mut capture) = self.output_capture {
                        capture.push(line);
                    } else {
                        println!("{}", line);
                    }
                }
                Ok(Control::None)
            }
            StmtKind::Whisper { expr, to } => {
                let val = self.evaluate_expr(expr)?;
                if let Some(to_expr) = to {
                    let target = self.evaluate_expr(to_expr)?;
                    self.dispatch_keyword_override(&target, "whisper", &[val.clone()], &format!("{}", val), stmt.span)?;
                } else {
                    let text = format!("{}", val);
                    if let Some(ref mut capture) = self.output_capture {
                        if let Some(last) = capture.last_mut() {
                            last.push_str(&text);
                        } else {
                            capture.push(text);
                        }
                    } else {
                        print!("{}", text);
                    }
                }
                Ok(Control::None)
            }
            StmtKind::Listen { name, from } => {
                let text = if let Some(from_expr) = from {
                    let source = self.evaluate_expr(from_expr)?;
                    match source {
                        JoeValue::Person(p) => {
                            if p.borrow().methods.contains_key("listen") {
                                let m = p.borrow().methods.get("listen").unwrap().clone();
                                let method_env = Environment::with_enclosing(m.env.clone());
                                method_env.borrow_mut().define("jthis", JoeValue::Person(p.clone()));
                                let habit = JoeValue::Habit(Gc::new(RefCell::new(JoeClosure {
                                    name: m.name,
                                    params: m.params,
                                    body: m.body,
                                    env: method_env,
                                })));
                                let ret = self.call_value(&habit, &[], stmt.span)?;
                                format!("{}", ret)
                            } else {
                                return Err(format!("Person does not override 'listen' at {}", stmt.span));
                            }
                        }
                        JoeValue::Text(path) => {
                            fs::read_to_string(&path)
                                .map_err(|e| format!("Failed to listen from file '{}': {}", path, e))?
                        }
                        other => return Err(format!("Cannot listen from '{}' at {}", other.type_name(), stmt.span)),
                    }
                } else {
                    let stdin = io::stdin();
                    let mut line = String::new();
                    let _ = stdin.lock().read_line(&mut line);
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    line
                };
                self.env.borrow_mut().define(name, JoeValue::Text(text));
                Ok(Control::None)
            }
            StmtKind::If { cond, then_branch, else_branch } => {
                let condition = self.evaluate_expr(cond)?;
                if condition.is_truthy() {
                    self.execute_block(then_branch)
                } else if let Some(elses) = else_branch {
                    self.execute_block(elses)
                } else {
                    Ok(Control::None)
                }
            }
            StmtKind::While { cond, body } => {
                while self.evaluate_expr(cond)?.is_truthy() {
                    let ctrl = self.execute_block(body)?;
                    match ctrl {
                        Control::Break => break,
                        Control::Continue => continue,
                        Control::Return(_) => return Ok(ctrl),
                        Control::None => {}
                    }
                }
                Ok(Control::None)
            }
            StmtKind::For { item, iterable, body } => {
                let iter_val = self.evaluate_expr(iterable)?;
                let items: Vec<JoeValue> = match iter_val {
                    JoeValue::List(l) => l.borrow().clone(),
                    JoeValue::Text(s) => s.chars().map(|c| JoeValue::Text(c.to_string())).collect(),
                    JoeValue::Bag(b) => b.borrow().keys().map(|k| JoeValue::Text(k.clone())).collect(),
                    other => return Err(format!("Cannot iterate over '{}' at {}", other.type_name(), iterable.span)),
                };

                for val in items {
                    let prev_env = self.env.clone();
                    self.env = Environment::with_enclosing(prev_env.clone());
                    self.env.borrow_mut().define(item, val);
                    let ctrl = self.execute_block_in_current_env(body);
                    self.env = prev_env;

                    match ctrl? {
                        Control::Break => break,
                        Control::Continue => continue,
                        Control::Return(v) => return Ok(Control::Return(v)),
                        Control::None => {}
                    }
                }
                Ok(Control::None)
            }
            StmtKind::GiveUp => Ok(Control::Break),
            StmtKind::TryAnyway => Ok(Control::Continue),
            StmtKind::Deliver(val_opt) => {
                let val = match val_opt {
                    Some(e) => Some(self.evaluate_expr(e)?),
                    None => None,
                };
                Ok(Control::Return(val))
            }
            StmtKind::Function { name, params, body } => {
                let closure = JoeClosure {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    env: self.env.clone(),
                };
                self.env.borrow_mut().define(name, JoeValue::Habit(Gc::new(RefCell::new(closure))));
                Ok(Control::None)
            }
            StmtKind::Class { name, fields, methods } => {
                self.classes.insert(name.clone(), (fields.clone(), methods.clone()));
                Ok(Control::None)
            }
            StmtKind::Module { name, body } => {
                let prev_env = self.env.clone();
                let mod_env = Environment::with_enclosing(prev_env.clone());
                self.env = mod_env.clone();
                for s in body {
                    self.execute_stmt(s)?;
                }
                self.env = prev_env;

                let mut exports = HashMap::new();
                for (k, v) in mod_env.borrow().all_locals() {
                    exports.insert(k, v);
                }
                let mod_bag = JoeValue::Bag(Gc::new(RefCell::new(exports)));
                self.env.borrow_mut().define(name, mod_bag);
                Ok(Control::None)
            }
            StmtKind::Panic(expr) => {
                let val = self.evaluate_expr(expr)?;
                Err(format!("Joe panics at {}: {}", stmt.span, val))
            }
            StmtKind::CallFor(path) => {
                let module = if let Some(m) = stdlib::load_module(path) {
                    m
                } else {
                    // Try loading file from disk
                    let file_candidates = [
                        format!("{}.joe", path),
                        path.to_string(),
                        format!("src/{}.joe", path),
                    ];
                    let mut loaded = None;
                    for candidate in &file_candidates {
                        if Path::new(candidate).exists() {
                            let content = fs::read_to_string(candidate)
                                .map_err(|e| format!("Failed to read module file '{}': {}", candidate, e))?;
                            let mut tokenizer = crate::tokenizer::Tokenizer::new(&content);
                            let tokens = tokenizer.tokenize()?;
                            let mut parser = crate::parser::Parser::new(tokens);
                            let stmts = parser.parse()?;

                            let prev_env = self.env.clone();
                            let file_env = Environment::with_enclosing(self.global_env.clone());
                            self.env = file_env.clone();
                            for s in &stmts {
                                self.execute_stmt(s)?;
                            }
                            self.env = prev_env;

                            let mut exports = HashMap::new();
                            for (k, v) in file_env.borrow().all_locals() {
                                exports.insert(k, v);
                            }
                            loaded = Some(JoeValue::Bag(Gc::new(RefCell::new(exports))));
                            break;
                        }
                    }
                    loaded.ok_or_else(|| format!("Joe calls for unknown module or file '{}' at {}", path, stmt.span))?
                };

                // Store under full path
                self.env.borrow_mut().define(path, module.clone());
                // Handle namespaces like std.math -> std is a bag, math is a property
                let parts: Vec<&str> = path.split('.').collect();
                if parts.len() == 2 {
                    let root = parts[0];
                    let sub = parts[1];
                    let existing = self.env.borrow().get(root);
                    let root_bag = match existing {
                        Some(JoeValue::Bag(b)) => b,
                        _ => {
                            let b = Gc::new(RefCell::new(HashMap::new()));
                            self.env.borrow_mut().define(root, JoeValue::Bag(b.clone()));
                            b
                        }
                    };
                    root_bag.borrow_mut().insert(sub.to_string(), module.clone());
                    self.env.borrow_mut().define(sub, module);
                } else if parts.len() == 1 {
                    self.env.borrow_mut().define(parts[0], module);
                }
                Ok(Control::None)
            }
            StmtKind::Expr(expr) => {
                self.evaluate_expr(expr)?;
                Ok(Control::None)
            }
        }
    }

    fn dispatch_keyword_override(
        &mut self,
        target: &JoeValue,
        method_name: &str,
        args: &[JoeValue],
        default_file_text: &str,
        span: crate::token::Span,
    ) -> Result<(), String> {
        match target {
            JoeValue::Person(p) => {
                if p.borrow().methods.contains_key(method_name) {
                    let m = p.borrow().methods.get(method_name).unwrap().clone();
                    let method_env = Environment::with_enclosing(m.env.clone());
                    method_env.borrow_mut().define("jthis", JoeValue::Person(p.clone()));
                    let habit = JoeValue::Habit(Gc::new(RefCell::new(JoeClosure {
                        name: m.name,
                        params: m.params,
                        body: m.body,
                        env: method_env,
                    })));
                    self.call_value(&habit, args, span)?;
                    Ok(())
                } else {
                    Err(format!("Archetype '{}' does not override '{}' at {}", p.borrow().class_name, method_name, span))
                }
            }
            JoeValue::Text(path) => {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|e| format!("Failed to open file '{}' for {}: {}", path, method_name, e))?;
                file.write_all(default_file_text.as_bytes())
                    .map_err(|e| format!("Failed to write to file '{}': {}", path, e))?;
                Ok(())
            }
            JoeValue::Bag(b) => {
                if let Some(JoeValue::Habit(h)) = b.borrow().get(method_name) {
                    let habit = JoeValue::Habit(h.clone());
                    self.call_value(&habit, args, span)?;
                    Ok(())
                } else if let Some(JoeValue::NativeHabit(f)) = b.borrow().get(method_name) {
                    f(args)?;
                    Ok(())
                } else {
                    Err(format!("Bag does not have a '{}' habit at {}", method_name, span))
                }
            }
            other => Err(format!("Cannot {} to '{}' at {}", method_name, other.type_name(), span)),
        }
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<Control, String> {
        let prev = self.env.clone();
        self.env = Environment::with_enclosing(prev.clone());
        let res = self.execute_block_in_current_env(stmts);
        self.env = prev;
        res
    }

    fn execute_block_in_current_env(&mut self, stmts: &[Stmt]) -> Result<Control, String> {
        for s in stmts {
            let ctrl = self.execute_stmt(s)?;
            if ctrl != Control::None {
                return Ok(ctrl);
            }
        }
        Ok(Control::None)
    }

    pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<JoeValue, String> {
        match &expr.kind {
            ExprKind::Nothing => Ok(JoeValue::Nothing),
            ExprKind::Bool(b) => Ok(JoeValue::Agree(*b)),
            ExprKind::Number(n) => Ok(JoeValue::Number(*n)),
            ExprKind::Text(s) => Ok(JoeValue::Text(s.clone())),
            ExprKind::Ident(name) => {
                if let Some(val) = self.env.borrow().get(name) {
                    Ok(val)
                } else {
                    Err(format!("Joe has no memory of '{}' at {}", name, expr.span))
                }
            }
            ExprKind::JThis => {
                if let Some(val) = self.env.borrow().get("jthis") {
                    Ok(val)
                } else {
                    Err(format!("'jthis' has no meaning outside of Joe's classes at {}", expr.span))
                }
            }
            ExprKind::List(elements) => {
                let mut list = Vec::new();
                for e in elements {
                    list.push(self.evaluate_expr(e)?);
                }
                Ok(JoeValue::List(Gc::new(RefCell::new(list))))
            }
            ExprKind::Bag(entries) => {
                let mut map = HashMap::new();
                for (k, v) in entries {
                    map.insert(k.clone(), self.evaluate_expr(v)?);
                }
                Ok(JoeValue::Bag(Gc::new(RefCell::new(map))))
            }
            ExprKind::Unary { op, expr: inner } => {
                let val = self.evaluate_expr(inner)?;
                match op {
                    UnaryOp::Neg => match val {
                        JoeValue::Number(n) => Ok(JoeValue::Number(-n)),
                        other => Err(format!("Cannot negate '{}' at {}", other.type_name(), expr.span)),
                    },
                    UnaryOp::Not => Ok(JoeValue::Agree(!val.is_truthy())),
                }
            }
            ExprKind::Binary { left, op, right } => {
                // Short-circuiting for logical operators
                if *op == BinaryOp::And {
                    let l = self.evaluate_expr(left)?;
                    if !l.is_truthy() {
                        return Ok(l);
                    }
                    return self.evaluate_expr(right);
                }
                if *op == BinaryOp::Or {
                    let l = self.evaluate_expr(left)?;
                    if l.is_truthy() {
                        return Ok(l);
                    }
                    return self.evaluate_expr(right);
                }

                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                // Check class operator overloading on left operand
                if let JoeValue::Person(ref p) = l {
                    let method_name = match op {
                        BinaryOp::Add => "add",
                        BinaryOp::Sub => "sub",
                        BinaryOp::Mul => "mul",
                        BinaryOp::Div => "div",
                        BinaryOp::Mod => "mod",
                        BinaryOp::Eq => "eq",
                        _ => "",
                    };
                    if !method_name.is_empty() && p.borrow().methods.contains_key(method_name) {
                        let m = p.borrow().methods.get(method_name).unwrap().clone();
                        let method_env = Environment::with_enclosing(m.env.clone());
                        method_env.borrow_mut().define("jthis", l.clone());
                        let habit = JoeValue::Habit(Gc::new(RefCell::new(JoeClosure {
                            name: m.name,
                            params: m.params,
                            body: m.body,
                            env: method_env,
                        })));
                        return self.call_value(&habit, &[r], expr.span);
                    }
                }

                match op {
                    BinaryOp::Add => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => Ok(JoeValue::Number(a + b)),
                        (JoeValue::Text(a), b) => Ok(JoeValue::Text(format!("{}{}", a, b))),
                        (a, JoeValue::Text(b)) => Ok(JoeValue::Text(format!("{}{}", a, b))),
                        (JoeValue::List(a), JoeValue::List(b)) => {
                            let mut combined = a.borrow().clone();
                            combined.extend(b.borrow().clone());
                            Ok(JoeValue::List(Gc::new(RefCell::new(combined))))
                        }
                        _ => Err(format!("Cannot add '{}' and '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::Sub => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => Ok(JoeValue::Number(a - b)),
                        _ => Err(format!("Cannot subtract '{}' and '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::Mul => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => Ok(JoeValue::Number(a * b)),
                        (JoeValue::Text(s), JoeValue::Number(n)) => {
                            let times = (*n as usize).max(0);
                            Ok(JoeValue::Text(s.repeat(times)))
                        }
                        _ => Err(format!("Cannot multiply '{}' and '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::Div => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => {
                            if *b == 0.0 {
                                Err(format!("Joe attempts to divide by zero at {}", expr.span))
                            } else {
                                Ok(JoeValue::Number(a / b))
                            }
                        }
                        _ => Err(format!("Cannot divide '{}' by '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::Mod => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => Ok(JoeValue::Number(a % b)),
                        _ => Err(format!("Cannot modulo '{}' and '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::Eq => Ok(JoeValue::Agree(l == r)),
                    BinaryOp::NotEq => Ok(JoeValue::Agree(l != r)),
                    BinaryOp::Lt => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => Ok(JoeValue::Agree(a < b)),
                        (JoeValue::Text(a), JoeValue::Text(b)) => Ok(JoeValue::Agree(a < b)),
                        _ => Err(format!("Cannot compare '<' on '{}' and '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::LtEq => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => Ok(JoeValue::Agree(a <= b)),
                        (JoeValue::Text(a), JoeValue::Text(b)) => Ok(JoeValue::Agree(a <= b)),
                        _ => Err(format!("Cannot compare '<=' on '{}' and '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::Gt => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => Ok(JoeValue::Agree(a > b)),
                        (JoeValue::Text(a), JoeValue::Text(b)) => Ok(JoeValue::Agree(a > b)),
                        _ => Err(format!("Cannot compare '>' on '{}' and '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::GtEq => match (&l, &r) {
                        (JoeValue::Number(a), JoeValue::Number(b)) => Ok(JoeValue::Agree(a >= b)),
                        (JoeValue::Text(a), JoeValue::Text(b)) => Ok(JoeValue::Agree(a >= b)),
                        _ => Err(format!("Cannot compare '>=' on '{}' and '{}' at {}", l.type_name(), r.type_name(), expr.span)),
                    },
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                }
            }
            ExprKind::Call { callee, args } => {
                let callee_val = self.evaluate_expr(callee)?;
                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.evaluate_expr(a)?);
                }
                self.call_value(&callee_val, &arg_vals, expr.span)
            }
            ExprKind::GetField { object, name } => {
                let obj = self.evaluate_expr(object)?;
                match obj {
                    JoeValue::Person(p) => {
                        let inst = p.borrow();
                        if let Some(f) = inst.fields.get(name) {
                            Ok(f.clone())
                        } else if let Some(m) = inst.methods.get(name) {
                            // Bind method with jthis = p
                            let method_env = Environment::with_enclosing(m.env.clone());
                            method_env.borrow_mut().define("jthis", JoeValue::Person(p.clone()));
                            let bound = JoeClosure {
                                name: m.name.clone(),
                                params: m.params.clone(),
                                body: m.body.clone(),
                                env: method_env,
                            };
                            Ok(JoeValue::Habit(Gc::new(RefCell::new(bound))))
                        } else {
                            Ok(JoeValue::Nothing)
                        }
                    }
                    JoeValue::Bag(b) => {
                        let bag = b.borrow();
                        Ok(bag.get(name).cloned().unwrap_or(JoeValue::Nothing))
                    }
                    other => Err(format!("Cannot access field '{}' on '{}' at {}", name, other.type_name(), expr.span)),
                }
            }
            ExprKind::GetIndex { object, index } => {
                let obj = self.evaluate_expr(object)?;
                let idx = self.evaluate_expr(index)?;
                match (&obj, &idx) {
                    (JoeValue::List(l), JoeValue::Number(n)) => {
                        let i = *n as usize;
                        let list = l.borrow();
                        if i < list.len() {
                            Ok(list[i].clone())
                        } else {
                            Ok(JoeValue::Nothing)
                        }
                    }
                    (JoeValue::Text(s), JoeValue::Number(n)) => {
                        let i = *n as usize;
                        if let Some(ch) = s.chars().nth(i) {
                            Ok(JoeValue::Text(ch.to_string()))
                        } else {
                            Ok(JoeValue::Nothing)
                        }
                    }
                    (JoeValue::Bag(b), key) => {
                        let k = format!("{}", key);
                        let bag = b.borrow();
                        Ok(bag.get(&k).cloned().unwrap_or(JoeValue::Nothing))
                    }
                    _ => Err(format!("Cannot index '{}' with '{}' at {}", obj.type_name(), idx.type_name(), expr.span)),
                }
            }
            ExprKind::New { class_name, args } => {
                let (fields, methods) = self.classes.get(class_name)
                    .cloned()
                    .ok_or_else(|| format!("Joe meets an unknown class '{}' at {}", class_name, expr.span))?;

                let mut inst = JoeInstance::new(class_name.clone());

                let mut evaluated_args = Vec::new();
                for a in args {
                    evaluated_args.push(self.evaluate_expr(a)?);
                }

                for (i, f) in fields.iter().enumerate() {
                    let val = evaluated_args.get(i).cloned().unwrap_or(JoeValue::Nothing);
                    inst.fields.insert(f.clone(), val);
                }

                for m_stmt in methods {
                    if let StmtKind::Function { name, params, body } = m_stmt.kind {
                        let closure = JoeClosure {
                            name: name.clone(),
                            params,
                            body,
                            env: self.env.clone(),
                        };
                        inst.methods.insert(name, closure);
                    }
                }

                let inst_gc = Gc::new(RefCell::new(inst));
                let person_val = JoeValue::Person(inst_gc.clone());

                if inst_gc.borrow().methods.contains_key("init") {
                    let method = inst_gc.borrow().methods.get("init").unwrap().clone();
                    let method_env = Environment::with_enclosing(method.env.clone());
                    method_env.borrow_mut().define("jthis", person_val.clone());
                    let bound = JoeClosure {
                        name: method.name,
                        params: method.params,
                        body: method.body,
                        env: method_env,
                    };
                    let habit = JoeValue::Habit(Gc::new(RefCell::new(bound)));
                    self.call_value(&habit, &evaluated_args, expr.span)?;
                }

                Ok(person_val)
            }
        }
    }

    pub fn call_value(&mut self, callee: &JoeValue, args: &[JoeValue], span: crate::token::Span) -> Result<JoeValue, String> {
        match callee {
            JoeValue::NativeHabit(native_fn) => native_fn(args),
            JoeValue::Habit(h) => {
                let closure = h.borrow().clone();
                let call_env = Environment::with_enclosing(closure.env.clone());

                for (i, param) in closure.params.iter().enumerate() {
                    let arg = args.get(i).cloned().unwrap_or(JoeValue::Nothing);
                    call_env.borrow_mut().define(param, arg);
                }

                let prev_env = self.env.clone();
                self.env = call_env;
                let ctrl = self.execute_block_in_current_env(&closure.body);
                self.env = prev_env;

                match ctrl? {
                    Control::Return(Some(v)) => Ok(v),
                    Control::Return(None) => Ok(JoeValue::Nothing),
                    _ => Ok(JoeValue::Nothing),
                }
            }
            other => Err(format!("Joe tried to invoke '{}', which is not a habit at {}", other.type_name(), span)),
        }
    }
}
