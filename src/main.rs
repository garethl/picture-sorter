use crate::cache::Cache;
use crate::expression::Expression;
use crate::options::Options;
use anyhow::Result;
use clap::Parser;
use std::process::exit;

mod cache;
mod date_time_format;
mod exclusion;
mod exiftool;
//mod exiftool_executor; // future impl
mod expression;
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
        log::error!("exiftool not available. Please ensure it is available in your path");
    }

    let cache = Cache::new(args.cache_file.clone())?;
    log::debug!("Cache initialized");

    let expression = Expression::new(&args.format);
    log::debug!("Format expression parsed as {:?}", &expression);

    match sorter::sort(cache, expression, &args) {
        Ok(_) => {
            log::debug!("Finished");
            Ok(())
        }
        Err(err) => {
            log::error!("Fatal error: {}", err);
            exit(1);
        }
    }
}
