// ------------------------------------------------------------------------------
//
//     _  __    _    __  __ ____  _   _ ____
//    | |/ /   / \  |  \/  |  _ \| | | / ___|
//    | ' /   / _ \ | |\/| | |_) | | | \___ \
//    | . \  / ___ \| |  | |  __/| |_| |___) |
//    |_|\_\/_/   \_\_|  |_|_|    \___/|____/
//
//    Knowledge Academic Modular Platform for Universal Study
//
// ------------------------------------------------------------------------------
//
// This file is part of Kampus - https://github.com/didactika/kampus
//
// Kampus is free software: you can redistribute it and/or modify it under the terms of the
// GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with Kampus.
// If not, see <https://www.gnu.org/licenses/>.
//
// @package   kampus-records
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Formulas of grades, without the database: an expression of numbers, references to other records written
//! `[[<record id>]]`, the four operations, parentheses and a few functions, parsed into a tree that lists
//! the records it reads and computes exactly with decimals. A reference without a grade, or a division by
//! zero, gives no result rather than a wrong one.

use std::collections::BTreeSet;

use rust_decimal::{Decimal, RoundingStrategy};

/// The longest expression, in characters.
pub const MAX_LENGTH: usize = 2000;

/// The deepest nesting of parentheses and calls.
const MAX_DEPTH: usize = 32;

/// The functions a formula may call.
pub const FUNCTIONS: &[&str] = &["min", "max", "sum", "average", "round", "floor", "ceil", "abs"];

/// What is wrong with an expression, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// What is wrong: `too_long`, `unexpected`, `unknown_function`, `arguments`, `reference`, `too_deep`.
    pub rule: &'static str,
    /// The position, in characters, where it was found.
    pub at: usize,
}

/// A parsed expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A number.
    Number(Decimal),
    /// The grade of another record.
    Reference(uuid::Uuid),
    /// The opposite of an expression.
    Negate(Box<Expr>),
    /// Two expressions joined by an operator.
    Binary(Box<Expr>, Op, Box<Expr>),
    /// A function applied to its arguments.
    Call(&'static str, Vec<Expr>),
}

/// An operator between two expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Addition.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
    /// Division.
    Divide,
}

/// A piece of an expression.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// A number.
    Number(Decimal),
    /// A reference to a record.
    Reference(uuid::Uuid),
    /// The name of a function.
    Name(String),
    /// One of `+ - * / ( ) ,`.
    Symbol(char),
}

/// Cuts an expression into tokens with their positions.
fn tokens(text: &str) -> Result<Vec<(Token, usize)>, ParseError> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let c = chars[at];
        if c.is_whitespace() {
            at += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let start = at;
            while at < chars.len() && (chars[at].is_ascii_digit() || chars[at] == '.') {
                at += 1;
            }
            let number: String = chars[start..at].iter().collect();
            let value = number.parse::<Decimal>().map_err(|_| ParseError {
                rule: "unexpected",
                at: start,
            })?;
            found.push((Token::Number(value), start));
        } else if c == '[' {
            let start = at;
            let close = (at..chars.len().saturating_sub(1))
                .find(|&i| chars[i] == ']' && chars[i + 1] == ']')
                .filter(|_| chars.get(at + 1) == Some(&'['))
                .ok_or(ParseError {
                    rule: "reference",
                    at: start,
                })?;
            let inside: String = chars[at + 2..close].iter().collect();
            let id = uuid::Uuid::parse_str(inside.trim()).map_err(|_| ParseError {
                rule: "reference",
                at: start,
            })?;
            found.push((Token::Reference(id), start));
            at = close + 2;
        } else if c.is_ascii_alphabetic() {
            let start = at;
            while at < chars.len() && chars[at].is_ascii_alphanumeric() {
                at += 1;
            }
            found.push((
                Token::Name(chars[start..at].iter().collect::<String>().to_lowercase()),
                start,
            ));
        } else if "+-*/(),".contains(c) {
            found.push((Token::Symbol(c), at));
            at += 1;
        } else {
            return Err(ParseError {
                rule: "unexpected",
                at,
            });
        }
    }
    Ok(found)
}

/// Reads tokens into a tree, by precedence.
struct Parser {
    /// The tokens.
    tokens: Vec<(Token, usize)>,
    /// The next one.
    next: usize,
    /// How deep the parser is.
    depth: usize,
    /// The length of the text, for an error at its end.
    end: usize,
}

impl Parser {
    /// The position of the next token.
    fn here(&self) -> usize {
        self.tokens.get(self.next).map_or(self.end, |(_, at)| *at)
    }

    /// Takes the next token when it is a symbol.
    fn eat(&mut self, symbol: char) -> bool {
        if matches!(self.tokens.get(self.next), Some((Token::Symbol(s), _)) if *s == symbol) {
            self.next += 1;
            true
        } else {
            false
        }
    }

