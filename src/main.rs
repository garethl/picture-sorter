use crate::app_state::AppState;
use crate::cache::Cache;
use crate::expression::Expression;
use crate::options::Options;
use anyhow::Result;
use clap::Parser;
use std::process::exit;
use tokio::runtime::Runtime;

mod cache;
mod date_time_format;
mod exclusion;
mod exiftool;
mod exiftool_persistent; // future impl
//mod exiftool_executor; // future impl
mod app_state;
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

    let exif = exiftool_persistent::ExifToolManager::new(4);
    log::debug!("ExifTool pool initialized");

    let cache = Cache::new(&args.cache_file)?;
    log::debug!("Cache initialized");

    let expression = Expression::new(&args.format);
    log::debug!("Format expression parsed as {:?}", &expression);

    let state = AppState::new(exif, cache, expression, args);

    let rt = Runtime::new().unwrap();

    match rt.block_on(sorter::sort(state)) {
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
