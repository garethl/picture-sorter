use std::path::Path;

use ::exiftool::{ExifTool, ExifToolError};
use deadpool::managed::{Manager, Metrics, Object, Pool, RecycleResult};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::exiftool::Exif;

#[derive(Clone)]
pub struct ExifToolManager {
    inner: Pool<ExifToolLifeCycle>,
}

#[derive(Error, Debug)]
pub enum ExifToolExecuteError {
    #[error("Exif pool error: {0}")]
    PoolError(deadpool::managed::PoolError<ExifToolError>),
    #[error("Exif tool error: {0}")]
    ExifToolError(ExifToolError),
    #[error("Invalid JSON error: {0}")]
    InvalidJsonError(Value),
}

impl ExifToolManager {
    pub fn new(size: usize) -> Self {
        let manager = ExifToolLifeCycle {};
        let pool = Pool::builder(manager)
            .max_size(size)
            .build()
            .expect("Failed to create ExifTool pool");
        ExifToolManager { inner: pool }
    }

    pub async fn extract_exif_metadata(&self, path: &Path) -> Result<Exif, ExifToolExecuteError> {
        let mut tool = self.get().await?;

        let value = tool
            .json(path, &[])
            .map_err(|e| ExifToolExecuteError::ExifToolError(e))?;

        let map = match value.as_object() {
            Some(map) => map,
            None => {
                return Err(ExifToolExecuteError::InvalidJsonError(value));
            }
        };

        let map = map
            .into_iter()
            .map(|(k, v)| (k.to_string(), get_string(v)))
            .collect();

        Ok(Exif { attributes: map })
    }

    pub async fn execute_bytes(&self, args: Vec<&str>) -> Result<Vec<u8>, ExifToolExecuteError> {
        let mut tool = self.get().await?;

        let data = tool
            .execute_raw(&args)
            .map_err(|e| ExifToolExecuteError::ExifToolError(e))?;

        Ok(data)
    }

    async fn get(&self) -> Result<Object<ExifToolLifeCycle>, ExifToolExecuteError> {
        let exif_tool = self
            .inner
            .get()
            .await
            .map_err(|e| ExifToolExecuteError::PoolError(e))?;
        Ok(exif_tool)
    }
}

fn get_string(value: &Value) -> String {
    match value {
        Value::Null => "".to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        _ => value.to_string(),
    }
}

pub struct ExifToolLifeCycle {}

impl Manager for ExifToolLifeCycle {
    type Type = ExifTool;

    type Error = ExifToolError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        ExifTool::new()
    }

    async fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &Metrics,
    ) -> RecycleResult<Self::Error> {
        Ok(())
    }
}
