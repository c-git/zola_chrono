use clap::{Parser, ValueEnum};
use log::LevelFilter;

#[derive(Parser, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default)]
#[command(
    author,
    version,
    about,
    long_about = "Updates the `date` and `updated` fields of the pages front matter"
)]
pub struct Cli {
    #[arg(
        value_name = "PATH",
        help = "The root folder to start at",
        long_help = "The root folder to start at (usually content or the folder). Required to be in a repository with a clean working tree.",
        default_value = "."
    )]
    pub root_path: String,

    #[arg(long, short)]
    pub unattended: bool,

    /// Set logging level to use
    #[arg(long, short, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
}

/// Exists to provide better help messages variants copied from LevelFilter as
/// that's the type that is actually needed
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Default)]
pub enum LogLevel {
    /// Nothing emitted in this mode
    #[default]
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}
