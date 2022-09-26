use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::ops::Add;

pub fn build_exclusion_filter(exclusions: Vec<String>) -> impl Fn(&str) -> bool {
    let exclusions: Result<Vec<Regex>> = exclusions
        .into_iter()
        .map(|exclusion| build_regex(exclusion))
        .collect();

    let exclusions = exclusions.unwrap();

    return move |d| exclusions.iter().any(|exclusion| exclusion.is_match(d));
}

fn build_regex(exclusion: String) -> Result<Regex> {
    let mut result = String::new();

    let mut buffer = String::new();
    let mut escaped = false;
    for c in exclusion.chars() {
        match c {
            '\\' => {
                if escaped {
                    buffer.push(c);
                    escaped = false
                } else {
                    escaped = true
                }
            }
            '*' => {
                if buffer.len() > 0 {
                    result = result.add(&regex::escape(&buffer));
                }
                result = result.add(".*");
            }
            _ => {
                if escaped {
                    buffer.push('\\');
                }
                buffer.push(c);
            }
        }
    }

    if buffer.len() > 0 {
        result = result.add(&regex::escape(&buffer));
    }

    Regex::new(&result).context("Error compiling exclusion regex")
}
