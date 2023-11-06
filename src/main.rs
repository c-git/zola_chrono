use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Context;
use clap::Parser;
use cli::Cli;
use log::{debug, info, trace, LevelFilter};
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
    if !cli.unattended && !confirm_proceed(&root_path) {
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

fn walk_directory(root_path: &Path) -> anyhow::Result<()> {
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

fn process_file(path: &Path) -> anyhow::Result<()> {
    if !should_skip_file(path) {
        // TODO Pattern on zola code https://github.com/c-git/zola/blob/3a73c9c5449f2deda0d287f9359927b0440a77af/components/content/src/front_matter/split.rs#L46
        // TODO Parse toml with https://docs.rs/toml_edit/latest/toml_edit/visit_mut/index.html
        info!("{path:?} (processed)");
    } else {
        trace!("Skipped {path:?}");
    }
    Ok(())
}

fn should_skip_file(path: &Path) -> bool {
    !path.extension().is_some_and(|ext| ext == "md") || path.ends_with("_index.md")
}

fn confirm_proceed(root_path: &Path) -> bool {
    print!("Are you sure you want to update dates at {root_path:?}? (enter 'yes' to proceed) ");
    io::stdout().flush().expect("Failed to flush to stdout");

    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    user_input.trim() == "yes"
}
