use crate::{
    parser::ast::Expression,
    scanner::{LiteralValue, Token, TokenType},
};

pub mod ast;
mod ast_macro;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn parse(&self) {}

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_equality()
    }

    /// equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn parse_equality(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_comparison()?;
        while self.check_and_consume_any(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let token = self.previous().clone();
            let right = self.parse_comparison()?;
            expression = Expression::binary(Box::new(expression), token, Box::new(right));
        }
        Ok(expression)
    }

    /// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn parse_comparison(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_term()?;
        while self.check_and_consume_any(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let token = self.previous().clone();
            let right = self.parse_term()?;
            expression = Expression::binary(Box::new(expression), token, Box::new(right));
        }
        Ok(expression)
    }

    /// term → factor ( ( "-" | "+" ) factor )* ;
    fn parse_term(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_factor()?;
        while self.check_and_consume_any(&[TokenType::Minus, TokenType::Plus]) {
            let token = self.previous().clone();
            let right = self.parse_factor()?;
            expression = Expression::binary(Box::new(expression), token, Box::new(right));
        }
        Ok(expression)
    }

    /// factor → unary ( ( "/" | "*" ) unary )* ;
    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.parse_unary()?;
        while self.check_and_consume_any(&[TokenType::Slash, TokenType::Star]) {
            let token = self.previous().clone();
            let right = self.parse_unary()?;
            expression = Expression::binary(Box::new(expression), token, Box::new(right));
        }
        Ok(expression)
    }

    /// unary → ( "!" | "-" ) unary | primary ;
    fn parse_unary(&mut self) -> Result<Expression, ParserError> {
        if self.check_and_consume_any(&[TokenType::Bang, TokenType::Minus]) {
            let right = self.parse_unary()?;
            return Ok(Expression::unary(self.previous().clone(), Box::new(right)));
        }
        self.parse_primary()
    }

    /// primary → NUMBER | STRING | "true" | "false" | "nil"
    ///           | "(" expression ")" ;
    fn parse_primary(&mut self) -> Result<Expression, ParserError> {
        if self.check_and_consume(&TokenType::False) {
            return Ok(Expression::literal(LiteralValue::Boolean(false)));
        }
        if self.check_and_consume(&TokenType::True) {
            return Ok(Expression::literal(LiteralValue::Boolean(true)));
        }
        if self.check_and_consume(&TokenType::Nil) {
            return Ok(Expression::literal(LiteralValue::Nil));
        }

        if self.check_and_consume_any(&[TokenType::Number, TokenType::String]) {
            return Ok(Expression::literal(self.previous().literal.clone().unwrap()));
        }

        if self.check_and_consume(&TokenType::LeftParen) {
            let expression = self.parse_expression()?;
            self.consume(&TokenType::RightParen);
            return Ok(Expression::grouping(Box::new(expression)));
        }

        Err(ParserError::Heh)
    }
}

impl Parser {
    fn check_and_consume_any(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check_and_consume(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }
        false
    }

    fn consume(&mut self, token_type: &TokenType) -> &Token {
        if self.check(token_type) {
            return self.advance();
        }
        panic!("Expected token of type {:?}, but got {:?}.", token_type, self.peek());
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

#[derive(Debug)]
pub enum ParserError {
    Heh,
}

#[cfg(test)]
mod tests {
    use log::LevelFilter::Trace;

    use crate::parser::ast::PrettyPrinter;

    fn init_logger() {
        let _ = env_logger::builder().is_test(false).filter_level(Trace).try_init();
    }

    #[test]
    fn parse_expression() {
        init_logger();
        let tokens = crate::scanner::scan(
            "
            1 + 2 * (3 - 4) / 5 * 6;
            ",
        )
        .collect::<Vec<_>>();
        let mut parser = crate::parser::Parser {
            tokens,
            current: 0,
        };
        let result = parser.parse_expression().expect("Parsing failed.");
        let actual = result.accept(&PrettyPrinter::clear()).expect("Pretty printing failed.");
        assert_eq!(actual, "(1 + (((2 * ((3 - 4))) / 5) * 6))");
    }
}
