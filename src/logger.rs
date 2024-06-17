use crate::loggers::journald::Journald;

use anyhow::Result;

/// Log system implementation.
pub struct Logger;

impl Logger {
    /// Get the log system.
    pub fn get_log_system() -> impl LogSystem {
        Journald::new()
    }
}

/// Log system interface.
///
/// For now, only [`Journald`] is supported and few methods supported.
pub trait LogSystem {
    /// Log `n` lines from the service.
    fn log(&self, n: u64) -> Result<Vec<String>>;
    /// Follow the log for any new lines.
    fn follow(&self) -> Result<()>;
    /// Set the service name being queried.
    fn set_service_name(&mut self, name: &str);
}
