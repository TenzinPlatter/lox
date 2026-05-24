use crate::expr::{Expr, Object};

impl Expr {
    pub fn pretty_print_ast(&self) -> anyhow::Result<String> {
        let parenthize = |name: String, exprs: &[&Expr]| -> anyhow::Result<String> {
            let mut result = format!("({}", name);
            for expr in exprs {
                result += &format!(" {}", expr.pretty_print_ast()?);
            }
            result.push(')');
            Ok(result)
        };

        Ok(match self {
            Expr::Ternary {
                condition,
                left,
                right,
            } => parenthize("?:".to_string(), &[condition, left, right])?,
            Expr::Binary {
                left,
                right,
                operator,
            } => parenthize(operator.lexeme.clone(), &[left, right])?,
            Expr::Grouping { expression } => parenthize("group".to_string(), &[expression])?,
            Expr::Literal { value } => value.to_string(),
            Expr::Unary { operator, right } => parenthize(operator.lexeme.to_string(), &[right])?,
        })
    }
}
