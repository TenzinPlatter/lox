use crate::expr::Expr;

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
            Expr::Binary {
                left,
                right,
                operator,
            } => {
                parenthize(operator.lexeme.clone(), &[left, right])?
            }
            Expr::Grouping { expression } => {
                parenthize("group".to_string(), &[expression])?
            }
            Expr::Literal { value } => match value {
                Some(object) => {
                    // TODO: ig this will need dynamic dispatch to work for any type/user defined
                    // types and classes?
                    if let Some(object) = object.downcast_ref::<f64>() {
                        object.to_string()
                    } else {
                        "object".to_string()
                    }
                }
                None => "nil".to_string(),
            },
            Expr::Unary { operator, right } => {
                parenthize(operator.lexeme.to_string(), &[right])?
            }
        })
    }
}
