use joelang::interpreter::Interpreter;
use joelang::parser::Parser;
use joelang::tokenizer::Tokenizer;
use joelang::value::JoeValue;
use std::fs;

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
fn test_keyword_override_with_class() {
    let script = r#"
        Meet a CustomSpeaker:
            It has name, log.
            To yell with msg:
                jthis.log = jthis.log + "[LOUD: " + msg + "]".
            That's how Joe does it.
            To whisper with msg:
                jthis.log = jthis.log + "[SOFT: " + msg + "]".
            That's how Joe does it.
        That's a CustomSpeaker.

        Joe remembers speaker as a new CustomSpeaker("Megaphone", "").
        Joe yells "Attention everyone!" to speaker.
        Joe whispers "psst quiet" to speaker.

        Joe yells "Captured log: " + speaker.log.
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("[LOUD: Attention everyone!]"));
    assert!(output.contains("[SOFT: psst quiet]"));
}

#[test]
fn test_yell_and_whisper_to_file() {
    let log_file = "test_yell_whisper.tmp";
    let script = format!(r#"
        Joe yells "First line" to "{}".
        Joe whispers "Second line " to "{}".
        Joe whispers "continued" to "{}".
    "#, log_file, log_file, log_file);

    let (res, _) = run_joe(&script);
    assert!(res.is_ok());

    let content = fs::read_to_string(log_file).expect("Failed to read created file");
    assert!(content.contains("First line\n"));
    assert!(content.contains("Second line continued"));
    let _ = fs::remove_file(log_file);
}

#[test]
fn test_module_declaration() {
    let script = r#"
        Joe establishes Calculator:
            Joe remembers base as 10.
            To add_base with x:
                Joe delivers x + base.
            That's how Joe does it.
        That's Calculator.

        Joe yells "Base is: " + Calculator.base.
        Joe remembers result as Calculator.add_base(5).
        Joe yells "Result is: " + result.
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("Base is: 10"));
    assert!(output.contains("Result is: 15"));
}

#[test]
fn test_operator_overloading_in_class() {
    let script = r#"
        Meet a Vector:
            It has x, y.
            To add with other:
                Joe delivers a new Vector(jthis.x + other.x, jthis.y + other.y).
            That's how Joe does it.
        That's a Vector.

        Joe remembers v1 as a new Vector(1, 2).
        Joe remembers v2 as a new Vector(10, 20).
        Joe remembers v3 as v1 + v2.

        Joe yells "v3.x = " + v3.x + ", v3.y = " + v3.y.
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("v3.x = 11, v3.y = 22"));
}
