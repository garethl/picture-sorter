use crate::exiftool::Exif;
use anyhow::anyhow;
use anyhow::Result;
use log::debug;
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
    let path = adjust_canonicalization(path);

    debug!("Reading exif data from {}", path);

    let exif = Exif::new(Path::new(&path)).map_err(|err| anyhow!("{}", err))?;

    let lowercase_map = exif
        .attributes
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();

    Ok(ExifMetadata {
        data: lowercase_map,
    })
}

#[cfg(not(target_os = "windows"))]
fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(target_os = "windows")]
fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}
