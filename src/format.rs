use anyhow::Result;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use dateparser::parse;
use interim::{parse_date_string, Dialect};
use lazy_static::lazy_static;

lazy_static! {
    static ref NAIVE_FORMATS: Vec<&'static str> = vec!["%Y:%m:%d %H:%M:%S"];
}

pub fn format(format: &str, value: &str) -> Result<String> {
    let date_time = try_parse_date_time(value);

    if date_time.is_none() {
        return Err(anyhow::anyhow!("Unable to parse {} as a date/time.", value));
    }
    let date_time: DateTime<Local> = DateTime::from(date_time.unwrap());

    Ok(format!("{}", date_time.format(format)))
}

fn try_parse_date_time(value: &str) -> Option<DateTime<Utc>> {
    // start with dateparser, which can recognise a lot
    if let Ok(value) = parse(value) {
        return Some(value);
    }

    // try our other fallback libraries / formats:
    if let Some(value) = NAIVE_FORMATS
        .iter()
        .filter_map(|fmt| NaiveDateTime::parse_from_str(value, fmt).ok())
        .next()
    {
        match value.and_local_timezone(Local) {
            chrono::LocalResult::Single(value) => return Some(DateTime::from(value)),
            chrono::LocalResult::Ambiguous(value, _) => return Some(DateTime::from(value)),
            chrono::LocalResult::None => {}
        };
    }

    // try interim as the last resort
    if let Ok(value) = parse_date_string(value, Local::now(), Dialect::Uk) {
        return Some(DateTime::from(value));
    }

    // give up
    None
}
