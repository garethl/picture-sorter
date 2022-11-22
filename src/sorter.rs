use crate::exclusion::build_exclusion_filter;
use crate::picture::Picture;
use crate::{Cache, Expression};
use anyhow::{Context, Error};
use dpc_pariter::IteratorExt;
use log::{debug, error, info, warn};
use std::fs::{create_dir_all, metadata};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub fn sort(
    cache: Cache,
    expression: Expression,
    source: String,
    destination: String,
    exclusions: Vec<String>,
    use_hard_links: bool,
    overwrite: bool,
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
        process_picture(
            &expression,
            &destination,
            picture,
            use_hard_links,
            overwrite,
            dry_run,
        )?;
    }

    Ok(())
}

fn process_picture(
    expression: &Expression,
    destination: &str,
    picture: Picture,
    use_hard_links: bool,
    overwrite: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    debug!("Processing {}", picture.short_path);

    match expression.execute(&picture) {
        Ok(name) => {
            if dry_run {
                info!("[dry-run] copy {} to {}", picture.short_path, name);
                return Ok(());
            }

            let destination = Path::new(destination).join(name);

            debug!(
                "Going to copy {} to {}",
                picture.short_path,
                destination.display()
            );

            let path = picture.path;

            if let Some(destination_dir) = destination.parent() {
                debug!("Creating path {}", destination_dir.display());
                create_dir_all(destination_dir)?;
            }

            let destination_exists = destination.exists();
            let overwrite_required = if destination.exists() {
                are_files_different(&path, &destination)?
            } else {
                false
            };

            if overwrite_required && !overwrite {
                info!("Skipping {}. The destination ({}) already exists, is different, and overwrite flag not provided.", picture.short_path, destination.display());
                return Ok(());
            }

            if !overwrite_required && destination_exists && !overwrite {
                info!("Skipping {}. The destination ({}) already exists, is the same, and overwrite flag not provided.", picture.short_path, destination.display());
                return Ok(());
            }

            if use_hard_links {
                std::fs::hard_link(&path, &destination).with_context(|| {
                    format!(
                        "Error creating hard-link from {} to {}",
                        &path,
                        &destination.display()
                    )
                })?;
                info!(
                    "hard-linked {} to {}",
                    picture.short_path,
                    destination.display()
                )
            } else {
                std::fs::copy(&path, &destination).with_context(|| {
                    format!("Error copying {} to {}", &path, &destination.display())
                })?;
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

fn are_files_different(path: &str, destination: &std::path::PathBuf) -> anyhow::Result<bool> {
    let source_metadata = metadata(&path)?;
    let destination_metadata = metadata(&destination)?;

    let different = source_metadata.len() != destination_metadata.len()
        || !source_metadata
            .modified()?
            .eq(&destination_metadata.modified()?);

    Ok(different)
}

fn short_path_path(source_path: &str, path: &Path) -> String {
    path.to_str()
        .unwrap()
        .chars()
        .skip(source_path.len() + 1)
        .collect()
}
