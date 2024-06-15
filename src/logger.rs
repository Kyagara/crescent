use crate::loggers::journald::Journald;

use anyhow::Result;

pub struct Logger;

impl Logger {
    /// Get the log system.
    pub fn get_log_system() -> impl LogSystem {
        Journald::new()
    }
}

/// Log system interface.
///
/// For now, only journald is supported and few methods supported.
pub trait LogSystem {
    fn set_service_name(&mut self, name: &str);
    fn log(&self, lines: u64) -> Result<Vec<String>>;
    fn follow(&self) -> Result<()>;
}
