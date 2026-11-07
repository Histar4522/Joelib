pub mod ast;
pub mod checker;
pub mod debugger;
pub mod environment;
pub mod explainer;
pub mod interpreter;
pub mod parser;
pub mod stdlib;
pub mod token;
pub mod tokenizer;
pub mod value;

use checker::{Checker, Diagnostic};
use explainer::Explainer;
use interpreter::Interpreter;
use parser::Parser;
use tokenizer::Tokenizer;
use value::JoeValue;

/// `JoeLang` is the central facade and explainer for the JoeLang programming environment.
pub struct JoeLang {
    pub interpreter: Interpreter,
}

impl Default for JoeLang {
    fn default() -> Self {
        Self::new()
    }
}

impl JoeLang {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    /// Tokenizes and parses JoeLang code into an AST.
    pub fn parse(code: &str) -> Result<Vec<ast::Stmt>, String> {
        let mut tokenizer = Tokenizer::new(code);
        let tokens = tokenizer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Evaluates JoeLang code to completion.
    pub fn eval(&mut self, code: &str) -> Result<JoeValue, String> {
        let stmts = Self::parse(code)?;
        self.interpreter.interpret(&stmts)
    }

    /// Analyzes the code using Joe's Conscience, emitting semantic diagnostics and warnings.
    pub fn check(&self, code: &str) -> Result<Vec<Diagnostic>, String> {
        let stmts = Self::parse(code)?;
        let checker = Checker::new();
        Ok(checker.check(&stmts))
    }

    /// Explains the code using the Narrator (documentary voiceover).
    pub fn explain(&self, code: &str) -> Result<String, String> {
        let stmts = Self::parse(code)?;
        Ok(Explainer::explain_program(&stmts))
    }
}
