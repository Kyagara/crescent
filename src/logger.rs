use std::process::Child;

use crate::loggers::journald::Journald;

use anyhow::Result;

/// Log system implementation.
pub struct Logger;

impl Logger {
    /// Get the log system.
    pub fn get(application_name: String) -> impl LogSystem {
        Journald::new(application_name)
    }
}

/// Log system interface.
///
/// For now, only [`Journald`] is supported and few methods supported.
pub trait LogSystem {
    /// Log `n` lines from the service.
    fn log(&self, n: u64) -> Result<String>;

    /// Follow the log for any new lines.
    fn follow(&self) -> Result<Child>;
}
