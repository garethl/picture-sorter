use crate::exclusion::build_exclusion_filter;
use crate::options::{Options, SortMode};
use crate::picture::Picture;
use crate::special::execute_special_handlers;
use crate::{Cache, Expression};
use anyhow::{Context, Error};
use dpc_pariter::IteratorExt;
use log::{debug, info, warn};
use std::fs::{create_dir_all, metadata};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn sort(
    cache: Cache,
    expression: Expression,
    options: &Options,
) -> Result<(), Error> {
    let exclusion_filter = build_exclusion_filter(options.exclude.clone());
    debug!("Reading from {}", options.source);

    let source = Path::new(&options.source).canonicalize()?;
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
                        short_path_path(&source_path, path),
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
                    warn!(
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
        process_picture(&expression, picture, options)?;
    }

    Ok(())
}

fn process_picture(
    expression: &Expression,
    picture: Picture,
    options: &Options,
) -> anyhow::Result<()> {
    debug!("Processing {}", &picture.short_path);

    match expression.execute(&picture) {
        Ok(name) => {
            let dry_run_prefix = if options.dry_run { "[dry-run] " } else { "" };
            let destination = Path::new(&options.destination).join(name);

            debug!(
                "{}Going to {} {} to {}",
                dry_run_prefix,
                options.mode,
                picture.short_path,
                destination.display()
            );

            let path = &picture.path;

            if let Some(destination_dir) = destination.parent() {
                debug!(
                    "{}Creating path {}",
                    dry_run_prefix,
                    destination_dir.display()
                );
                if !options.dry_run {
                    create_dir_all(destination_dir)?;
                }
            }

            let destination_exists = destination.exists();

            let special_handler_outcome = execute_special_handlers(
                options,
                dry_run_prefix,
                &picture,
                &destination,
                destination_exists,
                &options.mode,
            );
            match special_handler_outcome {
                Ok(processed) => {
                    if processed {
                        return Ok(());
                    }
                }
                Err(err) => {
                    warn!(
                        "{}Error processing {}. Special handler errored: {}",
                        dry_run_prefix, picture.short_path, err
                    );
                    return Ok(());
                }
            }

            let overwrite_required = if destination.exists() {
                are_files_different(path, &destination)?
            } else {
                false
            };

            if overwrite_required && !options.overwrite {
                info!("{}Skipping {}. The destination ({}) already exists, is different, and overwrite flag not provided.",
                    dry_run_prefix,
                    picture.short_path,
                    destination.display()
                );
                return Ok(());
            }

            if !overwrite_required && destination_exists && !options.overwrite {
                debug!("{}Skipping {}. The destination ({}) already exists, is the same, and overwrite flag not provided.",
                    dry_run_prefix,
                    picture.short_path,
                    destination.display()
                );
                return Ok(());
            }

            sort_single_picture_file(&picture, &options.mode, options.dry_run, dry_run_prefix, &destination, path)?;
        }
        Err(err) => warn!(
            "Skipping {}, unable to apply name template due to `{}`.",
            picture.short_path, err
        ),
    }

    Ok(())
}

pub fn sort_single_picture_file(
    picture: &Picture,
    mode: &SortMode,
    dry_run: bool,
    dry_run_prefix: &str,
    destination: &PathBuf,
    path: &String,
) -> Result<(), Error> {
    Ok(match mode {
        SortMode::Copy => {
            if !dry_run {
                std::fs::copy(path, destination).with_context(|| {
                    format!("Error copying {} to {}", &path, destination.display())
                })?;
            }
            info!(
                "{}copied {} to {}",
                dry_run_prefix,
                picture.short_path,
                destination.display()
            )
        }
        SortMode::Move => {
            if !dry_run {
                std::fs::copy(path, destination).with_context(|| {
                    format!("Error copying {} to {}", &path, destination.display())
                })?;
                std::fs::remove_file(path).with_context(|| {
                    format!(
                        "Error removing file at {} (already copied to {})",
                        &path,
                        &destination.display()
                    )
                })?;
            }
            info!(
                "{}moved {} to {}",
                dry_run_prefix,
                picture.short_path,
                destination.display()
            )
        }
        SortMode::HardLink => {
            if !dry_run {
                std::fs::hard_link(path, destination).with_context(|| {
                    format!(
                        "Error creating hard-link from {} to {}",
                        &path,
                        destination.display()
                    )
                })?;
            }
            info!(
                "{}hard-linked {} to {}",
                dry_run_prefix,
                picture.short_path,
                destination.display()
            )
        }
    })
}

fn are_files_different(path: &str, destination: &PathBuf) -> anyhow::Result<bool> {
    let source_metadata = metadata(path)?;
    let destination_metadata = metadata(destination)?;

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
