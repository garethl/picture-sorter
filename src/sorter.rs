use crate::exclusion::build_exclusion_filter;
use crate::picture::Picture;
use crate::{Cache, Expression};
use anyhow::Error;
use dpc_pariter::IteratorExt;
use log::{debug, error, info, warn};
use std::fs::create_dir_all;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub fn sort(
    cache: Cache,
    expression: Expression,
    source: String,
    destination: String,
    exclusions: Vec<String>,
    dry_run: bool,
) -> Result<(), Error> {
    let exclusion_filter = build_exclusion_filter(exclusions);
    debug!("Reading from {}", source);

    let source = Path::new(&source).canonicalize()?;
    let source_path = source.to_str().unwrap().to_string();
    let source_path2 = source.to_str().unwrap().to_string();

    let pictures = WalkDir::new(source)
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
        .parallel_map(move |entry: DirEntry| {
            Picture::from_dir_entry(&source_path2, entry, cache.clone())
        })
        .parallel_map(move |result| {
            let picture = match result {
                Ok(file) => Some(file),
                Err(err) => {
                    error!(
                        "Error reading metadata for `{}`. Ignoring. {}",
                        err.short_path, err.error
                    );
                    None
                }
            };

            if let Some(picture) = picture {
                if let Some(error) = picture.get("Error") {
                    warn!(
                        "Ignoring `{}`, cannot extract exif data because: `{}`",
                        picture.short_path, error
                    );
                    None
                } else {
                    Some(picture)
                }
            } else {
                None
            }
        })
        .flatten();

    for picture in pictures {
        process_picture(&expression, &destination, dry_run, picture)?;
    }

    Ok(())
}

fn process_picture(
    expression: &Expression,
    destination: &str,
    dry_run: bool,
    picture: Picture,
) -> anyhow::Result<()> {
    debug!("Processing {}", picture.short_path);

    match expression.execute(&picture) {
        Ok(name) => {
            if dry_run {
                info!("[dry-run] copy {} to {}", picture.short_path, name)
            } else {
                let destination = Path::new(destination).join(name);

                debug!(
                    "Going to copy {} to {}",
                    picture.short_path,
                    destination.display()
                );

                if !destination.exists() {
                    if let Some(destination_dir) = destination.parent() {
                        debug!("Creating path {}", destination_dir.display());
                        create_dir_all(destination_dir)?;
                    }

                    //TODO: Copy
                } else {
                    //TODO: wat
                }

                info!("copied {} to {}", picture.short_path, destination.display())
            }
        }
        Err(err) => warn!(
            "Skipping {}, unable to apply name template due to `{}`.",
            picture.short_path, err
        ),
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
