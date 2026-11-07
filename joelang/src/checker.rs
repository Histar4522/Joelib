use std::collections::HashMap;
use crate::ast::*;
use crate::token::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub is_warning: bool,
}

pub struct Checker {
    scopes: Vec<HashMap<String, (Span, bool)>>, // name -> (declared_at, used)
    loop_depth: usize,
    function_depth: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Checker {
    pub fn new() -> Self {
        let mut global_scope = HashMap::new();
        // Global prelude identifiers
        for name in &["len", "type", "str", "num", "bool", "panic", "reflect"] {
            global_scope.insert(name.to_string(), (Span::new(0, 0), true));
        }

        Self {
            scopes: vec![global_scope],
            loop_depth: 0,
            function_depth: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn check(mut self, stmts: &[Stmt]) -> Vec<Diagnostic> {
        for stmt in stmts {
            self.check_stmt(stmt);
        }

        // Check unused variables in global scope
        self.check_unused_in_current_scope();

        self.diagnostics
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.check_unused_in_current_scope();
        self.scopes.pop();
    }

    fn check_unused_in_current_scope(&mut self) {
        if let Some(scope) = self.scopes.last() {
            for (name, (span, used)) in scope {
                if !*used && !name.starts_with('_') && span.line > 0 {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Joe remembered '{}' at {}, but never thought about it again. Typical Joe.", name, span),
                        span: *span,
                        is_warning: true,
                    });
                }
            }
        }
    }

    fn declare(&mut self, name: &str, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                self.diagnostics.push(Diagnostic {
                    message: format!("Joe had already remembered '{}' earlier. Joe seems to be repeating himself.", name),
                    span,
                    is_warning: true,
                });
            }
            scope.insert(name.to_string(), (span, false));
        }
    }

    fn mark_used(&mut self, name: &str) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(entry) = scope.get_mut(name) {
                entry.1 = true;
                return true;
            }
        }
        false
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Remember { name, init } => {
                self.check_expr(init);
                self.declare(name, stmt.span);
            }
            StmtKind::Assign { target, value } => {
                self.check_expr(value);
                if let ExprKind::Ident(name) = &target.kind {
                    if !self.mark_used(name) {
                        self.diagnostics.push(Diagnostic {
                            message: format!("Amnesia Error: Joe tries to change '{}', but Joe has no recollection of it.", name),
                            span: target.span,
                            is_warning: false,
                        });
                    }
                } else {
                    self.check_expr(target);
                }
            }
            StmtKind::Yell { expr, to } | StmtKind::Whisper { expr, to } => {
                self.check_expr(expr);
                if let Some(t) = to {
                    self.check_expr(t);
                }
            }
            StmtKind::Panic(e) => {
                self.check_expr(e);
            }
            StmtKind::Listen { name, from } => {
                if let Some(f) = from {
                    self.check_expr(f);
                }
                self.declare(name, stmt.span);
            }
            StmtKind::If { cond, then_branch, else_branch } => {
                self.check_expr(cond);
                self.enter_scope();
                for s in then_branch {
                    self.check_stmt(s);
                }
                self.exit_scope();

                if let Some(elses) = else_branch {
                    self.enter_scope();
                    for s in elses {
                        self.check_stmt(s);
                    }
                    self.exit_scope();
                }
            }
            StmtKind::While { cond, body } => {
                self.check_expr(cond);
                self.loop_depth += 1;
                self.enter_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.exit_scope();
                self.loop_depth -= 1;
            }
            StmtKind::For { item, iterable, body } => {
                self.check_expr(iterable);
                self.loop_depth += 1;
                self.enter_scope();
                self.declare(item, stmt.span);
                for s in body {
                    self.check_stmt(s);
                }
                self.exit_scope();
                self.loop_depth -= 1;
            }
            StmtKind::GiveUp => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(Diagnostic {
                        message: "Existential Error: 'Joe gives up.' outside of a loop. Joe cannot escape reality that easily.".to_string(),
                        span: stmt.span,
                        is_warning: false,
                    });
                }
            }
            StmtKind::TryAnyway => {
                if self.loop_depth == 0 {
                    self.diagnostics.push(Diagnostic {
                        message: "Existential Error: 'Joe tries anyway.' outside of a loop. Try what, exactly?".to_string(),
                        span: stmt.span,
                        is_warning: false,
                    });
                }
            }
            StmtKind::Deliver(val_opt) => {
                if self.function_depth == 0 {
                    self.diagnostics.push(Diagnostic {
                        message: "Confusion Error: 'Joe delivers' outside of any habit/function. Deliver to whom?".to_string(),
                        span: stmt.span,
                        is_warning: false,
                    });
                }
                if let Some(e) = val_opt {
                    self.check_expr(e);
                }
            }
            StmtKind::Function { name, params, body } => {
                self.declare(name, stmt.span);
                self.function_depth += 1;
                self.enter_scope();
                for p in params {
                    self.declare(p, stmt.span);
                }
                for s in body {
                    self.check_stmt(s);
                }
                self.exit_scope();
                self.function_depth -= 1;
            }
            StmtKind::Class { name, fields, methods } => {
                self.declare(name, stmt.span);
                self.enter_scope();
                self.declare("jthis", stmt.span);
                for f in fields {
                    self.declare(f, stmt.span);
                }
                for m in methods {
                    self.check_stmt(m);
                }
                self.exit_scope();
            }
            StmtKind::Module { name, body } => {
                self.declare(name, stmt.span);
                self.enter_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.exit_scope();
            }
            StmtKind::CallFor(path) => {
                let parts: Vec<&str> = path.split('.').collect();
                if let Some(root) = parts.first() {
                    self.declare(root, stmt.span);
                }
                if let Some(short) = parts.last() {
                    self.declare(short, stmt.span);
                }
            }
            StmtKind::Expr(e) => {
                self.check_expr(e);
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Ident(name) => {
                if !self.mark_used(name) {
                    self.diagnostics.push(Diagnostic {
                        message: format!("Amnesia Error at {}: Joe is thinking about '{}', but Joe has never heard of '{}'.", expr.span, name, name),
                        span: expr.span,
                        is_warning: false,
                    });
                }
            }
            ExprKind::Unary { expr: inner, .. } => self.check_expr(inner),
            ExprKind::Binary { left, right, .. } => {
                self.check_expr(left);
                self.check_expr(right);
            }
            ExprKind::Call { callee, args } => {
                self.check_expr(callee);
                for a in args {
                    self.check_expr(a);
                }
            }
            ExprKind::GetField { object, .. } => {
                self.check_expr(object);
            }
            ExprKind::GetIndex { object, index } => {
                self.check_expr(object);
                self.check_expr(index);
            }
            ExprKind::List(elements) => {
                for e in elements {
                    self.check_expr(e);
                }
            }
            ExprKind::Bag(entries) => {
                for (_, v) in entries {
                    self.check_expr(v);
                }
            }
            ExprKind::New { class_name, args } => {
                self.mark_used(class_name);
                for a in args {
                    self.check_expr(a);
                }
            }
            _ => {}
        }
    }
}
