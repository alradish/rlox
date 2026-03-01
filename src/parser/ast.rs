use crate::{lox_ast, scanner::Token};

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

#[derive(Debug)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Nil,
}

struct PrettyPrinter;

impl ExpressionVisitor<String, ()> for PrettyPrinter {
    fn visit_binary(&self, expr: &BinaryExpr) -> Result<String, ()> {
        let left = expr.left.accept(self)?;
        let operator = expr.operator.lexeme.clone();
        let right = expr.right.accept(self)?;
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
            LiteralValue::Nil => "nil".to_string(),
        };
        Ok(value)
    }

    fn visit_unary(&self, expr: &UnaryExpr) -> Result<String, ()> {
        let operator = expr.operator.lexeme.clone();
        let right = expr.right.accept(self)?;
        Ok(format!("({} {})", operator, right))
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
            operator: Token::new(TokenType::Plus, "+".to_string(), 0, 0),
            right: Box::new(Expression::Literal(LiteralExpr {
                value: LiteralValue::String("World".to_string()),
            })),
        });
        let result = expr.accept(&PrettyPrinter);
        assert_eq!(result.unwrap(), "\"Hello\" + \"World\"");
    }
}
