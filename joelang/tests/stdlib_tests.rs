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
fn test_math_module() {
    let script = r#"
        Joe calls for std.math.
        Joe remembers root as std.math.sqrt(16).
        Joe remembers power as std.math.pow(2, 3).
        Joe remembers sine as std.math.sin(0).
        Joe yells "Root: " + root + ", Pow: " + power + ", Sin: " + sine.
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("Root: 4, Pow: 8, Sin: 0"));
}

#[test]
fn test_strings_module() {
    let script = r#"
        Joe calls for std.strings.
        Joe remembers raw as "   hello, world of joe!   ".
        Joe remembers clean as std.strings.trim(raw).
        Joe remembers upper as std.strings.to_upper(clean).
        Joe remembers parts as std.strings.split(clean, ", ").
        Joe yells "Upper: " + upper.
        Joe yells "First part: " + parts[0].
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("Upper: HELLO, WORLD OF JOE!"));
    assert!(output.contains("First part: hello"));
}

#[test]
fn test_collections_module() {
    let script = r#"
        Joe calls for std.collections.
        Joe remembers items as [3, 1, 2].
        std.collections.push(items, 4).
        Joe remembers sorted as std.collections.sort(items).
        Joe yells "Sorted: " + sorted.

        Joe remembers bag as {"name": "Joe", "age": 30}.
        Joe remembers has_name as std.collections.has_key(bag, "name").
        Joe yells "Has name: " + has_name.
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("Sorted: [1, 2, 3, 4]"));
    assert!(output.contains("Has name: Joe agrees"));
}

#[test]
fn test_file_io() {
    let test_file = "test_joe_diary.tmp";
    let script = format!(r#"
        Joe calls for std.file.
        std.file.write("{}", "Joe was here.\nSecond line.\n").
        Joe remembers exists as std.file.exists("{}").
        Joe remembers content as std.file.read("{}").
        std.file.remove("{}").
        Joe yells "Exists: " + exists.
        Joe yells "Content: " + content.
    "#, test_file, test_file, test_file, test_file);

    let (res, output) = run_joe(&script);
    assert!(res.is_ok());
    assert!(output.contains("Exists: Joe agrees"));
    assert!(output.contains("Content: Joe was here."));
}

#[test]
fn test_json_module() {
    let script = r#"
        Joe calls for std.json.
        Joe remembers raw as "{\"mood\": \"caffeinated\", \"cups\": 3}".
        Joe remembers parsed as std.json.parse(raw).
        Joe yells "Mood: " + parsed.mood.
        Joe yells "Cups: " + parsed.cups.
        Joe remembers re_json as std.json.stringify(parsed).
    "#;

    let (res, output) = run_joe(script);
    assert!(res.is_ok());
    assert!(output.contains("Mood: caffeinated"));
    assert!(output.contains("Cups: 3"));
}