    /// A sum of terms.
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(ParseError {
                rule: "too_deep",
                at: self.here(),
            });
        }
        let mut left = self.term()?;
        loop {
            let op = if self.eat('+') {
                Op::Add
            } else if self.eat('-') {
                Op::Subtract
            } else {
                break;
            };
            left = Expr::Binary(Box::new(left), op, Box::new(self.term()?));
        }
        self.depth -= 1;
        Ok(left)
    }

    /// A product of factors.
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.factor()?;
        loop {
            let op = if self.eat('*') {
                Op::Multiply
            } else if self.eat('/') {
                Op::Divide
            } else {
                break;
            };
            left = Expr::Binary(Box::new(left), op, Box::new(self.factor()?));
        }
        Ok(left)
    }

    /// A number, a reference, a call, a group or the opposite of one.
    fn factor(&mut self) -> Result<Expr, ParseError> {
        if self.eat('-') {
            return Ok(Expr::Negate(Box::new(self.factor()?)));
        }
        let at = self.here();
        let Some((token, _)) = self.tokens.get(self.next).cloned() else {
            return Err(ParseError {
                rule: "unexpected",
                at,
            });
        };
        self.next += 1;
        match token {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Reference(id) => Ok(Expr::Reference(id)),
            Token::Symbol('(') => {
                let inner = self.expression()?;
                if self.eat(')') {
                    Ok(inner)
                } else {
                    Err(ParseError {
                        rule: "unexpected",
                        at: self.here(),
                    })
                }
            }
            Token::Name(name) => {
                let function = FUNCTIONS.iter().find(|f| **f == name).ok_or(ParseError {
                    rule: "unknown_function",
                    at,
                })?;
                if !self.eat('(') {
                    return Err(ParseError {
                        rule: "unexpected",
                        at: self.here(),
                    });
                }
                let mut arguments = vec![self.expression()?];
                while self.eat(',') {
                    arguments.push(self.expression()?);
                }
                if !self.eat(')') {
                    return Err(ParseError {
                        rule: "unexpected",
                        at: self.here(),
                    });
                }
                let fits = match *function {
                    "round" => (1..=2).contains(&arguments.len()),
                    "floor" | "ceil" | "abs" => arguments.len() == 1,
                    _ => true,
                };
                if fits {
                    Ok(Expr::Call(function, arguments))
                } else {
                    Err(ParseError {
                        rule: "arguments",
                        at,
                    })
                }
            }
            Token::Symbol(_) => Err(ParseError {
                rule: "unexpected",
                at,
            }),
        }
    }
}

/// Parses an expression.
///
/// # Errors
///
/// A [`ParseError`] with the rule it breaks and where.
pub fn parse(text: &str) -> Result<Expr, ParseError> {
    if text.chars().count() > MAX_LENGTH {
        return Err(ParseError {
            rule: "too_long",
            at: MAX_LENGTH,
        });
    }
    let mut parser = Parser {
        tokens: tokens(text)?,
        next: 0,
        depth: 0,
        end: text.chars().count(),
    };
    let expr = parser.expression()?;
    if parser.next < parser.tokens.len() {
        return Err(ParseError {
            rule: "unexpected",
            at: parser.here(),
        });
    }
    Ok(expr)
}

impl Expr {
    /// The records the expression reads.
    #[must_use]
    pub fn references(&self) -> BTreeSet<uuid::Uuid> {
        let mut found = BTreeSet::new();
        self.collect(&mut found);
        found
    }

    /// Adds the records this expression reads.
    fn collect(&self, found: &mut BTreeSet<uuid::Uuid>) {
        match self {
            Self::Number(_) => {}
            Self::Reference(id) => {
                found.insert(*id);
            }
            Self::Negate(inner) => inner.collect(found),
            Self::Binary(left, _, right) => {
                left.collect(found);
                right.collect(found);
            }
            Self::Call(_, arguments) => {
                for argument in arguments {
                    argument.collect(found);
                }
            }
        }
    }

    /// The value, reading the grade of each record from `grade`; none when a record has no grade, a
    /// division is by zero or a number overflows.
    pub fn evaluate(&self, grade: &impl Fn(uuid::Uuid) -> Option<Decimal>) -> Option<Decimal> {
        match self {
            Self::Number(n) => Some(*n),
            Self::Reference(id) => grade(*id),
            Self::Negate(inner) => inner.evaluate(grade).map(|v| -v),
            Self::Binary(left, op, right) => {
                let (l, r) = (left.evaluate(grade)?, right.evaluate(grade)?);
                match op {
                    Op::Add => l.checked_add(r),
                    Op::Subtract => l.checked_sub(r),
                    Op::Multiply => l.checked_mul(r),
                    Op::Divide => l.checked_div(r),
                }
            }
            Self::Call(function, arguments) => {
                let values: Option<Vec<Decimal>> = arguments.iter().map(|a| a.evaluate(grade)).collect();
                call(function, &values?)
            }
        }
    }
}

/// Applies a function to its values.
fn call(function: &str, values: &[Decimal]) -> Option<Decimal> {
    match function {
        "min" => values.iter().copied().min(),
        "max" => values.iter().copied().max(),
        "sum" => values.iter().try_fold(Decimal::ZERO, |a, v| a.checked_add(*v)),
        "average" => {
            let total = values.iter().try_fold(Decimal::ZERO, |a, v| a.checked_add(*v))?;
            total.checked_div(Decimal::from(values.len()))
        }
        "round" => {
            let places = values.get(1).copied().unwrap_or(Decimal::ZERO);
            let places = u32::try_from(places.trunc().mantissa())
                .ok()
                .filter(|p| *p <= 5)?;
            Some(values[0].round_dp_with_strategy(places, RoundingStrategy::MidpointAwayFromZero))
        }
        "floor" => Some(values[0].floor()),
        "ceil" => Some(values[0].ceil()),
        "abs" => Some(values[0].abs()),
        _ => None,
    }
}
