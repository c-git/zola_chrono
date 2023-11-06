use std::{fs, io::Write, path::Path};

use anyhow::{bail, Context};
use log::{error, info, trace};
use once_cell::sync::Lazy;
use regex::Regex;
use toml_edit::Document;

pub fn walk_directory(root_path: &Path) -> anyhow::Result<()> {
    if root_path.is_file() {
        if let Err(e) =
            process_file(root_path).with_context(|| format!("Processing failed for: {root_path:?}"))
        {
            error!("{e:?}");
        };
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
        let mut data = extract_file_data(path)?;
        let last_edit_date =
            get_git_last_edit_date(path).context("Failed to get last edit date from git")?;
        data.update_front_matter(last_edit_date)
            .context("Failed to update front_matter")?;
        data.write(path).context("Failed to write to file")?;
        info!("{path:?} (processed)");
    } else {
        trace!("Skipped {path:?}");
    }
    Ok(())
}

fn get_git_last_edit_date(path: &Path) -> anyhow::Result<toml_edit::Date> {
    todo!()
}

struct FileData {
    front_matter: String,
    content: String,
}
impl FileData {
    fn write(&self, path: &Path) -> anyhow::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        let mut s = "+++".to_string();
        s.push_str(&self.front_matter);
        s.push_str("+++\n");
        if !self.content.is_empty() {
            // Added a space between to match `dprint`
            s.push('\n');
        }
        s.push_str(&self.content);
        file.write_all(s.as_bytes())?;
        Ok(())
    }

    /// See cli::Cli command.long for explanation of rules (or readme)
    fn update_front_matter(&mut self, last_edit_date: toml_edit::Date) -> anyhow::Result<()> {
        let toml = &self.front_matter[..];
        let mut doc = toml
            .parse::<Document>()
            .context("Failed to parse TOML in front matter")?;
        debug_assert_eq!(doc.to_string(), toml);
        self.front_matter = doc.to_string();
        Ok(())
    }
}

static TOML_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^[[:space:]]*\+\+\+(\r?\n(?s).*?(?-s))\+\+\+[[:space:]]*(?:$|(?:\r?\n((?s).*(?-s))$))",
    )
    .unwrap()
});

/// Split the file data into front matter and content
fn extract_file_data(path: &Path) -> anyhow::Result<FileData> {
    // Patterned on zola code https://github.com/c-git/zola/blob/3a73c9c5449f2deda0d287f9359927b0440a77af/components/content/src/front_matter/split.rs#L46

    let content = fs::read_to_string(path).context("Failed to read file")?;

    // 2. extract the front matter and the content
    let caps = if let Some(caps) = TOML_RE.captures(&content) {
        caps
    } else {
        bail!("Failed to find front matter");
    };
    // caps[0] is the full match
    // caps[1] => front matter
    // caps[2] => content
    let front_matter = caps.get(1).unwrap().as_str().to_string();
    let content = caps.get(2).map_or("", |m| m.as_str()).to_string();

    Ok(FileData {
        front_matter,
        content,
    })
}

fn should_skip_file(path: &Path) -> bool {
    !path.extension().is_some_and(|ext| ext == "md") || path.ends_with("_index.md")
}
