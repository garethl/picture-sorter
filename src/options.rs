use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    /// Source directory to scan
    #[clap(value_parser = valid_directory)]
    pub source: String,

    /// Destination directory to put files into
    #[clap(value_parser)]
    pub destination: String,

    /// Instead of copying, use hard links (requires source/destination to be on the same volume)
    #[clap(short, long, value_parser, default_value_t = false)]
    pub use_hard_links: bool,

    /// If a file exists at the destination, overwrite it instead of skipping it
    #[clap(short, long, value_parser, default_value_t = false)]
    pub overwrite: bool,

    /// Format string
    #[clap(short, long, value_parser)]
    pub format: String,

    /// Cache directory
    #[clap(short, long, value_parser)]
    pub cache_dir: String,

    /// Exclude files matching pattern (* is a wildcard)
    #[clap(short, long, value_parser)]
    pub exclude: Vec<String>,

    /// Quiet logging (errors only)
    #[clap(short, long, value_parser, default_value_t = false)]
    pub quiet: bool,

    /// Verbose logging
    #[clap(short, long, value_parser, default_value_t = false)]
    pub verbose: bool,

    /// Verbose logging
    #[clap(short, long, value_parser, default_value_t = false)]
    pub dry_run: bool,
}

fn valid_directory(value: &str) -> Result<String, String> {
    let path = Path::new(value);
    if !path.exists() || !path.is_dir() {
        Err(format!("Source directory `{}` does not exist", value))
    } else {
        Ok(value.to_string())
    }
}
