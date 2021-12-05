pub struct Expr<'a> {
    expr_stack: Vec<&'a str>,
}

#[derive(PartialEq)]
enum ExprSym {
    Operand,
    Operator,
}

#[derive(Debug)]
pub enum ExprError {
    ParseOperandError,
    ParseExprInvalidOperandError,
    ParseExprInvalidOperatorError,
    ParseOperatorError,
}

pub type ExprResult = Result<isize, ExprError>;

impl<'a> Expr<'a> {
    pub fn new(expr_stack: Vec<&'a str>) -> Self {
        Self { expr_stack }
    }

    pub fn eval(&self) -> ExprResult {
        let mut curr = ExprSym::Operand;
        let mut curr_stack = vec![0isize, 0, 0];
        let mut j = 0;
        let mut e = 0;
        let mut i;
        while e < self.expr_stack.len() {
            i = self.expr_stack[e];
            eprintln!("j:{}, i: {}, stack: {:?}", j, i, curr_stack);
            if j == 0 {
                match curr {
                    ExprSym::Operand if e == 0 => {
                        curr_stack[j] = Self::parse_operand(i)?;
                        curr = ExprSym::Operand;
                    }
                    ExprSym::Operator => {
                        return Err(ExprError::ParseExprInvalidOperandError);
                    }
                    _ => {
                        j += 1;
                        curr_stack[j] = Self::parse_operand(i)?;
                        curr = ExprSym::Operator;
                    }
                }
            } else if j == 1 {
                match curr {
                    ExprSym::Operand => {
                        curr_stack[j] = Self::parse_operand(i)?;
                        curr = ExprSym::Operator;
                    }
                    ExprSym::Operator => {
                        return Err(ExprError::ParseExprInvalidOperandError);
                    }
                }
            } else if j == 2 {
                match curr {
                    ExprSym::Operand => {
                        return Err(ExprError::ParseExprInvalidOperatorError);
                    }
                    ExprSym::Operator => {
                        curr_stack[j] = Self::parse_operator(i)?;
                    }
                }
            } else if j == 3 {
                curr_stack[0] = Self::run(&curr_stack);
                j = 0;
                curr = ExprSym::Operand;
                continue;
            }
            j += 1;
            e += 1;
        }
        if j == 3 {
            curr_stack[0] = Self::run(&curr_stack);
        }
        Ok(curr_stack[0])
    }
    fn run(stack: &Vec<isize>) -> isize {
        match stack[2] {
            0 => stack[0] + stack[1],
            1 => stack[0] - stack[1],
            2 => stack[0] * stack[1],
            3 => stack[0] / stack[1],
            4 => stack[0].rem_euclid(stack[1]),
            _ => stack[0],
        }
    }

    fn parse_operand(curr: &str) -> ExprResult {
        let operand = match isize::from_str_radix(curr, 10) {
            Ok(n) => Ok(n),
            _ => Err(ExprError::ParseOperandError),
        };
        operand
    }

    fn parse_operator(curr: &str) -> ExprResult {
        let operator = match curr {
            "+" => Ok(0),
            "-" => Ok(1),
            "*" => Ok(2),
            "/" => Ok(3),
            "%" => Ok(4),
            _ => Err(ExprError::ParseOperatorError),
        };
        operator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_test() {
        let expr = vec!["1", "2", "+"];
        let expr = Expr::new(expr);
        assert_eq!(expr.eval().unwrap(), 3);

        let expr = vec!["1", "2", "-"];
        let expr = Expr::new(expr);
        assert_eq!(expr.eval().unwrap(), -1);

        let expr = vec!["1", "2", "*"];
        let expr = Expr::new(expr);
        assert_eq!(expr.eval().unwrap(), 2);

        let expr = vec!["2", "3", "+", "5", "-"];
        let expr = Expr::new(expr);
        assert_eq!(expr.eval().unwrap(), 0);

        let expr = vec!["2", "3", "+", "5", "*", "2", "+", "3", "-", "6", "/"];
        let expr = Expr::new(expr);
        assert_eq!(expr.eval().unwrap(), 4);
        let expr = vec!["2", "3", "*", "4", "+", "5", "%"];
        let expr = Expr::new(expr);
        assert_eq!(expr.eval().unwrap(), 0);
    }
}
