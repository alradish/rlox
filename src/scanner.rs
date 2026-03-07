use std::fmt::{Debug, Display, Formatter};

use log::debug;
use serde::{Deserialize, Serialize};

pub fn scan(source: &str) -> impl Iterator<Item = Token> {
    Scanner::scan_string(source.to_string()).tokens.into_iter()
}

pub struct Scanner {
    errors: Vec<ScannerError>,
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    offset: usize,
    line: usize,
    character: usize,
}

/// Token scanning
impl Scanner {
    pub fn scan_string(source: String) -> Scanner {
        // hack until I decide what to do with scan(&mut self)
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan().collect();
        scanner.tokens = tokens;
        scanner
    }

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

    // todo should fill inner state?
    fn scan(&mut self) -> impl Iterator<Item = Token> {
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

    pub fn get_tokens(&self) -> Vec<Token> {
        self.tokens.clone()
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
                } else if self.check('*') {
                    loop {
                        let peek = self.peek();
                        if peek == '\n' {
                            self.next_line();
                        } else if peek == '*' && self.peek_next() == '/' {
                            self.advance();
                            self.advance();
                            break;
                        }
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
            c if is_digit(c) => self.number(),
            c if is_alpha(c) => self.identifier(),
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
        let value: String = self.source[(self.start + 1)..(self.offset - 1)].iter().collect();
        self.token_with_literal(TokenType::String, LiteralValue::String(value))
    }

    fn number(&mut self) -> Option<Token> {
        while is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();
            while is_digit(self.peek()) {
                self.advance();
            }
        }
        let lexeme = self.lexeme();
        let Ok(number) = lexeme.parse::<f64>() else {
            self.error("", format!("Failed to parse number {}.", lexeme).as_str());
            return None;
        };

        self.token_with_literal(TokenType::Number, LiteralValue::Number(number))
    }

    fn identifier(&mut self) -> Option<Token> {
        while is_alpha(self.peek()) || is_digit(self.peek()) || self.peek() == '_' {
            self.advance();
        }
        let identifier_string = self.lexeme();
        let token_type = TokenType::keyword(&identifier_string).unwrap_or(TokenType::Identifier);
        self.token(token_type)
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic()
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

    fn peek_next(&self) -> char {
        if self.offset + 1 >= self.source.len() { '\0' } else { self.source[self.offset + 1] }
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

    fn lexeme(&self) -> String {
        self.source[self.start..self.offset].iter().collect()
    }

    fn token(&mut self, token_type: TokenType) -> Option<Token> {
        Some(Token::new(token_type, self.lexeme(), None, self.line, self.character))
    }

    fn token_with_literal(
        &mut self,
        token_type: TokenType,
        literal: LiteralValue,
    ) -> Option<Token> {
        Some(Token::new(token_type, self.lexeme(), Some(literal), self.line, self.character))
    }

    fn is_at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        let result = self.source[self.offset];
        self.offset += 1;
        self.character += 1;
        result
    }
}

/// Error handling
/// TODO: delegate to error accumulator later
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

    pub fn get_errors(&self) -> Vec<ScannerError> {
        self.errors.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerError {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line: usize,
    pub character: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<LiteralValue>,
        line: usize,
        character: usize,
    ) -> Token {
        Token {
            token_type,
            lexeme,
            literal,
            line,
            character,
        }
    }

    pub fn eof(line: usize, character: usize) -> Token {
        Token::new(TokenType::Eof, "".to_string(), None, line, character)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(literal) = &self.literal {
            write!(
                f,
                "{:?}({:?},{:?})'{}'[{:?}]",
                self.token_type, self.line, self.character, self.lexeme, literal
            )
        } else {
            write!(
                f,
                "{:?}({:?},{:?})'{}'",
                self.token_type, self.line, self.character, self.lexeme
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
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

impl TokenType {
    fn keyword(identifier: &str) -> Option<TokenType> {
        match identifier {
            "and" => Some(TokenType::And),
            "class" => Some(TokenType::Class),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "for" => Some(TokenType::For),
            "fun" => Some(TokenType::Fun),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "print" => Some(TokenType::Print),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }
}

pub fn pretty(tokens: &Vec<Token>) -> String {
    let mut string_builder = String::new();
    let mut current_line = 1;
    for token in tokens {
        if token.line > current_line {
            string_builder.push('\n');
            current_line = token.line;
        }
        string_builder.push_str(&format!("[{:?}]", match &token.literal {
            Some(LiteralValue::String(s)) => s.clone(),
            Some(LiteralValue::Number(n)) => format!("{:?}", n),
            _ => format!("{:?}", token.token_type),
        }));
    }
    string_builder
}

#[cfg(test)]
mod tests {
    use log::LevelFilter::Trace;

    use super::*;

    fn init_logger() {
        let _ = env_logger::builder().is_test(false).filter_level(Trace).try_init();
    }

    #[test]
    fn test_token_debug() {
        init_logger();
        let token = Token::new(
            TokenType::Identifier,
            "test".to_string(),
            Some(LiteralValue::String("test".to_string())),
            1,
            1,
        );
        assert_eq!(format!("{:?}", token), "Identifier(1,1)'test'[String(\"test\")]");
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

    #[test]
    fn test_number() {
        init_logger();
        let source = "123.456";
        let tokens: Vec<Token> = scan(source).collect();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Number);
        assert_eq!(tokens[0].literal, Some(LiteralValue::Number(123.456)));
    }

    #[test]
    fn test_identifier() {
        init_logger();
        let source = "identifier";
        let tokens: Vec<Token> = scan(source).collect();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Identifier);
    }

    #[test]
    fn test_keyword() {
        init_logger();
        let source = "and";
        let tokens: Vec<Token> = scan(source).collect();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::And);
    }

    #[test]
    fn test_multiline_comment() {
        init_logger();
        let source = "/* this is a comment */
        /* this is also a comment
        but this is a multiline comment */";
        let tokens: Vec<Token> = scan(source).collect();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Eof);
        assert_eq!(tokens[0].line, 3);
    }

    #[test]
    fn test_lexeme() {
        init_logger();
        let source = "var x = \"hello\"; 123.456 + !=";
        let tokens: Vec<Token> = scan(source).collect();

        assert_eq!(tokens[0].token_type, TokenType::Var);
        assert_eq!(tokens[0].lexeme, "var");

        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].lexeme, "x");

        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[2].lexeme, "=");

        assert_eq!(tokens[3].token_type, TokenType::String);
        assert_eq!(tokens[3].literal, Some(LiteralValue::String("hello".to_string())));
        assert_eq!(tokens[3].lexeme, "\"hello\"");

        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[4].lexeme, ";");

        assert_eq!(tokens[5].token_type, TokenType::Number);
        assert_eq!(tokens[5].literal, Some(LiteralValue::Number(123.456)));
        assert_eq!(tokens[5].lexeme, "123.456");

        assert_eq!(tokens[6].token_type, TokenType::Plus);
        assert_eq!(tokens[6].lexeme, "+");

        assert_eq!(tokens[7].token_type, TokenType::BangEqual);
        assert_eq!(tokens[7].lexeme, "!=");
    }
}
