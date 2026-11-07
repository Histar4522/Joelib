use joelang::JoeLang;

#[test]
fn test_checker_detects_errors() {
    let script = r#"
        Joe gives up.
        Joe delivers 42.
        Joe sets ghost to 10.
    "#;

    let joe = JoeLang::new();
    let diagnostics = joe.check(script).expect("Check failed");

    assert!(diagnostics.iter().any(|d| d.message.contains("Existential Error")));
    assert!(diagnostics.iter().any(|d| d.message.contains("Confusion Error")));
    assert!(diagnostics.iter().any(|d| d.message.contains("Amnesia Error")));
}

#[test]
fn test_checker_detects_unused_warning() {
    let script = r#"
        Joe remembers savings as 500.
        Joe yells "Living without savings!".
    "#;

    let joe = JoeLang::new();
    let diagnostics = joe.check(script).expect("Check failed");

    assert!(diagnostics.iter().any(|d| d.is_warning && d.message.contains("never thought about it again")));
}

#[test]
fn test_explainer_produces_documentary() {
    let script = r#"
        Joe remembers coffee as 3.
        While Joe worries about coffee > 0:
            coffee turns into coffee - 1.
            Joe yells "Sip!".
        Joe moves on.
    "#;

    let joe = JoeLang::new();
    let explanation = joe.explain(script).expect("Explain failed");

    assert!(explanation.contains("The Chronicles of Joe: An Analytical Exposition"));
    assert!(explanation.contains("Joe commits 'coffee' to memory"));
    assert!(explanation.contains("Joe enters an obsessive cycle"));
}
