use std::sync::Arc;

use crate::{
    cache::Cache, exiftool_persistent::ExifToolManager, expression::Expression, options::Options,
};

#[derive(Clone)]
pub struct AppState {
    pub exif: ExifToolManager,
    pub cache: Cache,
    pub expression: Arc<Expression>,
    pub options: Arc<Options>,
}

impl AppState {
    pub(crate) fn new(
        exif: ExifToolManager,
        cache: Cache,
        expression: Expression,
        options: Options,
    ) -> Self {
        Self {
            exif,
            cache,
            expression: Arc::new(expression),
            options: Arc::new(options),
        }
    }
}
