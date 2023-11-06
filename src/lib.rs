use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Context;
use cli::Cli;
use log::info;

use version_control_clean_check::{check_version_control, CheckOptions};

use crate::processing::walk_directory;
pub mod cli;
mod logging;
mod processing;

pub use logging::init_logging;

pub fn run(cli: &Cli) -> anyhow::Result<()> {
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

    // Change current working directory to target folder so that git commands will work correctly
    env::set_current_dir(&root_path).context("Failed change working directory to {root_path:?}")?;

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

fn confirm_proceed(root_path: &Path) -> bool {
    print!("Are you sure you want to update dates at {root_path:?}? (enter 'yes' to proceed) ");
    io::stdout().flush().expect("Failed to flush to stdout");

    let mut user_input = String::new();
    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    user_input.trim() == "yes"
}
