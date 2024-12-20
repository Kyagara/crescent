use std::process::Child;

use anyhow::Result;

/// Log system interface.
///
/// For now, only [`Journald`](crate::loggers::journald::Journald) is supported and few methods supported.
pub trait Logger {
    /// Log `n` lines from the service.
    fn log(&self, n: u64) -> Result<String>;

    /// Follow the log for any new lines.
    fn follow(&self) -> Result<Child>;
}
