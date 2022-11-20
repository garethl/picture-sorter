use crate::metadata::{get_metadata, ExifMetadata};
use crate::Cache;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use walkdir::DirEntry;

lazy_static! {
    static ref MAPPINGS: HashMap<&'static str, Vec<&'static str>> = vec![(
        "datetime",
        vec![
            "DateTime",
            "DateTimeOriginal",
            "MediaCreateDate",
            "GPSDateTime",
            "DateTimeFromFileName"
        ]
    )]
    .into_iter()
    .collect();
}

type Generator = fn(&Picture) -> Option<String>;

lazy_static! {
    static ref GENERATORS: HashMap<&'static str, Generator> =
        vec![("datetime", get_date_time_from_filename as Generator)]
            .into_iter()
            .collect();
}

lazy_static! {
    //need to match something like: 20131231_212454 - phones name files like this
    static ref FILENAME_DATE_TIME: Regex = Regex::new("(\\d{4})(\\d{2})(\\d{2})_(\\d{2})(\\d{2})(\\d{2})").unwrap();
}
fn get_date_time_from_filename(picture: &Picture) -> Option<String> {
    let file_name = Path::new(&picture.short_path).file_name()?.to_str()?;

    let captures = FILENAME_DATE_TIME.captures(file_name)?;

    let date_time = format!(
        "{}-{}-{} {}:{}:{}",
        captures.get(1)?.as_str(),
        captures.get(2)?.as_str(),
        captures.get(3)?.as_str(),
        captures.get(4)?.as_str(),
        captures.get(5)?.as_str(),
        captures.get(6)?.as_str()
    );

    Some(date_time)
}

#[derive(Debug)]
pub struct Picture {
    pub path: String,
    pub dir_entry: DirEntry,
    pub short_path: String,
    pub metadata: ExifMetadata,
}

impl Picture {
    pub fn get(&self, key: &str) -> Option<String> {
        let value = if let Some(values) = MAPPINGS.get(&*key.to_lowercase()) {
            for value in values {
                if let Some(value) = self.get_internal(*value) {
                    return Some(value.to_string());
                }
            }
            None
        } else {
            self.get_internal(key)
        };

        if None == value {
            if let Some(generator) = GENERATORS.get(&*key.to_lowercase()) {
                return generator(self);
            }
        }
        return value.map(|i| i.to_string());
    }

    fn get_internal(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

#[derive(Debug)]
pub struct PictureError {
    pub dir_entry: DirEntry,
    pub short_path: String,
    pub error: anyhow::Error,
}

impl Display for PictureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PictureError({:?}, {})", self.dir_entry, self.error)
    }
}

impl Error for PictureError {}

impl Picture {
    pub fn from_dir_entry(
        source_path: &str,
        dir_entry: DirEntry,
        cache: Cache,
    ) -> Result<Picture, PictureError> {
        let path = dir_entry.path();
        let path_string = path.to_str().unwrap();

        let short_path: String = path_string.chars().skip(source_path.len() + 1).collect();

        let metadata = cache
            .get(path_string, || get_metadata(path))
            .map_err(|err| PictureError {
                short_path: short_path.clone(),
                dir_entry: dir_entry.clone(),
                error: err,
            })?;

        Ok(Picture {
            path: path_string.to_string(),
            short_path,
            dir_entry,
            metadata,
        })
    }
}
