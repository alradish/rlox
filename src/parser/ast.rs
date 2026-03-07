use crate::{
    lox_ast,
    scanner::{LiteralValue, Token},
};

lox_ast!(
    Expression {
        Binary(
            left: Box<Expression>,
            operator: Token,
            right: Box<Expression>,
        ),
        Grouping(
            expression: Box<Expression>,
        ),
        Literal(
            value: LiteralValue,
        ),
        Unary(
            operator: Token,
            right: Box<Expression>,
        ),
    }
);

#[derive(Default)]
pub struct PrettyPrinter {
    /// Whether to use parentheses to clearly show order of operations.
    clear: bool,
}

impl PrettyPrinter {
    pub fn clear() -> Self {
        PrettyPrinter {
            clear: true,
        }
    }
}

impl ExpressionVisitor<String, ()> for PrettyPrinter {
    fn visit_binary(&self, expr: &BinaryExpr) -> Result<String, ()> {
        let left = expr.left.accept(self)?;
        let operator = expr.operator.lexeme.clone();
        let right = expr.right.accept(self)?;
        if self.clear {
            return Ok(format!("({} {} {})", left, operator, right));
        }
        Ok(format!("{} {} {}", left, operator, right))
    }

    fn visit_grouping(&self, expr: &GroupingExpr) -> Result<String, ()> {
        let expression = expr.expression.accept(self)?;
        Ok(format!("({})", expression))
    }

    fn visit_literal(&self, expr: &LiteralExpr) -> Result<String, ()> {
        let value = match expr.value {
            LiteralValue::String(ref s) => format!("\"{}\"", s),
            LiteralValue::Number(n) => format!("{}", n),
            LiteralValue::Boolean(b) => format!("{}", b),
            LiteralValue::Nil => "nil".to_string(),
        };
        Ok(value)
    }

    fn visit_unary(&self, expr: &UnaryExpr) -> Result<String, ()> {
        let operator = expr.operator.lexeme.clone();
        let right = expr.right.accept(self)?;
        if self.clear {
            return Ok(format!("({}{})", operator, right));
        }
        Ok(format!("{}{}", operator, right))
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter::Trace;

    use super::*;
    use crate::scanner::TokenType;

    fn init_logger() {
        let _ = env_logger::builder().is_test(false).filter_level(Trace).try_init();
    }

    #[test]
    fn test_print() {
        init_logger();
        let expr = Expression::Binary(BinaryExpr {
            left: Box::new(Expression::Literal(LiteralExpr {
                value: LiteralValue::String("Hello".to_string()),
            })),
            operator: Token::new(TokenType::Plus, "+".to_string(), None, 0, 0),
            right: Box::new(Expression::Literal(LiteralExpr {
                value: LiteralValue::String("World".to_string()),
            })),
        });
        let result = expr.accept(&PrettyPrinter::default());
        assert_eq!(result.unwrap(), "\"Hello\" + \"World\"");
    }
}
