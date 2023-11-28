use anyhow::{Context, Result};
use log::debug;
use regex::Regex;
use std::ops::Add;

pub fn build_exclusion_filter(exclusions: Vec<String>) -> impl Fn(&str) -> bool {
    let exclusions: Result<Vec<Regex>> = exclusions
        .into_iter()
        .map(build_regex)
        .collect();

    let exclusions = exclusions.unwrap();

    move |d| exclusions.iter().any(|exclusion| exclusion.is_match(d))
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
                if !buffer.is_empty() {
                    result = result.add(&regex::escape(&buffer));
                    buffer.truncate(0);
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

    if !buffer.is_empty() {
        result = result.add(&regex::escape(&buffer));
    }

    debug!("Compiled regex exclusion: {}", &result);

    Regex::new(&result).context("Error compiling exclusion regex")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() -> Result<()> {
        let regex = build_regex(".trashed-*".to_string()).unwrap();

        assert_eq!("\\.trashed\\-.*", regex.as_str());

        Ok(())
    }
}
