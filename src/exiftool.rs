// some code from https://github.com/alexipeck/exif, but this is using the json output, so
//  has more reliable parsing.

use anyhow::{anyhow, Context, Result};
use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    path::Path,
    process::{Command, Stdio},
};

pub fn exiftool_available() -> bool {
    return Command::new("exiftool")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok();
}

#[derive(Clone, Debug, Default)]
pub struct Exif {
    pub attributes: HashMap<String, String>,
}

impl Exif {
    pub fn new(file_path: &Path) -> Result<Self> {
        let child = match Command::new("exiftool")
            .arg("-j")
            .arg(file_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(err) => {
                return Err(anyhow!(err.to_string()));
            }
        };

        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(err) => {
                return Err(anyhow!(err.to_string()));
            }
        };

        let stderr_output = match String::from_utf8(output.stderr) {
            Ok(output) => output,
            Err(err) => {
                return Err(anyhow!(err.to_string()).context("Error extracting exif data"));
            }
        };

        if !output.status.success() {
            return Err(anyhow!(stderr_output).context("Error extracting exif data"));
        }

        let output = match String::from_utf8(output.stdout) {
            Ok(output) => output,
            Err(err) => {
                return Err(anyhow!(err.to_string()).context("Error extracting exif data"));
            }
        };

        let json: Value = serde_json::from_str(&output).context("Error extracting exif data")?;
        let obj = extract_map(&json);
        if obj.is_none() {
            return Err(anyhow!(
                "Error extracting exif data - invalid json `{}`",
                &output
            ));
        }

        let map = obj
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k.to_string(), get_string(v)))
            .collect();
        Ok(Exif { attributes: map })
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

fn extract_map(value: &Value) -> Option<&Map<String, Value>> {
    value.as_array()?.get(0)?.as_object()
}
