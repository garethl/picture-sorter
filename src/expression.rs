// {key|altkey|3rdkey...|nthkey:format}

use std::ops::Add;
use std::path;

use crate::format::format;
use crate::picture::Picture;
use anyhow::anyhow;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref PATH_SEPARATOR_REGEX: Regex = Regex::new("[\\\\/]").unwrap();
    static ref PATH_SEPARATOR: String = path::MAIN_SEPARATOR.to_string();
}

#[derive(Debug)]
pub struct Expression {
    expressions: Vec<ExpressionChunk>,
}

impl Expression {
    pub fn new(format: &str) -> Self {
        Expression {
            expressions: extract_expressions(format),
        }
    }

    pub fn execute(&self, picture: &Picture) -> Result<String> {
        let mut buffer = String::new();

        for expression in &self.expressions {
            buffer = buffer.add(&expression.execute(picture)?);
        }

        // coerce the path separators to the platform ones
        buffer = PATH_SEPARATOR_REGEX
            .replace_all(&buffer, &*PATH_SEPARATOR)
            .to_string();

        Ok(buffer)
    }
}

#[derive(Debug, PartialEq)]
enum ExpressionChunk {
    Literal(String),
    Value(ValueFunction),
}

impl ExpressionChunk {
    fn execute<'a>(&self, picture: &Picture) -> Result<String> {
        match self {
            ExpressionChunk::Literal(value) => Ok(value.into()),
            ExpressionChunk::Value(function) => function.execute(picture),
        }
    }
}

#[derive(Debug, PartialEq)]
struct ValueFunction {
    keys: Vec<String>,
    format: Option<String>,
}

impl ValueFunction {
    fn execute(&self, picture: &Picture) -> Result<String> {
        let value = self.get_value_internal(picture)?;

        Ok(value)
    }

    fn get_value_internal(&self, picture: &Picture) -> Result<String> {
        let value = self.keys.iter().filter_map(|k| picture.get(k)).next();

        if value.is_none() {
            return Err(anyhow!(
                "Unable to find matching key from {}.",
                self.keys.join(", ")
            ));
        }

        let value = value.unwrap();

        match &self.format {
            Some(format_string) => format(format_string, &value),
            None => Ok(value),
        }
    }
}

impl ValueFunction {
    fn new(expression: &Vec<char>) -> ValueFunction {
        let colon_index = index_of_next(expression, 0, ':');
        let mut format: Option<String> = None;

        let keys = match colon_index {
            None => expression
                .split(|c| *c == ':')
                .map(String::from_iter)
                .collect(),
            Some(_) => {
                format = Some(expression[(colon_index.unwrap() + 1)..].iter().collect());
                expression[0..colon_index.unwrap()]
                    .split(|c| *c == '|')
                    .map(String::from_iter)
                    .collect()
            }
        };

        ValueFunction { keys, format }
    }
}

#[derive(PartialEq)]
enum State {
    Literal,
    Expression,
}

fn extract_expressions(format: &str) -> Vec<ExpressionChunk> {
    let mut expressions: Vec<ExpressionChunk> = vec![];
    let format: Vec<char> = format.chars().collect();

    let mut buffer = Vec::new();
    let mut i = 0;
    let mut state = State::Literal;
    loop {
        let c = format[i];

        match c {
            '{' => {
                if i + 1 < format.len() && format[i + 1] == '{' {
                    // escaped
                    i += 1;
                    buffer.push('{');
                    buffer.push('{');
                    continue;
                } else {
                    if !buffer.is_empty() {
                        expressions.push(ExpressionChunk::Literal(buffer.iter().collect()));
                        buffer.clear();
                    }
                    state = State::Expression;
                }
            }
            '}' => {
                if state == State::Literal {
                    buffer.push(c)
                } else if i + 1 < format.len() && format[i + 1] == '}' {
                    // escaped
                    i += 1;
                    buffer.push('}');
                    buffer.push('}');
                    continue;
                } else {
                    if !buffer.is_empty() {
                        expressions.push(ExpressionChunk::Value(ValueFunction::new(&buffer)));
                        buffer.clear();
                    }
                    state = State::Literal;
                }
            }
            _ => {
                buffer.push(c);
            }
        }

        i += 1;
        if i >= format.len() {
            break;
        }
    }

    if !buffer.is_empty() {
        match state {
            State::Literal => expressions.push(ExpressionChunk::Literal(buffer.iter().collect())),
            State::Expression => {
                expressions.push(ExpressionChunk::Value(ValueFunction::new(&buffer)))
            }
        }
    }

    expressions
}

fn index_of_next(value: &Vec<char>, start: usize, c: char) -> Option<usize> {
    let mut i = start;
    while i < value.len() {
        if value[i] == c {
            return Some(i);
        }
        i += 1;
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_literal() {
        let formatter = Expression::new("test");
        println!("{:?}", formatter);
        assert_eq!(
            vec![ExpressionChunk::Literal("test".to_string())],
            formatter.expressions
        );
    }

    #[test]
    fn single_expression() {
        let formatter = Expression::new("{test}");
        println!("{:?}", formatter);
        assert_eq!(
            vec![ExpressionChunk::Value(ValueFunction {
                format: None,
                keys: vec!["test".to_string()]
            })],
            formatter.expressions
        );
    }

    #[test]
    fn literal_wrapped() {
        let formatter = Expression::new("{test}/and/{test}");
        println!("{:?}", formatter);
        assert_eq!(
            vec![
                ExpressionChunk::Value(ValueFunction {
                    format: None,
                    keys: vec!["test".to_string()]
                }),
                ExpressionChunk::Literal("/and/".to_string()),
                ExpressionChunk::Value(ValueFunction {
                    format: None,
                    keys: vec!["test".to_string()]
                })
            ],
            formatter.expressions
        );
    }

    #[test]
    fn expression_wrapped() {
        let formatter = Expression::new("test/{and}/test");
        println!("{:?}", formatter);
        assert_eq!(
            vec![
                ExpressionChunk::Literal("test/".to_string()),
                ExpressionChunk::Value(ValueFunction {
                    format: None,
                    keys: vec!["and".to_string()]
                }),
                ExpressionChunk::Literal("/test".to_string()),
            ],
            formatter.expressions
        );
    }

    #[test]
    fn adjacent_expressions() {
        let formatter = Expression::new("{test}{and}{test2}");
        println!("{:?}", formatter);
        assert_eq!(
            vec![
                ExpressionChunk::Value(ValueFunction {
                    format: None,
                    keys: vec!["test".to_string()]
                }),
                ExpressionChunk::Value(ValueFunction {
                    format: None,
                    keys: vec!["and".to_string()]
                }),
                ExpressionChunk::Value(ValueFunction {
                    format: None,
                    keys: vec!["test2".to_string()]
                }),
            ],
            formatter.expressions
        );
    }
    #[test]
    fn format_values() {
        let formatter = Expression::new("{test:%Y}/{test2:%Y-%M}");
        println!("{:?}", formatter);
        assert_eq!(
            vec![
                ExpressionChunk::Value(ValueFunction {
                    format: Some("%Y".to_string()),
                    keys: vec!["test".to_string()]
                }),
                ExpressionChunk::Literal("/".to_string()),
                ExpressionChunk::Value(ValueFunction {
                    format: Some("%Y-%M".to_string()),
                    keys: vec!["test2".to_string()]
                }),
            ],
            formatter.expressions
        );
    }
}
