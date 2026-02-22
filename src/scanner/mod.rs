use std::fmt::{Debug, Display, Formatter};

use log::debug;

pub fn scan(source: &str) -> impl Iterator<Item = Token> {
    let scanner = Scanner::new(source.to_string());
    scanner.scan()
}

struct Scanner {
    errors: Vec<ScannerError>,
    source: Vec<char>,  // todo i am not sure that this is rust way
    tokens: Vec<Token>, // todo i dont want to store all then return, i want to return iterator
    start: usize,
    offset: usize,
    line: usize,
    character: usize,
}

impl Scanner {
    fn new(source: String) -> Scanner {
        Scanner {
            errors: vec![],
            source: source.chars().collect(),
            tokens: vec![],
            start: 0,
            offset: 0,
            line: 1,
            character: 0,
        }
    }

    fn scan(mut self) -> impl Iterator<Item = Token> {
        let mut result = vec![];
        while !self.is_at_end() {
            self.start = self.offset;
            if let Some(token) = self.scan_token() {
                result.push(token);
            }
        }
        result.push(Token::eof(self.line, self.character + 1));
        result.into_iter()
    }

    fn scan_token(&mut self) -> Option<Token> {
        let c = self.advance();
        match c {
            '(' => self.token(TokenType::LeftParen),
            ')' => self.token(TokenType::RightParen),
            '{' => self.token(TokenType::LeftBrace),
            '}' => self.token(TokenType::RightBrace),
            ',' => self.token(TokenType::Comma),
            '.' => self.token(TokenType::Dot),
            '-' => self.token(TokenType::Minus),
            '+' => self.token(TokenType::Plus),
            ';' => self.token(TokenType::Semicolon),
            '*' => self.token(TokenType::Star),
            '!' => self.match_token_or('=', TokenType::BangEqual, TokenType::Bang),
            '=' => self.match_token_or('=', TokenType::EqualEqual, TokenType::Equal),
            '<' => self.match_token_or('=', TokenType::LessEqual, TokenType::Less),
            '>' => self.match_token_or('=', TokenType::GreaterEqual, TokenType::Greater),
            '/' => {
                if self.check('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    None
                } else {
                    self.token(TokenType::Slash)
                }
            },
            ' ' | '\r' | '\t' => None,
            '\n' => {
                self.next_line();
                None
            },
            '"' => self.string(),
            _ => {
                self.error("", format!("Unknown character '{}'", c).as_str());
                None
            },
        }
    }

    fn string(&mut self) -> Option<Token> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.next_line();
            }
            self.advance();
        }
        if self.is_at_end() {
            self.error("", "Unterminated string.");
            return None;
        }
        self.advance();
        Some(Token::new(
            TokenType::String,
            Some(LiteralValue::String(
                self.source[(self.start + 1)..(self.offset - 1)].iter().collect(),
            )),
            self.line,
            self.character,
        ))
    }
}

/// Token manipulation and utility functions
impl Scanner {
    fn next_line(&mut self) {
        self.line += 1;
        self.character = 0;
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.offset] }
    }

    fn check(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.offset] != expected {
            return false;
        }
        self.advance();
        true
    }

    fn match_token_or(
        &mut self,
        expected: char,
        token_type_on_match: TokenType,
        or: TokenType,
    ) -> Option<Token> {
        if self.check(expected) { self.token(token_type_on_match) } else { self.token(or) }
    }

    fn token(&mut self, token_type: TokenType) -> Option<Token> {
        Some(Token::token(token_type, self.line, self.character))
    }

    fn is_at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let result = self.source[self.offset];
        self.offset += 1;
        self.character += 1;
        result
    }
}

/// Error handling
impl Scanner {
    fn error(&mut self, location: &str, message: &str) {
        let error = ScannerError {
            location: location.to_string(),
            line: self.line,
            character: self.character,
            message: message.to_string(),
        };
        debug!("Encountered error: {}.", error);
        self.errors.push(error);
    }
}

struct ScannerError {
    location: String,
    line: usize,
    character: usize,
    message: String,
}

impl Display for ScannerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}:{}] {}", self.location, self.line, self.character, self.message)
    }
}

pub struct Token {
    token_type: TokenType,
    literal: Option<LiteralValue>,
    line: usize,
    character: usize,
}

impl Token {
    fn new(
        token_type: TokenType,
        literal: Option<LiteralValue>,
        line: usize,
        character: usize,
    ) -> Token {
        Token {
            token_type,
            literal,
            line,
            character,
        }
    }

    fn token(token_type: TokenType, line: usize, character: usize) -> Token {
        Token::new(token_type, None, line, character)
    }

    fn eof(line: usize, character: usize) -> Token {
        Token::new(TokenType::Eof, None, line, character)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.literal {
            None => write!(f, "{:?}({:?},{:?})", self.token_type, self.line, self.character),
            Some(literal) => write!(
                f,
                "{:?}({:?},{:?})[{:?}]",
                self.token_type, self.line, self.character, literal
            ),
        }
    }
}

#[derive(PartialEq)]
pub enum LiteralValue {
    Number(f64),
    String(String),
}

impl Debug for LiteralValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Number(n) => write!(f, "{}", n),
            LiteralValue::String(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_token_debug() {
        init_logger();
        let token =
            Token::new(TokenType::Identifier, Some(LiteralValue::String("test".to_string())), 1, 1);
        assert_eq!(format!("{:?}", token), "Identifier(1,1)[test]");
    }

    #[test]
    fn test_many_tokens() {
        init_logger();
        let source = "// this is a comment
(( )){} // grouping stuff
!*+-/=<> <= == // operators";

        let tokens: Vec<Token> = scan(source).collect();
        assert_eq!(tokens.len(), 17);
        assert_eq!(tokens.last().unwrap().token_type, TokenType::Eof);
    }

    #[test]
    fn test_string() {
        init_logger();
        let source = "\"Hello, world!\"";
        let tokens: Vec<Token> = scan(source).collect();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].literal, Some(LiteralValue::String("Hello, world!".to_string())));
    }
}
