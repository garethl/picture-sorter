use crate::cache::Cache;
use crate::expression::Expression;
use crate::options::Options;
use anyhow::Error;
use clap::Parser;
use log::{debug, error};
use std::process::exit;

mod cache;
mod exclusion;
mod exiftool;
mod expression;
mod format;
mod logging;
mod metadata;
mod options;
mod picture;
mod sorter;

fn main() {
    let args: Options = Options::parse();
    logging::configure(args.quiet, args.verbose);

    if !exiftool::exiftool_available() {
        error!("exiftool not available. Please ensure it is available in your path");
    }

    let cache = Cache::new(args.cache_dir);
    let expression = Expression::new(&args.format);

    match sorter::sort(
        cache,
        expression,
        args.source,
        args.destination,
        args.exclude,
    ) {
        Ok(_) => {
            debug!("Finished")
        }
        Err(err) => {
            error!("Fatal error: {}", err);
            exit(1);
        }
    }
}
