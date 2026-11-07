use std::collections::HashSet;
use crate::ast::Stmt;
use crate::interpreter::Interpreter;
use crate::value::JoeValue;

pub struct Debugger {
    pub interpreter: Interpreter,
    pub breakpoints: HashSet<usize>,
    pub current_stmt_idx: usize,
    pub call_stack: Vec<String>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            breakpoints: HashSet::new(),
            current_stmt_idx: 0,
            call_stack: vec!["main".to_string()],
        }
    }

    pub fn add_breakpoint(&mut self, line: usize) {
        self.breakpoints.insert(line);
    }

    pub fn remove_breakpoint(&mut self, line: usize) {
        self.breakpoints.remove(&line);
    }

    pub fn locals(&self) -> Vec<(String, JoeValue)> {
        self.interpreter.env.borrow().all_locals()
    }

    pub fn examine(&mut self, expr_str: &str) -> Result<JoeValue, String> {
        let mut tokenizer = crate::tokenizer::Tokenizer::new(expr_str);
        let tokens = tokenizer.tokenize()?;
        let mut parser = crate::parser::Parser::new(tokens);
        let expr = parser.expression()?;
        self.interpreter.evaluate_expr(&expr)
    }

    pub fn step(&mut self, stmts: &[Stmt]) -> Result<Option<usize>, String> {
        if self.current_stmt_idx >= stmts.len() {
            return Ok(None);
        }

        let stmt = &stmts[self.current_stmt_idx];
        self.interpreter.execute_stmt(stmt)?;
        self.current_stmt_idx += 1;

        if self.current_stmt_idx < stmts.len() {
            Ok(Some(stmts[self.current_stmt_idx].span.line))
        } else {
            Ok(None)
        }
    }

    pub fn continue_exec(&mut self, stmts: &[Stmt]) -> Result<Option<usize>, String> {
        while self.current_stmt_idx < stmts.len() {
            let stmt = &stmts[self.current_stmt_idx];
            if self.breakpoints.contains(&stmt.span.line) {
                return Ok(Some(stmt.span.line));
            }
            self.interpreter.execute_stmt(stmt)?;
            self.current_stmt_idx += 1;
        }
        Ok(None)
    }
}
