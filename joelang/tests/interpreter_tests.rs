use joelang::interpreter::Interpreter;
use joelang::parser::Parser;
use joelang::tokenizer::Tokenizer;
use joelang::value::JoeValue;

fn run_joe(code: &str) -> (Result<JoeValue, String>, String) {
    let mut tokenizer = Tokenizer::new(code);
    let tokens = tokenizer.tokenize().expect("Lex failed");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().expect("Parse failed");
    let mut interp = Interpreter::with_capture();
    let res = interp.interpret(&stmts);
    (res, interp.output())
}

#[test]
fn test_basic_life_of_joe() {
    let script = r#"
        Joe remembers coffee as 5.
        Joe remembers cups as 0.
        While Joe worries about coffee > 0:
            coffee turns into coffee - 1.
            cups turns into cups + 1.
            Joe yells "Drank cup " + cups.
        Joe moves on.
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("Drank cup 1"));
    assert!(output.contains("Drank cup 5"));
}

#[test]
fn test_recursive_fibonacci() {
    let script = r#"
        To fib with n:
            If Joe believes n <= 1:
                Joe delivers n.
            Otherwise:
                Joe delivers fib(n - 1) + fib(n - 2).
            That is that.
        That's how Joe does it.

        Joe remembers result as fib(6).
        Joe yells "Result is " + result.
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("Result is 8"));
}

#[test]
fn test_person_class_and_methods() {
    let script = r#"
        Meet a Person:
            It has name, friend.
            To greet with other:
                Joe yells jthis.name + " awkward nods at " + other.
            That's how Joe does it.
        That's a Person.

        Joe remembers alice as a new Person("Alice", nothing).
        Joe remembers bob as a new Person("Bob", alice).
        alice.friend turns into bob.

        alice.greet("Bob").
        bob.greet("Alice").
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("Alice awkward nods at Bob"));
    assert!(output.contains("Bob awkward nods at Alice"));
}
