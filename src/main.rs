use std::{
    io::{self, Write},
    path::PathBuf,
};

use anyhow::Context;
use clap::Parser;
use cli::Cli;
use log::{debug, LevelFilter};
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use version_control_clean_check::{check_version_control, CheckOptions};
mod cli;

fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();
    init_logging(cli.log_level.into())?;
    debug!("Cli: {cli:#?}");
    run(&cli)
}

fn init_logging(log_level: LevelFilter) -> anyhow::Result<()> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({d} {l} {t} - {m})}{n}")))
        .build();
    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(log_level))?;
    log4rs::init_config(config)?;
    Ok(())
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    // This also checks that the path exists as that is required for canonicalization
    let root_path = PathBuf::from(&cli.root_path)
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: '{}'", cli.root_path))?;

    // Confirm it is safe to make changes
    check_version_control(&root_path, &CheckOptions::new()).with_context(|| {
        format!("Failed to find a clean version control system before changes at {root_path:?}")
    })?;

    // Confirm user wants to make changes
    if !confirm_proceed(&root_path) {
        println!("Aborted at users request");
        return Ok(());
    }

    // Walk tree and process files

    // Process each file

    todo!("End of Run")
}

fn confirm_proceed(root_path: &PathBuf) -> bool {
    print!("Are you sure you want to update dates at {root_path:?}? (enter 'yes' to proceed) ");
    io::stdout().flush().expect("Failed to flush to stdout");

    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    user_input.trim() == "yes"
}
