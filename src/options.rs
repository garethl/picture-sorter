use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::{fmt::Display, path::Path};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    /// Source directory to scan
    #[clap(value_parser = valid_directory)]
    pub source: String,

    /// Destination directory to put files into
    #[clap(value_parser)]
    pub destination: String,

    /// Mode used to sort the files into the destination directory
    #[clap(short, long, default_value = "copy")]
    pub mode: SortMode,

    /// If a file exists at the destination, overwrite it instead of skipping it
    #[clap(short, long, value_parser, default_value_t = false)]
    pub overwrite: bool,

    /// Format string
    #[clap(short, long, value_parser)]
    pub format: String,

    /// Cache file
    #[clap(short, long, value_parser)]
    pub cache_file: String,

    /// Exclude files matching pattern (* is a wildcard)
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub exclude: Vec<String>,

    /// Quiet logging (errors only)
    #[clap(short, long, value_parser, default_value_t = false)]
    pub quiet: bool,

    /// Verbose logging
    #[clap(short, long, value_parser, default_value_t = false)]
    pub verbose: bool,

    /// Dry-run mode
    #[clap(short, long, value_parser, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq, Serialize)]
pub enum SortMode {
    /// Copy files to the destination
    #[default]
    Copy,
    /// Move files to the destination
    Move,
    /// Hard-link the files in the destination with the ones in the source.
    /// (requires source/destination to be on the same volume)
    HardLink,
}

impl Display for SortMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortMode::Copy => write!(f, "copy"),
            SortMode::Move => write!(f, "move"),
            SortMode::HardLink => write!(f, "hard-link"),
        }
    }
}

fn valid_directory(value: &str) -> Result<String, String> {
    let path = Path::new(value);
    if !path.exists() || !path.is_dir() {
        Err(format!("Source directory `{}` does not exist", value))
    } else {
        Ok(value.to_string())
    }
}
