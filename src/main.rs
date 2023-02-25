use std::io;
use std::str::{FromStr, SplitWhitespace};

use crate::Operator::{Division, Minus, Multiplication, Plus};
use crate::Value::{BinaryOperation, Int, Variable};

#[derive(Copy, Clone)]
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

fn process_line(line: String, history: &mut Vec<isize>) -> Result<(), String> {
    let mut iter = line.split_whitespace();
    match parse_value(&mut iter) {
        Ok(value) => match iter.next() {
            None => Ok(()),
            Some(str) => Err(format!("Expected end of line, instead found '{}'.", str)),
        },
        Err(msg) => Err(msg),
    }
}

fn main() {
    let mut history: Vec<isize> = Vec::new();
    for line in io::stdin().lines() {
        match line {
            Ok(line) => match process_line(line, &mut history) {
                Err(msg) => eprintln!("Error: {}", msg),
                _ => {}
            },
            Err(e) => eprintln!("{}", e),
        }
    }
}
