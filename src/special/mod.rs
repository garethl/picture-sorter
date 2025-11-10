use crate::options::{Options, SortMode};
use crate::picture::Picture;
use crate::special::motion::MotionPhoto;
use anyhow::anyhow;
use anyhow::Error;
use lazy_static::lazy_static;
use log::{debug, info};
use std::path::Path;

mod motion;

trait SpecialHandler: Sync {
    fn name(&self) -> &'static str;

    fn can_handle(
        &self,
        options: &Options,
        picture: &Picture,
        destination: &Path,
        destination_exists: bool,
        mode: &SortMode,
    ) -> bool;

    fn handle(
        &self,
        options: &Options,
        picture: &Picture,
        destination: &Path,
        destination_exists: bool,
        mode: &SortMode,
    ) -> Result<(), Error>;
}

lazy_static! {
    static ref SPECIAL_HANDLERS: Vec<Box<dyn SpecialHandler + 'static>> = {
        let mut m: Vec<Box<dyn SpecialHandler + 'static>> = Vec::new();
        m.push(Box::<MotionPhoto>::default());
        m
    };
}

pub fn execute_special_handlers(
    options: &Options,
    dry_run_prefix: &str,
    picture: &Picture,
    destination: &Path,
    destination_exists: bool,
    mode: &SortMode,
) -> Result<bool, Error> {
    if !options.motion_extract && !options.motion_strip {
        return Ok(false);
    }

    for handler in SPECIAL_HANDLERS.iter() {
        if handler.can_handle(options, picture, destination, destination_exists, mode) {
            if !options.dry_run {
                debug!(
                    "{}Special handler {} handling {}",
                    dry_run_prefix,
                    handler.name(),
                    picture.short_path
                );
                return handler
                    .handle(options, picture, destination, destination_exists, mode)
                    .map_err(|err| anyhow!("{}: {}", handler.name(), err))
                    .map(|_| true);
            } else {
                info!(
                    "{}Special handler {} handling {}",
                    dry_run_prefix,
                    handler.name(),
                    picture.short_path
                );
            }
            //
            return Ok(true);
        }
    }

    Ok(false)
}
