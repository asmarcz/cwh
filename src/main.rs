use std::io;
use std::io::Write;
use std::str::{FromStr, SplitWhitespace};

use crate::BinaryOperator::{Division, Minus, Multiplication, Plus};
use crate::UnaryOperator::{Abs, Factorial, Negative, Predecessor, Signum, Successor};
use crate::Value::{BinaryOperation, Int, UnaryOperation, Variable};

#[derive(Copy, Clone, Debug, PartialEq)]
enum BinaryOperator {
    Division,
    Minus,
    Multiplication,
    Plus,
}

impl FromStr for BinaryOperator {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "/" => Ok(Division),
            "-" => Ok(Minus),
            "*" => Ok(Multiplication),
            "+" => Ok(Plus),
            _ => Err(()),
        }
    }
}

fn factorial(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum UnaryOperator {
    Abs,
    Factorial,
    Negative,
    Predecessor,
    Signum,
    Successor,
}

impl FromStr for UnaryOperator {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "abs" => Ok(Abs),
            "fact" | "!" => Ok(Factorial),
            "neg" => Ok(Negative),
            "pred" => Ok(Predecessor),
            "sgn" => Ok(Signum),
            "succ" => Ok(Successor),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Value {
    BinaryOperation {
        operator: BinaryOperator,
        left: Box<Value>,
        right: Box<Value>,
    },
    Int(isize),
    UnaryOperation {
        operator: UnaryOperator,
        arg: Box<Value>,
    },
    Variable(usize),
}

fn parse_value(iter: &mut SplitWhitespace) -> Result<Value, String> {
    match iter.next() {
        None => Err(String::from("Expected arguments at the end of input.")),
        Some(str) => match str {
            var if var.chars().nth(0).unwrap_or_default() == '$' => {
                match (&var[1..]).parse::<usize>() {
                    Ok(idx) => Ok(Variable(idx)),
                    Err(_) => Err(format!(
                        "Expected valid number as a variable name, instead got '{}'.",
                        var
                    )),
                }
            }
            num if num.parse::<isize>().is_ok() => Ok(Int(num.parse::<isize>().unwrap())),
            op if BinaryOperator::from_str(op).is_ok() => match (parse_value(iter), parse_value(iter)) {
                (Ok(left), Ok(right)) => Ok(BinaryOperation {
                    operator: BinaryOperator::from_str(op).unwrap(),
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                (Err(_), _) | (_, Err(_)) => {
                    Err(format!("Binary operator '{}' expected two arguments.", op))
                }
            },
            op if UnaryOperator::from_str(op).is_ok() => match parse_value(iter) {
                Ok(value) => Ok(UnaryOperation {
                    operator: UnaryOperator::from_str(op).unwrap(),
                    arg: Box::new(value),
                }),
                Err(_) => Err(format!("Unary operator '{}' expected an argument.", op))
            }
            _ => Err(format!("Unexpected input '{}'.", str)),
        },
    }
}

fn evaluate_value(value: &Value, variables: &[isize]) -> Result<isize, String> {
    match value {
        BinaryOperation { operator, left, right } => {
            match (evaluate_value(left, variables), evaluate_value(right, variables)) {
                (Ok(lhs), Ok(rhs)) => {
                    match operator {
                        Division => {
                            if rhs == 0 {
                                Err(String::from("Division by zero."))
                            } else { Ok(lhs / rhs) }
                        }
                        Minus => Ok(lhs - rhs),
                        Multiplication => Ok(lhs * rhs),
                        Plus => Ok(lhs + rhs),
                    }
                }
                (Err(msg), _) | (_, Err(msg)) => Err(msg),
            }
        }
        Int(int) => Ok(*int),
        UnaryOperation { operator, arg } => {
            match evaluate_value(arg, variables) {
                Ok(int) => match operator {
                    Abs => Ok(int.abs()),
                    Negative => Ok(-int),
                    Factorial => {
                        if int.is_positive() {
                            Ok(factorial(int as usize) as isize)
                        } else {
                            Err(String::from("Expected a non-negative number as an! argument to factorial."))
                        }
                    }
                    Predecessor => Ok(int - 1),
                    Signum => Ok(int.signum()),
                    Successor => Ok(int + 1),
                }
                Err(msg) => Err(msg),
            }
        }
        Variable(idx) => match variables.get(*idx) {
            None => Err(format!("Invalid variable index '{}'.", idx)),
            Some(int) => Ok(*int),
        },
    }
}

fn process_line(line: String, history: &[isize]) -> Result<isize, String> {
    let mut iter = line.split_whitespace();
    match parse_value(&mut iter) {
        Ok(value) => match iter.next() {
            None => evaluate_value(&value, history),
            Some(str) => Err(format!("Expected end of line, instead found '{}'.", str)),
        },
        Err(msg) => Err(msg),
    }
}

fn new_prompt() {
    print!("# ");
    io::stdout().flush().unwrap();
}

fn main() {
    let mut history: Vec<isize> = Vec::new();
    new_prompt();
    for line in io::stdin().lines() {
        match process_line(line.unwrap(), &history) {
            Ok(result) => {
                history.push(result);
                println!("{}", result);
            }
            Err(msg) => eprintln!("Error: {}", msg),
        }
        new_prompt();
    }
}

#[cfg(test)]
mod tests {
    mod parser {
        use crate::*;

        fn to_result(str: &str) -> Result<Value, String> {
            let mut iter = str.split_whitespace();
            parse_value(&mut iter)
        }

        #[test]
        fn expressions() {
            assert_eq!(to_result("+ 3 2"), Ok(BinaryOperation {
                operator: Plus,
                left: Box::new(Int(3)),
                right: Box::new(Int(2)),
            }));

            assert_eq!(to_result("+ 3 * 8 / 2 3"), Ok(BinaryOperation {
                operator: Plus,
                left: Box::new(Int(3)),
                right: Box::new(BinaryOperation {
                    operator: Multiplication,
                    left: Box::new(Int(8)),
                    right: Box::new(BinaryOperation {
                        operator: Division,
                        left: Box::new(Int(2)),
                        right: Box::new(Int(3)),
                    }),
                }),
            }));
        }

        #[test]
        fn variables() {
            assert_eq!(to_result("- $0 $1"), Ok(BinaryOperation {
                operator: Minus,
                left: Box::new(Variable(0)),
                right: Box::new(Variable(1)),
            }));
        }

        #[test]
        fn errors() {
            assert_eq!(to_result(""),
                       Err(String::from("Expected arguments at the end of input.")));
            assert_eq!(to_result("$a"),
                       Err(String::from("Expected valid number as a variable name, instead got '$a'.")));
            assert_eq!(to_result("* 1"),
                       Err(String::from("Binary operator '*' expected two arguments.")));
            assert_eq!(to_result("!#"),
                       Err(String::from("Unexpected input '!#'.")));
        }
    }

    mod evaluator {
        use crate::*;

        fn to_result(str: &str) -> Result<isize, String> {
            let mut history: Vec<isize> = Vec::new();
            for line in str.lines() {
                let mut iter = line.split_whitespace();
                match evaluate_value(&parse_value(&mut iter).unwrap(), &history) {
                    Ok(int) => history.push(int),
                    Err(msg) => return Err(msg),
                }
            }
            Ok(*history.last().unwrap())
        }

        #[test]
        fn priority() {
            assert_eq!(to_result("* + 3 - 2 1 / 16 4"), Ok(16))
        }

        #[test]
        fn sequence() {
            assert_eq!(
                to_result(r#"
                    + 3 2
                    * 2 5
                    / $1 $0
                "#.trim()),
                Ok(2),
            )
        }
    }
}
