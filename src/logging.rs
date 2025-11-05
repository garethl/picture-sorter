use log::{LevelFilter, debug};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

pub fn configure(quiet: bool, verbose: bool) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S.%3f)} {l} - {m}{n}",
        )))
        .build();

    let level = if verbose {
        LevelFilter::Debug
    } else if quiet {
        LevelFilter::Warn
    } else {
        LevelFilter::Info
    };
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    log4rs::init_config(config).unwrap();

    if level == LevelFilter::Debug {
        debug!("Debug logging enabled")
    }
}
