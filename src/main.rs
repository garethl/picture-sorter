use crate::expression::Expression;
use crate::options::Options;
use crate::{cache::Cache, exiftool_executor::new_pool};
use anyhow::Result;
use clap::Parser;
use log::{debug, error};
use std::process::exit;

mod cache;
mod exclusion;
mod exiftool;
mod exiftool_executor;
mod expression;
mod format;
mod kv_store;
mod logging;
mod metadata;
mod options;
mod picture;
mod sorter;
mod special;
mod temp;

fn main() -> Result<()> {
    let args: Options = Options::parse();
    logging::configure(args.quiet, args.verbose);

    log::debug!("Options: {:?}", &args);

    if !exiftool::exiftool_available() {
        error!("exiftool not available. Please ensure it is available in your path");
    }

    let _pool: exiftool_executor::ExifToolPool = new_pool()?;

    let cache = Cache::new(args.cache_dir)?;
    let expression = Expression::new(&args.format);

    log::info!("Cache initialized");

    match sorter::sort(
        cache,
        expression,
        args.source,
        args.destination,
        args.exclude,
        args.use_hard_links,
        args.overwrite,
        args.dry_run,
    ) {
        Ok(_) => {
            debug!("Finished");
            Ok(())
        }
        Err(err) => {
            error!("Fatal error: {}", err);
            exit(1);
        }
    }
}
