use crate::token::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Nothing,
    Bool(bool),
    Number(f64),
    Text(String),
    Ident(String),
    JThis,
    List(Vec<Expr>),
    Bag(Vec<(String, Expr)>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    GetField {
        object: Box<Expr>,
        name: String,
    },
    GetIndex {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    New {
        class_name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Remember {
        name: String,
        init: Expr,
    },
    Assign {
        target: Expr,
        value: Expr,
    },
    Yell {
        expr: Expr,
        to: Option<Expr>,
    },
    Whisper {
        expr: Expr,
        to: Option<Expr>,
    },
    Listen {
        name: String,
        from: Option<Expr>,
    },
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        item: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    GiveUp,
    TryAnyway,
    Deliver(Option<Expr>),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Class {
        name: String,
        fields: Vec<String>,
        methods: Vec<Stmt>,
    },
    Module {
        name: String,
        body: Vec<Stmt>,
    },
    Panic(Expr),
    CallFor(String),
    Expr(Expr),
}
