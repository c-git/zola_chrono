use std::{fs, io::Write, path::Path};

use anyhow::{bail, Context};
use chrono::Datelike;
use log::warn;
use once_cell::sync::Lazy;
use regex::Regex;
use toml_edit::Document;

static TOML_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^[[:space:]]*\+\+\+(\r?\n(?s).*?(?-s))\+\+\+[[:space:]]*(?:$|(?:\r?\n((?s).*(?-s))$))",
    )
    .unwrap()
});

static TODAY: Lazy<toml_edit::Item> = Lazy::new(|| {
    let now = chrono::Local::now();
    item_from_date(toml_edit::Date {
        year: now.year() as _,
        month: now.month() as _,
        day: now.day() as _,
    })
});

pub struct FileData<'a> {
    changed: bool,
    path: &'a Path,
    front_matter: String,
    content: String,
}

impl<'a> FileData<'a> {
    /// Write changes to disk.
    ///
    /// Precondition: Data is changed. If not changed function returns an error to avoid writing out the same data read in.
    pub fn write(&self) -> anyhow::Result<()> {
        if !self.is_changed() {
            bail!("No change detected. Write aborted. Path: {:?}", self.path);
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.path)?;
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
    pub fn update_front_matter(
        &mut self,
        last_edit_date: Option<toml_edit::Date>,
    ) -> anyhow::Result<()> {
        let key_date = "date";
        let key_updated = "updated";
        let toml = &self.front_matter[..];
        let mut doc = toml
            .parse::<Document>()
            .context("Failed to parse TOML in front matter")?;
        debug_assert_eq!(doc.to_string(), toml);
        let mut date = doc.get(key_date);
        let mut updated = doc.get(key_updated);

        // Record original values to compare if they changed at the end. Uses copy because it's just of a reference. Guaranteed by move semantics.
        let org_date = date;
        let org_updated = updated;

        // Check for wrong type
        if let Some(d) = date {
            if !d.is_datetime() {
                warn!("Non date value found for `date` in {:?}", self.path);
                date = None; // Only allow dates
            }
        }
        if let Some(u) = updated {
            if !u.is_datetime() {
                warn!("Non date value found for `updated` in {:?}", self.path);
                updated = None; // Only allow dates
            }
        }

        // Ensure if updated exists it is greater than or equal to date otherwise discard value
        if let Some(updated_date) = updated {
            if let Some(date) = date {
                if is_less_than_date(updated_date, date) {
                    warn!("`updated` is before `date` but this should never happen. `updated` being ignored in {:?}", self.path);
                    updated = None;
                }
            }
        }

        // Clear date if it is in the future
        if let Some(curr_date) = date {
            if is_less_than_date(&TODAY, curr_date) {
                warn!(
                    "date is set in the future. Date is being ignored in {:?}",
                    self.path
                );
                date = None;
            }
        }
        if let Some(curr_updated) = updated {
            if is_less_than_date(&TODAY, curr_updated) {
                warn!(
                    "updated is set in the future. updated is being ignored in {:?}",
                    self.path
                );
                date = None;
            }
        }

        // Set new date values base on the rules.
        // Prefer reusing the existing values as only the date is set in the generated value and not time nor offset
        let (new_date, new_updated) = match (last_edit_date, date, updated) {
            (None, None, _) => (TODAY.clone(), None),
            (None, Some(curr_date), _) => {
                if is_less_than_date(curr_date, &TODAY) {
                    (curr_date.clone(), Some(TODAY.clone()))
                } else if is_equal_date(curr_date, &TODAY) {
                    (curr_date.clone(), None)
                } else {
                    unreachable!("Future dates should have been cleared before starting, so must be less than or equal.")
                }
            }
            (Some(last), None, _) => {
                let last = item_from_date(last);
                if is_equal_date(&last, &TODAY) {
                    (last, None)
                } else {
                    (last, Some(TODAY.clone()))
                }
            }
            (Some(last), Some(date), None) => {
                let last = item_from_date(last);
                if is_equal_date(&last, date) {
                    (date.clone(), None)
                } else if is_less_than_date(date, &TODAY) {
                    (date.clone(), Some(TODAY.clone()))
                } else if is_equal_date(date, &TODAY) {
                    (date.clone(), None)
                } else {
                    debug_assert!(is_less_than_date(&TODAY, date));
                    (last, Some(TODAY.clone()))
                }
            }
            (Some(last), Some(date), Some(updated)) => {
                debug_assert!(
                    !is_less_than_date(updated, date),
                    "THIS SHOULDN'T HAPPEN WAS SUPPOSED TO HAVE BEEN CHECKED"
                );
                let last = item_from_date(last);
                if is_less_than_date(date, &last) || is_equal_date(date, &last) {
                    if is_less_than_date(updated, &TODAY) || is_equal_date(updated, &TODAY) {
                        (date.clone(), Some(updated.clone()))
                    } else {
                        (date.clone(), Some(TODAY.clone()))
                    }
                } else if is_less_than_date(date, &TODAY) || is_equal_date(date, &TODAY) {
                    if is_equal_date(updated, &TODAY) {
                        (date.clone(), Some(updated.clone()))
                    } else {
                        (date.clone(), Some(TODAY.clone()))
                    }
                } else {
                    debug_assert!(is_less_than_date(&TODAY, date));

                    if is_equal_date(updated, &TODAY) {
                        (TODAY.clone(), Some(updated.clone()))
                    } else {
                        (TODAY.clone(), None)
                    }
                }
            }
        };

        // Check if we've changed the starting values
        // NB: - date must change if it was None
        //     - This approach is slower due to loss of short circuit evaluation but I can read it, before it was...
        let is_date_same = org_date.is_some() && is_equal_date(org_date.unwrap(), &new_date);
        let did_update_start_and_end_none =
            org_updated.is_none() && org_updated.is_none() == new_updated.is_none();
        let did_updated_start_some_and_end_same_value = org_updated.is_some()
            && new_updated.is_some()
            && is_equal_date(org_updated.unwrap(), new_updated.as_ref().unwrap());
        let is_update_same =
            did_update_start_and_end_none || did_updated_start_some_and_end_same_value;
        let is_new_same_as_org = is_date_same && is_update_same;
        if !is_new_same_as_org {
            self.changed = true;
            match doc.entry(key_date) {
                toml_edit::Entry::Occupied(mut entry) => *entry.get_mut() = new_date,
                toml_edit::Entry::Vacant(entry) => {
                    entry.insert(new_date);
                }
            }
            if let Some(nu) = new_updated {
                match doc.entry(key_updated) {
                    toml_edit::Entry::Occupied(mut entry) => *entry.get_mut() = nu,
                    toml_edit::Entry::Vacant(entry) => {
                        entry.insert(nu);
                    }
                }
            } else {
                doc.remove(key_updated);
            }
            self.front_matter = doc.to_string();
        }

        Ok(())
    }

    fn new(path: &'a Path, front_matter: String, content: String) -> Self {
        Self {
            changed: false,
            path,
            front_matter,
            content,
        }
    }

    pub(crate) fn is_changed(&self) -> bool {
        self.changed
    }
}

/// Checks if both a and b are dates and if a < b
fn is_less_than_date(a: &toml_edit::Item, b: &toml_edit::Item) -> bool {
    match (a, b) {
        (toml_edit::Item::Value(a), toml_edit::Item::Value(b)) => match (a, b) {
            (toml_edit::Value::Datetime(a), toml_edit::Value::Datetime(b)) => {
                match (a.value().date, b.value().date) {
                    (Some(a), Some(b)) => match a.year.cmp(&b.year) {
                        std::cmp::Ordering::Less => true,
                        std::cmp::Ordering::Equal => match a.month.cmp(&b.month) {
                            std::cmp::Ordering::Less => true,
                            std::cmp::Ordering::Equal => a.day < b.day,
                            std::cmp::Ordering::Greater => false,
                        },
                        std::cmp::Ordering::Greater => false,
                    },
                    _ => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

fn item_from_date(d: toml_edit::Date) -> toml_edit::Item {
    toml_edit::Item::Value(toml_edit::Value::Datetime(toml_edit::Formatted::new(
        toml_edit::Datetime {
            date: Some(d),
            time: None,
            offset: None,
        },
    )))
}

// Check if both a and b are dates and a == b
fn is_equal_date(a: &toml_edit::Item, b: &toml_edit::Item) -> bool {
    match (a, b) {
        (toml_edit::Item::Value(a), toml_edit::Item::Value(b)) => match (a, b) {
            (toml_edit::Value::Datetime(a), toml_edit::Value::Datetime(b)) => {
                match (a.value().date, b.value().date) {
                    (Some(a), Some(b)) => a.year == b.year && a.month == b.month && a.day == b.day,
                    _ => false,
                }
            }
            _ => false,
        },
        _ => false,
    }
}

/// Build a FileData from a path
///
/// Splits the file data into front matter and content
/// Patterned on zola code https://github.com/c-git/zola/blob/3a73c9c5449f2deda0d287f9359927b0440a77af/components/content/src/front_matter/split.rs#L46
pub fn extract_file_data(path: &Path) -> anyhow::Result<FileData> {
    // TODO: Change to a constructor
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

    Ok(FileData::new(path, front_matter, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_equal_date() {
        assert!(is_equal_date(&TODAY, &TODAY));
    }

    #[test]
    fn test_is_less_than() {
        let past = item_from_date(toml_edit::Date {
            year: 1900,
            month: 1,
            day: 1,
        });
        assert!(is_less_than_date(&past, &TODAY));
        assert!(!is_less_than_date(&TODAY, &past));
        assert!(!is_less_than_date(&TODAY, &TODAY));
    }
}
