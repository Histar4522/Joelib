use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Narrative Statement / Block Keywords
    JoeRemembers,             // "Joe remembers"
    JoeSets,                  // "Joe sets"
    TurnsInto,                // "turns into"
    JoeYells,                 // "Joe yells"
    JoeWhispers,              // "Joe whispers"
    JoeListensTo,             // "Joe listens to"
    IfJoeBelieves,            // "If Joe believes"
    Otherwise,                // "Otherwise:" or "Otherwise"
    ThatIsThat,               // "That is that." or "That is that"
    WhileJoeWorriesAbout,     // "While Joe worries about"
    ForEvery,                 // "For every"
    JoeMovesOn,               // "Joe moves on." or "Joe moves on"
    JoeGivesUp,               // "Joe gives up." or "Joe gives up"
    JoeTriesAnyway,           // "Joe tries anyway." or "Joe tries anyway"
    ToFunc,                   // "To"
    With,                     // "with"
    JoeDelivers,              // "Joe delivers"
    ThatsHowJoeDoesIt,        // "That's how Joe does it."
    MeetA,                    // "Meet a"
    ItHas,                    // "It has"
    ThatsA,                   // "That's a"
    Thats,                    // "That's"
    ANew,                     // "a new"
    JoePanics,                // "Joe panics"
    JoeAttempts,              // "Joe attempts:" or "Joe attempts"
    JoeBreaksDown,            // "Joe breaks down in tears with"
    JoeRecovers,              // "Joe recovers." or "Joe recovers"
    JoeCallsFor,              // "Joe calls for"
    JoeEstablishes,           // "Joe establishes"
    Chapter,                  // "Chapter"

    // Narrative Values & Modifiers
    As,                       // "as"
    In,                       // "in"
    To,                       // "to"
    From,                     // "from"
    JThis,                    // "jthis"
    JoeAgrees,                // "Joe agrees" (true)
    JoeDisagrees,             // "Joe disagrees" (false)
    Nothing,                  // "nothing" (nil)

    // Logical Operators
    And,                      // "and"
    Or,                       // "or"
    Not,                      // "not"

    // Literals & Identifiers
    Ident(String),
    Number(f64),
    Text(String),

    // Symbols & Punctuation
    Dot,                      // "."
    Colon,                    // ":"
    Comma,                    // ","
    LParen,                   // "("
    RParen,                   // ")"
    LBracket,                 // "["
    RBracket,                 // "]"
    LBrace,                   // "{"
    RBrace,                   // "}"
    Plus,                     // "+"
    Minus,                    // "-"
    Star,                     // "*"
    Slash,                    // "/"
    Percent,                  // "%"
    Eq,                       // "=="
    NotEq,                    // "!="
    Lt,                       // "<"
    LtEq,                     // "<="
    Gt,                       // ">"
    GtEq,                     // ">="
    Assign,                   // "="
    Arrow,                    // "->"

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
