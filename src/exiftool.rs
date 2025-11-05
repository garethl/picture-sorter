use log::warn;

use std::{
    collections::HashMap,
    path::Path,
    process::{Command, Stdio},
};

pub fn exiftool_available() -> bool {
    match Command::new("exiftool")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            match child.wait() {
                Ok(_) => {}
                Err(err) => {
                    warn!("Error waiting on exiftool execution: {}", err)
                }
            }
            true
        }
        Err(err) => {
            warn!("Unable to execute exiftool: {}", err);
            false
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Exif {
    pub attributes: HashMap<String, String>,
}

#[cfg(not(target_os = "windows"))]
pub fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(target_os = "windows")]
pub fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    let p = if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    };

    p
}
