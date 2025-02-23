use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use log::warn;
use rand::distr::{Alphanumeric, SampleString};

pub struct TempFileTracker {
    paths: Vec<PathBuf>,
}

impl TempFileTracker {
    pub fn new() -> Self {
        Self { paths: vec![] }
    }

    pub fn with_prefix_in(&mut self, prefix: &OsStr, dir: &Path) -> PathBuf {
        let mut name = OsString::new();
        name.push(prefix);
        name.push("_");
        name.push(&generate_random_name(16));

        let path = dir.join(name);
        self.paths.push(path.clone());
        path
    }
}

fn generate_random_name(length: usize) -> String {
    Alphanumeric.sample_string(&mut rand::rng(), length) + ".temp"
}

impl Drop for TempFileTracker {
    fn drop(&mut self) {
        for path in self.paths.iter() {
            if path.exists() {
                match fs::remove_file(path) {
                    Ok(_) => {}
                    Err(err) => {
                        warn!("Error deleting temporary file: {}", err);
                    }
                }
            }
        }
    }
}
