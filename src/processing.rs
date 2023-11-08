use crate::stats::Stats;

use self::file_data::extract_file_data;
use anyhow::{bail, Context};
use log::{error, info, trace};
use std::{fs, path::Path, process::Command};
mod file_data;

pub fn walk_directory(root_path: &Path) -> anyhow::Result<Stats> {
    let mut result = Stats::new();
    if root_path.is_file() {
        match process_file(root_path)
            .with_context(|| format!("Processing failed for: {root_path:?}"))
        {
            Ok(stats) => result += stats,
            Err(e) => {
                error!("{e:?}");
                result.inc_errors();
            }
        }
    } else {
        for entry in fs::read_dir(root_path)
            .with_context(|| format!("Failed to read directory: {root_path:?}"))?
        {
            let entry =
                entry.with_context(|| format!("Failed to extract a DirEntry in {root_path:?}"))?;
            let path = entry.path();
            result += walk_directory(&path)?;
        }
    }

    Ok(result)
}

fn process_file(path: &Path) -> anyhow::Result<Stats> {
    let mut result = Stats::new();
    if !should_skip_file(path) {
        let mut data = extract_file_data(path)?;
        let last_edit_date =
            get_git_last_edit_date(path).context("Failed to get last edit date from git")?;
        data.update_front_matter(last_edit_date)
            .context("Failed to update front_matter")?;
        if data.is_changed() {
            data.write().context("Failed to write to file")?;
            result.inc_changed();
            trace!("(Changed)     {path:?}");
        } else {
            result.inc_not_changed();
            trace!("(Not Changed) {path:?}");
        };
    } else {
        result.inc_skipped();
        trace!("(Skipped)     {path:?}");
    }
    Ok(result)
}

fn get_git_last_edit_date(path: &Path) -> anyhow::Result<Option<toml_edit::Date>> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%cs", path.to_string_lossy().as_ref()])
        .output()
        .context("Failed to execute git command")?;
    if !output.status.success() || !output.stderr.is_empty() {
        bail!(
            "Running git failed. status: {} stdout: {}, stderr: {}",
            output.status,
            std::str::from_utf8(&output.stdout)?,
            std::str::from_utf8(&output.stderr)?
        );
    }
    let stdout = std::str::from_utf8(&output.stdout)?;
    info!("GitDate: {:?} - {path:?}", stdout.trim());

    if stdout.is_empty() {
        Ok(None)
    } else {
        let year: u16 = stdout[..=3].parse().context("Failed to parse year")?;
        let month: u8 = stdout[5..=6].parse().context("Failed to parse month")?;
        let day: u8 = stdout[8..=9].parse().context("Failed to parse day")?;
        Ok(Some(toml_edit::Date { year, month, day }))
    }
}

fn should_skip_file(path: &Path) -> bool {
    !path.extension().is_some_and(|ext| ext == "md") || path.ends_with("_index.md")
}
