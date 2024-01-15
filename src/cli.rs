//! Stores Command Line Interface (cli)  configuration
use clap::{Parser, ValueEnum};
use log::LevelFilter;

#[derive(Parser, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default)]
#[command(
    author,
    version,
    about,
    long_about = "Updates the `date` and `updated` fields of the pages front matter
    1. `date` should be the original publish date (Must exist and be today or earlier).
    2. `updated` should only be set if `date` is not equal to the last commit date, if it needs to be set it should match the last commit date
"
)]
/// Stores the configurations acquired via the command line
pub struct Cli {
    #[arg(value_name = "PATH", default_value = ".")]
    /// The root folder to start at
    ///
    /// Usually you want to point this to the content folder of the zola repo. It is required for it to be in a repository with a clean working tree.
    pub root_path: String,

    /// If set will not prompt for confirmation before running
    #[arg(long, short)]
    pub unattended: bool,

    /// If set will not modify any files and only report how many files would have been changed
    ///
    /// Return codes in this mode: (0) No files would have been changed (1) Error Occurred (2) Files would have been changed
    #[arg(long = "check", short = 'c')]
    pub should_check_only: bool,

    /// Allows changes to be made even if there are dirty files in the vcs. WARNING: This means that there will be no easy way to undo changes made
    #[arg(long)]
    pub allow_dirty: bool,

    /// Set logging level to use
    #[arg(long, short, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
}

/// Exists to provide better help messages variants copied from LevelFilter as
/// that's the type that is actually needed
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Default)]
#[allow(missing_docs)]
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
