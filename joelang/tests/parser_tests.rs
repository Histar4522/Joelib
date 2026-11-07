use joelang::ast::*;
use joelang::parser::Parser;
use joelang::tokenizer::Tokenizer;

#[test]
fn test_parse_declarations_and_loops() {
    let script = r#"
        Joe remembers coffee as 10.
        While Joe worries about coffee > 0:
            coffee turns into coffee - 1.
            Joe yells "Drinking coffee!".
        Joe moves on.
    "#;

    let mut tokenizer = Tokenizer::new(script);
    let tokens = tokenizer.tokenize().expect("Tokenize failed");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().expect("Parse failed");

    assert_eq!(stmts.len(), 2);
    match &stmts[0].kind {
        StmtKind::Remember { name, init } => {
            assert_eq!(name, "coffee");
            assert_eq!(init.kind, ExprKind::Number(10.0));
        }
        other => panic!("Expected Remember, got {:?}", other),
    }

    match &stmts[1].kind {
        StmtKind::While { cond, body } => {
            assert!(matches!(cond.kind, ExprKind::Binary { op: BinaryOp::Gt, .. }));
            assert_eq!(body.len(), 2);
        }
        other => panic!("Expected While, got {:?}", other),
    }
}

#[test]
fn test_parse_class_and_methods() {
    let script = r#"
        Meet a Person:
            It has name, friend.
            To greet with other:
                Joe yells jthis.name + " greets " + other.
            That's how Joe does it.
        That's a Person.

        Joe remembers p as a new Person("Joe", nothing).
        p.greet("World").
    "#;

    let mut tokenizer = Tokenizer::new(script);
    let tokens = tokenizer.tokenize().expect("Tokenize failed");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().expect("Parse failed");
    assert_eq!(stmts.len(), 3);
    match &stmts[0].kind {
        StmtKind::Class { name, fields, methods } => {
            assert_eq!(name, "Person");
            assert_eq!(fields, &["name", "friend"]);
            assert_eq!(methods.len(), 1);
        }
        other => panic!("Expected Class, got {:?}", other),
    }

    match &stmts[1].kind {
        StmtKind::Remember { name, init } => {
            assert_eq!(name, "p");
            assert!(matches!(init.kind, ExprKind::New { .. }));
        }
        other => panic!("Expected Remember, got {:?}", other),
    }
}

#[test]
fn test_parse_functions_and_conditionals() {
    let script = r#"
        To fib with n:
            If Joe believes n <= 1:
                Joe delivers n.
            Otherwise:
                Joe delivers fib(n - 1) + fib(n - 2).
            That is that.
        That's how Joe does it.
    "#;

    let mut tokenizer = Tokenizer::new(script);
    let tokens = tokenizer.tokenize().expect("Tokenize failed");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().expect("Parse failed");

    assert_eq!(stmts.len(), 1);
    match &stmts[0].kind {
        StmtKind::Function { name, params, body } => {
            assert_eq!(name, "fib");
            assert_eq!(params, &["n"]);
            assert_eq!(body.len(), 1);
            match &body[0].kind {
                StmtKind::If { cond, then_branch, else_branch } => {
                    assert!(matches!(cond.kind, ExprKind::Binary { op: BinaryOp::LtEq, .. }));
                    assert_eq!(then_branch.len(), 1);
                    assert!(else_branch.is_some());
                }
                other => panic!("Expected If, got {:?}", other),
            }
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}
