use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};

use anyhow::Context;
use clap::Parser;
use cli::Cli;
use log::{debug, info, LevelFilter};
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
    let start = Instant::now();
    walk_directory(&root_path)?;
    info!(
        "Run duration: {} ms",
        Instant::now().duration_since(start).as_millis()
    );
    println!("Run Completed");
    Ok(())
}

fn walk_directory(root_path: &PathBuf) -> anyhow::Result<()> {
    if root_path.is_file() {
        process_file(root_path)?;
    } else {
        for entry in fs::read_dir(root_path)
            .with_context(|| format!("Failed to read directory: {root_path:?}"))?
        {
            let entry =
                entry.with_context(|| format!("Failed to extract a DirEntry in {root_path:?}"))?;
            let path = entry.path();
            walk_directory(&path)?;
        }
    }

    Ok(())
}

fn process_file(path: &PathBuf) -> anyhow::Result<()> {
    if !should_skip_file(path) {
        todo!()
    }
    Ok(())
}

fn should_skip_file(path: &PathBuf) -> bool {
    todo!()
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
