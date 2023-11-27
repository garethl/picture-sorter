// some code from https://github.com/alexipeck/exif, but this is using the json output, so
//  has more reliable parsing.

use anyhow::{anyhow, Context, Error, Result};
use log::warn;
use serde_json::{Map, Value};
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

        if !output.status.success() {
            if let Err(err) = try_extract_exiftool_error(output.stdout, output.stderr) {
                return Err(err);
            }

            return Err(anyhow!("Error extracting exif data: unknown error",));
        }

        let output = match String::from_utf8(output.stdout) {
            Ok(output) => output,
            Err(err) => {
                return Err(anyhow!(
                    "Error extracting exif data: invalid utf-8 string: {}",
                    err
                ));
            }
        };

        let json: Value = serde_json::from_str(&output).map_err(|err| {
            anyhow!(
                "Error extracting exif data: json serialization error: {}",
                err
            )
        })?;
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

fn try_extract_exiftool_error(std_out: Vec<u8>, std_err: Vec<u8>) -> Result<(), Error> {
    if std_err.len() > 0 {
        return Err(anyhow!(
            "Error extracting exif data: exiftool output: {}",
            String::from_utf8_lossy(&std_err)
        ));
    } else {
        // we have some json to parse
        let output = match String::from_utf8(std_out) {
            Ok(output) => output,
            Err(err) => {
                return Err(anyhow!(
                    "Error extracting exif data: invalid utf-8 string: {}",
                    err
                ));
            }
        };
        let json: Value = serde_json::from_str(&output)
            .map_err(|err| anyhow!("Error extracting exif data: unknown error: {}", err))?;

        match json {
            Value::Array(value) => {
                if value.len() > 0 {
                    if value[0].is_object() {
                        let value = value[0].as_object().unwrap();

                        if let Some(value) = value.get("Error") {
                            if value.is_string() {
                                return Err(anyhow!(
                                    "Error extracting exif data: {}",
                                    value.as_str().unwrap()
                                ));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
