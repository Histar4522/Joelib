use crate::token::{Span, Token, TokenKind};

pub struct Tokenizer<'a> {
    src: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            chars: src.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments()?;
            if self.is_eof() {
                tokens.push(Token::new(TokenKind::Eof, self.current_span()));
                break;
            }

            let _span = self.current_span();
            let token = self.next_token()?;
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn current_span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.pos + 1 < self.chars.len() {
            Some(self.chars[self.pos + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn remaining_str(&self) -> &str {
        if self.pos < self.chars.len() {
            // Find char boundary in src
            let char_offset = self.src.chars().take(self.pos).map(|c| c.len_utf8()).sum();
            &self.src[char_offset..]
        } else {
            ""
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        while !self.is_eof() {
            if let Some(ch) = self.peek() {
                if ch.is_whitespace() {
                    self.advance();
                    continue;
                }
                if ch == '/' {
                    if let Some('/') = self.peek_next() {
                        // Single-line comment
                        while let Some(c) = self.peek() {
                            self.advance();
                            if c == '\n' {
                                break;
                            }
                        }
                        continue;
                    } else if let Some('*') = self.peek_next() {
                        // Multi-line comment
                        self.advance(); // consume '/'
                        self.advance(); // consume '*'
                        let mut closed = false;
                        while !self.is_eof() {
                            if self.peek() == Some('*') && self.peek_next() == Some('/') {
                                self.advance();
                                self.advance();
                                closed = true;
                                break;
                            }
                            self.advance();
                        }
                        if !closed {
                            return Err(format!("Unterminated block comment at line {}", self.line));
                        }
                        continue;
                    }
                }
            }
            break;
        }
        Ok(())
    }

    fn match_phrase(&mut self, phrase: &str) -> bool {
        let rem = self.remaining_str();
        if rem.starts_with(phrase) {
            // Check word boundary if phrase ends with alphanumeric
            let next_char = rem[phrase.len()..].chars().next();
            if let Some(last_ch) = phrase.chars().last() {
                if last_ch.is_alphanumeric() || last_ch == '_' {
                    if let Some(nc) = next_char {
                        if nc.is_alphanumeric() || nc == '_' {
                            return false;
                        }
                    }
                }
            }
            for _ in 0..phrase.chars().count() {
                self.advance();
            }
            true
        } else {
            false
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let span = self.current_span();

        // 1. Try matching long narrative phrases first
        let phrases: &[(&str, TokenKind)] = &[
            ("Joe breaks down in tears with", TokenKind::JoeBreaksDown),
            ("While Joe worries about", TokenKind::WhileJoeWorriesAbout),
            ("That's how Joe does it.", TokenKind::ThatsHowJoeDoesIt),
            ("That's how Joe does it", TokenKind::ThatsHowJoeDoesIt),
            ("Joe tries anyway.", TokenKind::JoeTriesAnyway),
            ("Joe tries anyway", TokenKind::JoeTriesAnyway),
            ("If Joe believes", TokenKind::IfJoeBelieves),
            ("Joe listens to", TokenKind::JoeListensTo),
            ("Joe calls for", TokenKind::JoeCallsFor),
            ("Joe establishes", TokenKind::JoeEstablishes),
            ("Chapter", TokenKind::Chapter),
            ("Joe remembers", TokenKind::JoeRemembers),
            ("Joe whispers", TokenKind::JoeWhispers),
            ("Joe disagrees", TokenKind::JoeDisagrees),
            ("Joe moves on.", TokenKind::JoeMovesOn),
            ("Joe moves on", TokenKind::JoeMovesOn),
            ("Joe gives up.", TokenKind::JoeGivesUp),
            ("Joe gives up", TokenKind::JoeGivesUp),
            ("Joe recovers.", TokenKind::JoeRecovers),
            ("Joe recovers", TokenKind::JoeRecovers),
            ("Joe attempts:", TokenKind::JoeAttempts),
            ("Joe attempts", TokenKind::JoeAttempts),
            ("That is that.", TokenKind::ThatIsThat),
            ("That is that", TokenKind::ThatIsThat),
            ("Joe delivers", TokenKind::JoeDelivers),
            ("Joe panics", TokenKind::JoePanics),
            ("Joe agrees", TokenKind::JoeAgrees),
            ("turns into", TokenKind::TurnsInto),
            ("Otherwise:", TokenKind::Otherwise),
            ("Otherwise", TokenKind::Otherwise),
            ("Joe yells", TokenKind::JoeYells),
            ("Joe sets", TokenKind::JoeSets),
            ("For every", TokenKind::ForEvery),
            ("That's a", TokenKind::ThatsA),
            ("That's", TokenKind::Thats),
            ("Meet a", TokenKind::MeetA),
            ("It has", TokenKind::ItHas),
            ("a new", TokenKind::ANew),
        ];

        for (phrase, kind) in phrases {
            if self.match_phrase(phrase) {
                return Ok(Token::new(kind.clone(), span));
            }
        }

        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(Token::new(TokenKind::Eof, span)),
        };

        // 2. Operators & Punctuation
        if ch == '-' && self.peek_next() == Some('>') {
            self.advance();
            self.advance();
            return Ok(Token::new(TokenKind::Arrow, span));
        }
        if ch == '=' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token::new(TokenKind::Eq, span));
        }
        if ch == '!' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token::new(TokenKind::NotEq, span));
        }
        if ch == '<' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token::new(TokenKind::LtEq, span));
        }
        if ch == '>' && self.peek_next() == Some('=') {
            self.advance();
            self.advance();
            return Ok(Token::new(TokenKind::GtEq, span));
        }

        match ch {
            '.' => { self.advance(); Ok(Token::new(TokenKind::Dot, span)) }
            ':' => { self.advance(); Ok(Token::new(TokenKind::Colon, span)) }
            ',' => { self.advance(); Ok(Token::new(TokenKind::Comma, span)) }
            '(' => { self.advance(); Ok(Token::new(TokenKind::LParen, span)) }
            ')' => { self.advance(); Ok(Token::new(TokenKind::RParen, span)) }
            '[' => { self.advance(); Ok(Token::new(TokenKind::LBracket, span)) }
            ']' => { self.advance(); Ok(Token::new(TokenKind::RBracket, span)) }
            '{' => { self.advance(); Ok(Token::new(TokenKind::LBrace, span)) }
            '}' => { self.advance(); Ok(Token::new(TokenKind::RBrace, span)) }
            '+' => { self.advance(); Ok(Token::new(TokenKind::Plus, span)) }
            '-' => { self.advance(); Ok(Token::new(TokenKind::Minus, span)) }
            '*' => { self.advance(); Ok(Token::new(TokenKind::Star, span)) }
            '/' => { self.advance(); Ok(Token::new(TokenKind::Slash, span)) }
            '%' => { self.advance(); Ok(Token::new(TokenKind::Percent, span)) }
            '<' => { self.advance(); Ok(Token::new(TokenKind::Lt, span)) }
            '>' => { self.advance(); Ok(Token::new(TokenKind::Gt, span)) }
            '=' => { self.advance(); Ok(Token::new(TokenKind::Assign, span)) }
            '"' => self.scan_string(),
            _ if ch.is_ascii_digit() => self.scan_number(),
            _ if ch.is_alphabetic() || ch == '_' => self.scan_identifier_or_keyword(),
            _ => {
                self.advance();
                Err(format!("Unexpected character '{}' at {}", ch, span))
            }
        }
    }

    fn scan_string(&mut self) -> Result<Token, String> {
        let span = self.current_span();
        self.advance(); // consume opening quote '"'
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance(); // consume closing quote
                return Ok(Token::new(TokenKind::Text(text), span));
            }
            if ch == '\\' {
                self.advance();
                match self.advance() {
                    Some('n') => text.push('\n'),
                    Some('t') => text.push('\t'),
                    Some('r') => text.push('\r'),
                    Some('\\') => text.push('\\'),
                    Some('"') => text.push('"'),
                    Some(other) => {
                        text.push('\\');
                        text.push(other);
                    }
                    None => return Err(format!("Unterminated string escape at {}", span)),
                }
            } else {
                text.push(ch);
                self.advance();
            }
        }

        Err(format!("Unclosed string literal starting at {}", span))
    }

    fn scan_number(&mut self) -> Result<Token, String> {
        let span = self.current_span();
        let mut num_str = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('.') {
            if let Some(next) = self.peek_next() {
                if next.is_ascii_digit() {
                    num_str.push('.');
                    self.advance(); // consume '.'
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_digit() {
                            num_str.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        match num_str.parse::<f64>() {
            Ok(n) => Ok(Token::new(TokenKind::Number(n), span)),
            Err(e) => Err(format!("Invalid number '{}' at {}: {}", num_str, span, e)),
        }
    }

    fn scan_identifier_or_keyword(&mut self) -> Result<Token, String> {
        let span = self.current_span();
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match ident.as_str() {
            "To" => TokenKind::ToFunc,
            "to" => TokenKind::To,
            "from" => TokenKind::From,
            "with" => TokenKind::With,
            "as" => TokenKind::As,
            "in" => TokenKind::In,
            "jthis" => TokenKind::JThis,
            "nothing" => TokenKind::Nothing,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            _ => TokenKind::Ident(ident),
        };

        Ok(Token::new(kind, span))
    }
}
