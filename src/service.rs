use crate::services::systemd::Systemd;

use anyhow::Result;

pub struct Service;

impl Service {
    /// Get the current init system.
    pub fn get_init_system() -> impl InitSystem {
        Systemd::new()
    }
}

pub struct Status {
    pub script: String,
    pub stdin: String,
    pub pid: u32,
    pub active: String,
    pub cmd: String,
}

/// Init system interface.
///
/// For now, only systemd is supported.
pub trait InitSystem {
    fn start(&self, cmd: &str) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn restart(&self) -> Result<()>;
    fn status(&self, raw: bool) -> Result<Status>;
    fn list(&self) -> Result<Vec<String>>;
    fn is_running(&self) -> Result<bool>;
    fn set_service_name(&mut self, name: &str);
}
