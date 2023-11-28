use crate::exiftool::Exif;
use anyhow::anyhow;
use anyhow::Result;

use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExifMetadata {
    data: HashMap<String, String>,
}

impl ExifMetadata {
    pub fn get(&self, key: &str) -> Option<&String> {
        let key = key.to_lowercase();
        self.data.get(&key)
    }
}

pub fn get_metadata(path: &Path) -> Result<ExifMetadata> {
    let exif = Exif::new(path).map_err(|err| anyhow!("{}", err))?;

    let lowercase_map = exif
        .attributes
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    Ok(ExifMetadata {
        data: lowercase_map,
    })
}
