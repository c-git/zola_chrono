use log::LevelFilter;

use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

/// Initializes logging for the application (Should be called at most once)
pub fn init_logging(log_level: LevelFilter) -> anyhow::Result<()> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({d(%Y-%m-%d %H:%M:%S)} {l} {t} - {m})}{n}",
        )))
        .build();
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(log_level))?;
    log4rs::init_config(config)?;
    Ok(())
}
