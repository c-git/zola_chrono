use std::{fmt::Display, ops::AddAssign};

/// Tracks files changed (NB: Stops counting at 2^16 and saturates)
#[derive(Debug, Default)]
#[must_use]
pub struct Stats {
    changed: u16,
    not_changed: u16,
    skipped: u16,
    errors: u16,
}

impl Stats {
    pub(crate) fn new() -> Self {
        Self {
            changed: 0,
            not_changed: 0,
            skipped: 0,
            errors: 0,
        }
    }

    pub fn changed(&self) -> u16 {
        self.changed
    }

    pub fn not_changed(&self) -> u16 {
        self.not_changed
    }

    pub fn skipped(&self) -> u16 {
        self.skipped
    }
    pub fn errors(&self) -> u16 {
        self.errors
    }

    pub fn inc_changed(&mut self) {
        self.changed = self.changed.saturating_add(1);
    }

    pub fn inc_not_changed(&mut self) {
        self.not_changed = self.not_changed.saturating_add(1);
    }

    pub fn inc_skipped(&mut self) {
        self.skipped = self.skipped.saturating_add(1);
    }
    pub fn inc_errors(&mut self) {
        self.errors = self.errors.saturating_add(1);
    }
}

impl AddAssign for Stats {
    fn add_assign(&mut self, rhs: Self) {
        self.changed += rhs.changed;
        self.not_changed += rhs.not_changed;
        self.skipped += rhs.skipped;
        self.errors += rhs.errors;
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Change: {}, Not Changed: {}, Skipped: {}, Errors: {}",
            self.changed, self.not_changed, self.skipped, self.errors
        )
    }
}
