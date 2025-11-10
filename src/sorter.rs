use crate::app_state::AppState;
use crate::exclusion::build_exclusion_filter;
use crate::options::SortMode;
use crate::picture::Picture;
use crate::special::execute_special_handlers;
use anyhow::{Context, Error};
use futures::stream::{self, StreamExt};
use log::{debug, error, info, warn};
use std::fs::{create_dir_all, metadata};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub async fn sort(state: AppState) -> Result<(), Error> {
    let options = state.options.clone();
    let exclusion_filter = build_exclusion_filter(&options.exclude);
    debug!("Reading from {}", options.source);

    let source = Path::new(&options.source).canonicalize()?;
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
                        short_path_path(&source_path, path),
                        err
                    ),
                };
                None
            }
        })
        .filter(|e| e.file_type().is_file());

    let pictures = stream::iter(files).map(async |entry| {
        let result = Picture::from_dir_entry(&source_path2, entry, state.clone()).await;
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
    });

    pictures
        .filter_map(|result| async {
            result.await.map(|picture| {
                process_picture(
                    state.clone(),
                    &options.destination,
                    picture,
                    &options.mode,
                    options.overwrite,
                    options.dry_run,
                )
            })
        })
        .for_each_concurrent(5, |result| async {
            match result.await {
                Ok(_) => {}
                Err(err) => {
                    error!("Unexpected error processing picture: {}", err);
                }
            }
        })
        .await;

    Ok(())
}

async fn process_picture(
    state: AppState,
    destination: &str,
    picture: Picture,
    mode: &SortMode,
    overwrite: bool,
    dry_run: bool,
) -> anyhow::Result<()> {
    debug!("Processing {}", &picture.short_path);

    match state.expression.execute(&picture) {
        Ok(name) => {
            let dry_run_prefix = if dry_run { "[dry-run] " } else { "" };
            let destination = Path::new(destination).join(name);

            debug!(
                "{}Going to {} {} to {}",
                dry_run_prefix,
                mode,
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
                if !dry_run {
                    create_dir_all(destination_dir)?;
                }
            }

            let destination_exists = destination.exists();

            let special_handler_outcome = execute_special_handlers(
                state,
                dry_run,
                dry_run_prefix,
                &picture,
                &destination,
                destination_exists,
                overwrite,
                mode,
            )
            .await;
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

            if overwrite_required && !overwrite {
                info!(
                    "{}Skipping {}. The destination ({}) already exists, is different, and overwrite flag not provided.",
                    dry_run_prefix,
                    picture.short_path,
                    destination.display()
                );
                return Ok(());
            }

            if !overwrite_required && destination_exists && !overwrite {
                debug!(
                    "{}Skipping {}. The destination ({}) already exists, is the same, and overwrite flag not provided.",
                    dry_run_prefix,
                    picture.short_path,
                    destination.display()
                );
                return Ok(());
            }

            sort_single_picture_file(&picture, mode, dry_run, dry_run_prefix, &destination, path)?;
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
