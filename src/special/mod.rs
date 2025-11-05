use crate::app_state::AppState;
use crate::options::SortMode;
use crate::picture::Picture;
use crate::special::motion::MotionPhoto;
use anyhow::Error;
use anyhow::anyhow;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::{debug, info};
use std::path::Path;

mod motion;

#[async_trait]
trait SpecialHandler: Sync {
    fn name(&self) -> &'static str;

    fn can_handle(
        &self,
        state: AppState,
        picture: &Picture,
        destination: &Path,
        destination_exists: bool,
        overwrite: bool,
        mode: &SortMode,
    ) -> bool;

    async fn handle(
        &self,
        state: AppState,
        picture: &Picture,
        destination: &Path,
        destination_exists: bool,
        overwrite: bool,
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

pub async fn execute_special_handlers(
    state: AppState,
    dry_run: bool,
    dry_run_prefix: &str,
    picture: &Picture,
    destination: &Path,
    destination_exists: bool,
    overwrite: bool,
    mode: &SortMode,
) -> Result<bool, Error> {
    for handler in SPECIAL_HANDLERS.iter() {
        if handler.can_handle(
            state.clone(),
            picture,
            destination,
            destination_exists,
            overwrite,
            mode,
        ) {
            if !dry_run {
                debug!(
                    "{}Special handler {} handling {}",
                    dry_run_prefix,
                    handler.name(),
                    picture.short_path
                );
                return handler
                    .handle(
                        state,
                        picture,
                        destination,
                        destination_exists,
                        overwrite,
                        mode,
                    )
                    .await
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
