use crate::ast::*;
use crate::token::{Span, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while !self.is_eof() {
            // Skip stray periods or colons between statements
            if self.check(&TokenKind::Dot) || self.check(&TokenKind::Colon) {
                self.advance();
                continue;
            }
            stmts.push(self.statement()?);
        }
        Ok(stmts)
    }

    fn peek(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            self.tokens.last().unwrap()
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    fn is_eof(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_eof() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_eof() {
            self.pos += 1;
        }
        self.previous()
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: &TokenKind, err_msg: &str) -> Result<&Token, String> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let span = self.peek().span;
            Err(format!("Error at {}: expected {:?}, got {:?}. {}", span, kind, self.peek().kind, err_msg))
        }
    }

    fn consume_optional_period(&mut self) {
        if self.check(&TokenKind::Dot) {
            self.advance();
        }
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        let span = self.peek().span;

        if self.match_token(&TokenKind::JoeRemembers) {
            return self.remember_stmt(span);
        }
        if self.match_token(&TokenKind::JoeSets) {
            return self.sets_stmt(span);
        }
        if self.match_token(&TokenKind::JoeYells) {
            let expr = self.expression()?;
            let to = if self.match_token(&TokenKind::To) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::Yell { expr, to }, span));
        }
        if self.match_token(&TokenKind::JoeWhispers) {
            let expr = self.expression()?;
            let to = if self.match_token(&TokenKind::To) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::Whisper { expr, to }, span));
        }
        if self.match_token(&TokenKind::JoeListensTo) {
            let name = self.consume_ident("Expected variable name after 'Joe listens to'")?;
            let from = if self.match_token(&TokenKind::From) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::Listen { name, from }, span));
        }
        if self.match_token(&TokenKind::IfJoeBelieves) {
            return self.if_stmt(span);
        }
        if self.match_token(&TokenKind::WhileJoeWorriesAbout) {
            return self.while_stmt(span);
        }
        if self.match_token(&TokenKind::ForEvery) {
            return self.for_stmt(span);
        }
        if self.match_token(&TokenKind::JoeGivesUp) {
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::GiveUp, span));
        }
        if self.match_token(&TokenKind::JoeTriesAnyway) {
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::TryAnyway, span));
        }
        if self.match_token(&TokenKind::JoeDelivers) {
            let val = if self.check(&TokenKind::Dot) || self.check(&TokenKind::Eof) {
                None
            } else {
                Some(self.expression()?)
            };
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::Deliver(val), span));
        }
        if self.match_token(&TokenKind::ToFunc) {
            return self.function_stmt(span);
        }
        if self.match_token(&TokenKind::MeetA) {
            return self.class_stmt(span);
        }
        if self.match_token(&TokenKind::JoeEstablishes) || self.match_token(&TokenKind::Chapter) {
            return self.module_stmt(span);
        }
        if self.match_token(&TokenKind::JoePanics) {
            let expr = self.expression()?;
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::Panic(expr), span));
        }
        if self.match_token(&TokenKind::JoeCallsFor) {
            let mod_name = self.module_path()?;
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::CallFor(mod_name), span));
        }

        // Could be an assignment via "turns into" or "=": `target turns into value.`
        let expr = self.expression()?;
        if self.match_token(&TokenKind::TurnsInto) || self.match_token(&TokenKind::Assign) {
            let value = self.expression()?;
            self.consume_optional_period();
            return Ok(Stmt::new(StmtKind::Assign { target: expr, value }, span));
        }

        self.consume_optional_period();
        Ok(Stmt::new(StmtKind::Expr(expr), span))
    }

    fn remember_stmt(&mut self, span: Span) -> Result<Stmt, String> {
        let name = self.consume_ident("Expected variable name after 'Joe remembers'")?;
        self.consume(&TokenKind::As, "Expected 'as' after variable name in declaration")?;
        let init = self.expression()?;
        self.consume_optional_period();
        Ok(Stmt::new(StmtKind::Remember { name, init }, span))
    }

    fn sets_stmt(&mut self, span: Span) -> Result<Stmt, String> {
        let target = self.expression()?;
        if !self.match_token(&TokenKind::To) && !self.match_token(&TokenKind::ToFunc) {
            return Err(format!("Expected 'to' after target in 'Joe sets ...' at {}", self.peek().span));
        }
        let value = self.expression()?;
        self.consume_optional_period();
        Ok(Stmt::new(StmtKind::Assign { target, value }, span))
    }

    fn if_stmt(&mut self, span: Span) -> Result<Stmt, String> {
        let cond = self.expression()?;
        self.consume_optional_colon();

        let mut then_branch = Vec::new();
        while !self.is_eof() && !self.check(&TokenKind::Otherwise) && !self.check(&TokenKind::ThatIsThat) {
            then_branch.push(self.statement()?);
        }

        let else_branch = if self.match_token(&TokenKind::Otherwise) {
            self.consume_optional_colon();
            let mut elses = Vec::new();
            while !self.is_eof() && !self.check(&TokenKind::ThatIsThat) {
                elses.push(self.statement()?);
            }
            Some(elses)
        } else {
            None
        };

        if !self.match_token(&TokenKind::ThatIsThat) {
            return Err(format!("Expected 'That is that.' to close 'If Joe believes' block at {}", self.peek().span));
        }
        self.consume_optional_period();

        Ok(Stmt::new(StmtKind::If { cond, then_branch, else_branch }, span))
    }

    fn while_stmt(&mut self, span: Span) -> Result<Stmt, String> {
        let cond = self.expression()?;
        self.consume_optional_colon();

        let mut body = Vec::new();
        while !self.is_eof() && !self.check(&TokenKind::JoeMovesOn) {
            body.push(self.statement()?);
        }

        if !self.match_token(&TokenKind::JoeMovesOn) {
            return Err(format!("Expected 'Joe moves on.' to close 'While Joe worries about' loop at {}", self.peek().span));
        }
        self.consume_optional_period();

        Ok(Stmt::new(StmtKind::While { cond, body }, span))
    }

    fn for_stmt(&mut self, span: Span) -> Result<Stmt, String> {
        let item = self.consume_ident("Expected loop variable after 'For every'")?;
        self.consume(&TokenKind::In, "Expected 'in' in for loop")?;
        let iterable = self.expression()?;
        self.consume_optional_colon();

        let mut body = Vec::new();
        while !self.is_eof() && !self.check(&TokenKind::JoeMovesOn) {
            body.push(self.statement()?);
        }

        if !self.match_token(&TokenKind::JoeMovesOn) {
            return Err(format!("Expected 'Joe moves on.' to close 'For every' loop at {}", self.peek().span));
        }
        self.consume_optional_period();

        Ok(Stmt::new(StmtKind::For { item, iterable, body }, span))
    }

    fn function_stmt(&mut self, span: Span) -> Result<Stmt, String> {
        let name = self.consume_ident("Expected function name after 'To'")?;
        let mut params = Vec::new();
        if self.match_token(&TokenKind::With) {
            loop {
                params.push(self.consume_ident("Expected parameter name")?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume_optional_colon();

        let mut body = Vec::new();
        while !self.is_eof() && !self.check(&TokenKind::ThatsHowJoeDoesIt) {
            body.push(self.statement()?);
        }

        if !self.match_token(&TokenKind::ThatsHowJoeDoesIt) {
            return Err(format!("Expected \"That's how Joe does it.\" to close function '{}' at {}", name, self.peek().span));
        }
        self.consume_optional_period();

        Ok(Stmt::new(StmtKind::Function { name, params, body }, span))
    }

    fn class_stmt(&mut self, span: Span) -> Result<Stmt, String> {
        let name = self.consume_ident("Expected class name after 'Meet a'")?;
        self.consume_optional_colon();

        let mut fields = Vec::new();
        if self.match_token(&TokenKind::ItHas) {
            loop {
                fields.push(self.consume_ident("Expected field name in 'It has ...'")?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.consume_optional_period();
        }

        let mut methods = Vec::new();
        while !self.is_eof() && !self.check(&TokenKind::ThatsA) {
            if self.match_token(&TokenKind::ToFunc) {
                methods.push(self.function_stmt(self.previous().span)?);
            } else {
                return Err(format!("Expected method definition in class '{}' at {}", name, self.peek().span));
            }
        }

        if !self.match_token(&TokenKind::ThatsA) && !self.match_token(&TokenKind::Thats) {
            return Err(format!("Expected \"That's a {}\" to close class at {}", name, self.peek().span));
        }
        let closing_name = self.consume_ident("Expected class name after \"That's\"")?;
        if closing_name != name {
            return Err(format!("Class started as '{}' but ended as '{}'", name, closing_name));
        }
        self.consume_optional_period();

        Ok(Stmt::new(StmtKind::Class { name, fields, methods }, span))
    }

    fn module_stmt(&mut self, span: Span) -> Result<Stmt, String> {
        let name = self.consume_ident("Expected module name")?;
        self.consume_optional_colon();

        let mut body = Vec::new();
        while !self.is_eof() && !self.check(&TokenKind::Thats) && !self.check(&TokenKind::ThatsA) {
            body.push(self.statement()?);
        }

        if self.match_token(&TokenKind::Thats) || self.match_token(&TokenKind::ThatsA) {
            let closing = self.consume_ident("Expected module name after closing 'That's'")?;
            if closing != name {
                return Err(format!("Module started as '{}' but closed as '{}'", name, closing));
            }
        } else {
            return Err(format!("Expected \"That's {}\" to close module at {}", name, self.peek().span));
        }
        self.consume_optional_period();

        Ok(Stmt::new(StmtKind::Module { name, body }, span))
    }

    fn module_path(&mut self) -> Result<String, String> {
        let mut path = self.consume_ident("Expected module name after 'Joe calls for'")?;
        while self.check(&TokenKind::Dot) {
            if let Some(next) = self.peek_next_token() {
                if matches!(next.kind, TokenKind::Ident(_)) && next.span.line == self.peek().span.line {
                    self.advance(); // consume Dot
                    if let TokenKind::Ident(sub) = &self.peek().kind {
                        let s = sub.clone();
                        path.push('.');
                        path.push_str(&s);
                        self.advance();
                        continue;
                    }
                }
            }
            break;
        }
        Ok(path)
    }

    fn consume_ident(&mut self, err_msg: &str) -> Result<String, String> {
        if let TokenKind::Ident(name) = &self.peek().kind {
            let n = name.clone();
            self.advance();
            Ok(n)
        } else {
            Err(format!("Error at {}: {}", self.peek().span, err_msg))
        }
    }

    fn consume_optional_colon(&mut self) {
        if self.check(&TokenKind::Colon) {
            self.advance();
        }
    }

    // --- Expression Pratt Parsing ---

    pub fn expression(&mut self) -> Result<Expr, String> {
        self.parse_precedence(Precedence::Or)
    }

    fn parse_precedence(&mut self, prec: Precedence) -> Result<Expr, String> {
        let mut left = self.unary()?;

        while prec <= self.current_precedence() {
            left = self.infix(left)?;
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        let span = self.peek().span;
        if self.match_token(&TokenKind::Minus) {
            let expr = self.parse_precedence(Precedence::Unary)?;
            return Ok(Expr::new(ExprKind::Unary { op: UnaryOp::Neg, expr: Box::new(expr) }, span));
        }
        if self.match_token(&TokenKind::Not) {
            let expr = self.parse_precedence(Precedence::Unary)?;
            return Ok(Expr::new(ExprKind::Unary { op: UnaryOp::Not, expr: Box::new(expr) }, span));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let span = self.peek().span;

        if self.match_token(&TokenKind::Nothing) {
            return Ok(Expr::new(ExprKind::Nothing, span));
        }
        if self.match_token(&TokenKind::JoeAgrees) {
            return Ok(Expr::new(ExprKind::Bool(true), span));
        }
        if self.match_token(&TokenKind::JoeDisagrees) {
            return Ok(Expr::new(ExprKind::Bool(false), span));
        }
        if let TokenKind::Number(n) = self.peek().kind {
            self.advance();
            return Ok(Expr::new(ExprKind::Number(n), span));
        }
        if let TokenKind::Text(s) = &self.peek().kind {
            let t = s.clone();
            self.advance();
            return Ok(Expr::new(ExprKind::Text(t), span));
        }
        if self.match_token(&TokenKind::JThis) {
            return Ok(Expr::new(ExprKind::JThis, span));
        }
        if self.match_token(&TokenKind::ANew) {
            let class_name = self.consume_ident("Expected class name after 'a new'")?;
            self.consume(&TokenKind::LParen, "Expected '(' after class name in instantiation")?;
            let mut args = Vec::new();
            if !self.check(&TokenKind::RParen) {
                loop {
                    args.push(self.expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume(&TokenKind::RParen, "Expected ')' after arguments in instantiation")?;
            return Ok(Expr::new(ExprKind::New { class_name, args }, span));
        }
        if let TokenKind::Ident(id) = &self.peek().kind {
            let name = id.clone();
            self.advance();
            return Ok(Expr::new(ExprKind::Ident(name), span));
        }
        if self.match_token(&TokenKind::LParen) {
            let expr = self.expression()?;
            self.consume(&TokenKind::RParen, "Expected ')' after parenthesized expression")?;
            return Ok(expr);
        }
        if self.match_token(&TokenKind::LBracket) {
            let mut elements = Vec::new();
            if !self.check(&TokenKind::RBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume(&TokenKind::RBracket, "Expected ']' after list literal")?;
            return Ok(Expr::new(ExprKind::List(elements), span));
        }
        if self.match_token(&TokenKind::LBrace) {
            let mut entries = Vec::new();
            if !self.check(&TokenKind::RBrace) {
                loop {
                    let key = if let TokenKind::Text(s) = &self.peek().kind {
                        let k = s.clone();
                        self.advance();
                        k
                    } else if let TokenKind::Ident(s) = &self.peek().kind {
                        let k = s.clone();
                        self.advance();
                        k
                    } else {
                        return Err(format!("Expected string or identifier key in bag literal at {}", self.peek().span));
                    };
                    self.consume(&TokenKind::Colon, "Expected ':' after key in bag literal")?;
                    let val = self.expression()?;
                    entries.push((key, val));
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume(&TokenKind::RBrace, "Expected '}' after bag literal")?;
            return Ok(Expr::new(ExprKind::Bag(entries), span));
        }

        Err(format!("Unexpected token in expression at {}: {:?}", span, self.peek().kind))
    }

    fn peek_next_token(&self) -> Option<&Token> {
        if self.pos + 1 < self.tokens.len() {
            Some(&self.tokens[self.pos + 1])
        } else {
            None
        }
    }

    fn current_precedence(&self) -> Precedence {
        match &self.peek().kind {
            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::Eq | TokenKind::NotEq => Precedence::Equality,
            TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::Dot => {
                if let Some(next) = self.peek_next_token() {
                    if matches!(next.kind, TokenKind::Ident(_)) && next.span.line == self.peek().span.line {
                        Precedence::Call
                    } else {
                        Precedence::None
                    }
                } else {
                    Precedence::None
                }
            }
            TokenKind::LParen | TokenKind::LBracket => Precedence::Call,
            _ => Precedence::None,
        }
    }

    fn infix(&mut self, left: Expr) -> Result<Expr, String> {
        let span = self.peek().span;

        match self.peek().kind {
            TokenKind::LParen => {
                self.advance();
                let mut args = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(&TokenKind::RParen, "Expected ')' after call arguments")?;
                Ok(Expr::new(ExprKind::Call { callee: Box::new(left), args }, span))
            }
            TokenKind::Dot => {
                self.advance();
                let name = self.consume_ident("Expected field or method name after '.'")?;
                Ok(Expr::new(ExprKind::GetField { object: Box::new(left), name }, span))
            }
            TokenKind::LBracket => {
                self.advance();
                let index = self.expression()?;
                self.consume(&TokenKind::RBracket, "Expected ']' after index")?;
                Ok(Expr::new(ExprKind::GetIndex { object: Box::new(left), index: Box::new(index) }, span))
            }
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash | TokenKind::Percent |
            TokenKind::Eq | TokenKind::NotEq | TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq |
            TokenKind::And | TokenKind::Or => {
                let op = match self.advance().kind {
                    TokenKind::Plus => BinaryOp::Add,
                    TokenKind::Minus => BinaryOp::Sub,
                    TokenKind::Star => BinaryOp::Mul,
                    TokenKind::Slash => BinaryOp::Div,
                    TokenKind::Percent => BinaryOp::Mod,
                    TokenKind::Eq => BinaryOp::Eq,
                    TokenKind::NotEq => BinaryOp::NotEq,
                    TokenKind::Lt => BinaryOp::Lt,
                    TokenKind::LtEq => BinaryOp::LtEq,
                    TokenKind::Gt => BinaryOp::Gt,
                    TokenKind::GtEq => BinaryOp::GtEq,
                    TokenKind::And => BinaryOp::And,
                    TokenKind::Or => BinaryOp::Or,
                    _ => unreachable!(),
                };
                let prec = match op {
                    BinaryOp::Or => Precedence::Or,
                    BinaryOp::And => Precedence::And,
                    BinaryOp::Eq | BinaryOp::NotEq => Precedence::Equality,
                    BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => Precedence::Comparison,
                    BinaryOp::Add | BinaryOp::Sub => Precedence::Term,
                    BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => Precedence::Factor,
                };
                let right = self.parse_precedence(prec)?;
                Ok(Expr::new(ExprKind::Binary { left: Box::new(left), op, right: Box::new(right) }, span))
            }
            _ => Err(format!("Unexpected infix token at {}: {:?}", span, self.peek().kind)),
        }
    }
}
