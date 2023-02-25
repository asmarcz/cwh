use std::io;
use std::io::Write;
use std::str::{FromStr, SplitWhitespace};

use crate::Operator::{Division, Minus, Multiplication, Plus};
use crate::Value::{BinaryOperation, Int, Variable};

#[derive(Copy, Clone, Debug, PartialEq)]
enum Operator {
    Division,
    Minus,
    Multiplication,
    Plus,
}

impl FromStr for Operator {
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

#[derive(Debug, PartialEq)]
enum Value {
    BinaryOperation {
        operator: Operator,
        left: Box<Value>,
        right: Box<Value>,
    },
    Int(isize),
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
            op if Operator::from_str(op).is_ok() => match (parse_value(iter), parse_value(iter)) {
                (Ok(left), Ok(right)) => Ok(BinaryOperation {
                    operator: Operator::from_str(op).unwrap(),
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                (Err(_), _) | (_, Err(_)) => {
                    Err(format!("Binary operator '{}' expected two arguments.", op))
                }
            },
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
        Variable(idx) => match variables.get(*idx) {
            None => Err(format!("Invalid variable index '{}'.", idx)),
            Some(int) => Ok(*int),
        }
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
}
