use crate::services::systemd::Systemd;

use anyhow::Result;

/// Init system implementation.
pub struct Service;

impl Service {
    /// Get the current `init` system.
    pub fn get_init_system() -> impl InitSystem {
        Systemd::new()
    }
}

/// Service status.
pub struct Status {
    pub script: String,
    pub stdin: String,
    pub pid: u32,
    pub active: String,
    pub cmd: String,
}

/// Init system interface.
///
/// For now, only [`Systemd`] is supported.
pub trait InitSystem {
    /// Create necessary file(s) for the service.
    ///
    /// - `systemd`: create service and socket units.
    fn create(&self, cmd: &str) -> Result<()>;
    /// Start the service.
    fn start(&self) -> Result<()>;
    /// Stop the service.
    ///
    /// - `systemd`: sends `stop` to the socket.
    fn stop(&self) -> Result<()>;
    /// Restart the service.
    fn restart(&self) -> Result<()>;
    /// Request the status of the service.
    fn status(&self, raw: bool) -> Result<Status>;
    /// List basic infomation of all services.
    ///
    /// Does not require [`InitSystem::set_service_name`].
    fn list(&self) -> Result<Vec<String>>;
    /// Checks if the service is running.
    fn is_running(&self) -> Result<bool>;
    /// Set the service name being queried. Not required if only using [`InitSystem::list`].
    fn set_service_name(&mut self, name: &str);
}
