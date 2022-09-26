use crate::exclusion::build_exclusion_filter;
use crate::picture::Picture;
use crate::{Cache, Expression};
use anyhow::Error;
use dpc_pariter::IteratorExt;
use log::{debug, error, info, warn};
use regex::Regex;
use std::fs;
use std::ops::Add;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub fn sort(
    cache: Cache,
    expression: Expression,
    source: String,
    destination: String,
    exclusions: Vec<String>,
) -> Result<(), Error> {
    let exclusion_filter = build_exclusion_filter(exclusions);
    debug!("Reading from {}", source);

    let source = Path::new(&source).canonicalize()?;
    let source_path = source.to_str().unwrap().to_string();
    let source_path2 = source.to_str().unwrap().to_string();

    let files = WalkDir::new(source)
        .follow_links(true)
        .into_iter()
        .filter_entry(move |entry| !exclusion_filter(entry.path().to_str().unwrap()))
        .filter_map(move |entry| match entry {
            Ok(e) => Some(e),
            Err(err) => {
                match err.path() {
                    None => warn!("Unknown error finding files. {}", err),
                    Some(path) => warn!(
                        "Error reading {}, skipping. {}",
                        short_path_path(&source_path, &path),
                        err
                    ),
                };
                None
            }
        })
        .filter(|e| e.file_type().is_file())
        .parallel_map(move |entry: DirEntry| Picture::from_dir_entry(&source_path2, entry, cache));

    for file in files {
        let picture = match file {
            Ok(file) => file,
            Err(err) => {
                error!(
                    "Error reading metadata for `{}`. Ignoring. {}",
                    err.short_path, err.error
                );
                continue;
            }
        };

        if let Some(error) = picture.get("Error") {
            warn!(
                "Ignoring `{}`, cannot extract exif data because: `{}`",
                picture.short_path, error
            );
            continue;
        }

        debug!("Processing {}", picture.short_path);

        match expression.execute(&picture) {
            Ok(name) => {
                info!("Would rename {} to {}", picture.short_path, name)
            }
            Err(err) => warn!(
                "Skipping {}, unable to apply name template due to `{}`.",
                picture.short_path, err
            ),
        }
    }

    Ok(())
}

fn short_path_path(source_path: &str, path: &Path) -> String {
    path.to_str()
        .unwrap()
        .chars()
        .skip(source_path.len() + 1)
        .collect()
}
