use anyhow::Result;
use deadpool::{
    async_trait,
    managed::{self, Pool, PoolError, RecycleResult},
};
use std::io::Write;
use std::process::{ChildStderr, ChildStdin, ChildStdout, Command, Stdio};

use thiserror::Error;

pub struct ExifToolPool {
    inner: Pool<ExifToolManager>,
}

impl ExifToolPool {
    async fn execute(&self) -> Result<(), ExifToolError> {
        let mut exif = self.inner.get().await?;
        exif.execute().await
    }
}

pub struct ExifTool {
    process: std::process::Child,
    stderr: ChildStderr,
    stdout: ChildStdout,
    stdin: ChildStdin,
}

impl ExifTool {
    fn launch() -> Result<ExifTool> {
        let mut child = match Command::new("exiftool")
            .arg("-stay_open")
            .arg("1")
            .arg("-@")
            .arg("-")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => {
                return Err(anyhow::anyhow!(err.to_string()));
            }
        };

        let stderr = child.stderr.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stdin = child.stdin.take().unwrap();

        Ok(ExifTool {
            process: child,
            stderr,
            stdout,
            stdin,
        })
    }
}

impl ExifTool {
    async fn execute(&mut self) -> Result<(), ExifToolError> {
        //write!(&mut self.stdin, "test\n")?;

        Ok(())
    }
}

pub struct ExifToolManager {
    //
}

pub fn new_pool() -> Result<ExifToolPool, ExifToolError> {
    match Pool::builder(ExifToolManager {}).build() {
        Ok(pool) => return Ok(ExifToolPool { inner: pool }),
        Err(err) => match err {
            managed::BuildError::Backend(err) => return Err(err),
            managed::BuildError::NoRuntimeSpecified(message) => {
                Err(ExifToolError::InternalError(message))
            }
        },
    }
}

#[async_trait]
impl managed::Manager for ExifToolManager {
    type Type = ExifTool;
    type Error = ExifToolError;

    /// Creates a new instance of [`Manager::Type`].
    async fn create(&self) -> Result<Self::Type, Self::Error> {
        ExifTool::launch().map_err(|err| ExifToolError::CannotCreate(format!("{:?}", err)))
    }

    /// Tries to recycle an instance of [`Manager::Type`].
    ///
    /// # Errors
    ///
    /// Returns [`Manager::Error`] if the instance couldn't be recycled.
    async fn recycle(&self, obj: &mut Self::Type) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ExifToolError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Cannot create exif-tool process: {0}")]
    CannotCreate(String),
}

impl From<PoolError<ExifToolError>> for ExifToolError {
    fn from(err: PoolError<ExifToolError>) -> Self {
        match err {
            PoolError::Timeout(t) => match t {
                managed::TimeoutType::Wait => todo!(),
                managed::TimeoutType::Create => {
                    ExifToolError::CannotCreate("Timeout waiting to create".to_string())
                }
                managed::TimeoutType::Recycle => {
                    ExifToolError::InternalError("Timeout recycling".to_string())
                }
            },
            PoolError::Backend(err) => err,
            PoolError::Closed => ExifToolError::InternalError("Pool closed".to_string()),
            PoolError::NoRuntimeSpecified => {
                ExifToolError::InternalError("No runtime specified".to_string())
            }
            PoolError::PostCreateHook(_) => {
                ExifToolError::InternalError("PostCreateHook - should never happen".to_string())
            }
            PoolError::PreRecycleHook(_) => {
                ExifToolError::InternalError("PreRecycleHook - should never happen".to_string())
            }
            PoolError::PostRecycleHook(_) => {
                ExifToolError::InternalError("PostRecycleHook - should never happen".to_string())
            }
        }
    }
}
