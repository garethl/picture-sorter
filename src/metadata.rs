use crate::exiftool_persistent::ExifToolManager;
use anyhow::Result;
use anyhow::anyhow;

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

pub async fn get_metadata(exif: ExifToolManager, path: &Path) -> Result<ExifMetadata> {
    let exif = exif
        .extract_exif_metadata(path)
        .await
        .map_err(|err| anyhow!("{}", err))?;

    //let exif = Exif::new(path).map_err(|err| anyhow!("{}", err))?;

    let lowercase_map = exif
        .attributes
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    Ok(ExifMetadata {
        data: lowercase_map,
    })
}
