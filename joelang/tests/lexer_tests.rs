use joelang::token::TokenKind;
use joelang::tokenizer::Tokenizer;

#[test]
fn test_tokenize_narrative_story() {
    let script = r#"
        // Chapter 1: Coffee
        Joe remembers coffee as 42.
        While Joe worries about coffee > 0:
            coffee turns into coffee - 1.
            Joe yells "Joe drank a cup!".
        Joe moves on.

        If Joe believes coffee == 0:
            Joe yells "No more coffee.".
        Otherwise:
            Joe whispers "How is this possible?".
        That is that.

        To sip with times:
            Joe delivers times * 2.
        That's how Joe does it.

        Joe calls for std.net.
        Joe panics "Out of caffeine!".
    "#;

    let mut tokenizer = Tokenizer::new(script);
    let tokens = tokenizer.tokenize().expect("Failed to tokenize");

    assert!(tokens.iter().any(|t| t.kind == TokenKind::JoeRemembers));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::WhileJoeWorriesAbout));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::TurnsInto));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::JoeYells));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::JoeMovesOn));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::IfJoeBelieves));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Otherwise));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::ThatIsThat));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::ToFunc));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::JoeDelivers));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::ThatsHowJoeDoesIt));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::JoeCallsFor));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::JoePanics));
}

#[test]
fn test_tokenize_classes_and_dumpster_cycles() {
    let script = r#"
        Meet a Person:
            It has name, friend.
            To greet with other:
                Joe yells jthis.name + " greets " + other.
            That's how Joe does it.
        That's a Person.

        Joe remembers alice as a new Person("Alice", nothing).
        Joe remembers bob as a new Person("Bob", alice).
        alice.friend turns into bob.
    "#;

    let mut tokenizer = Tokenizer::new(script);
    let tokens = tokenizer.tokenize().expect("Failed to tokenize");

    assert!(tokens.iter().any(|t| t.kind == TokenKind::MeetA));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::ItHas));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::ThatsA));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::ANew));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::JThis));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Nothing));
}
