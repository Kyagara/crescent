use crate::services::systemd::Systemd;

use anyhow::Result;

/// Init system implementation.
pub struct Service;

impl Service {
    /// Get the current `init` system.
    pub fn get() -> impl InitSystem {
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

pub enum StatusOutput {
    Pretty(Status),
    Raw(String),
}

/// Init system interface.
///
/// For now, only [`Systemd`] is supported.
pub trait InitSystem {
    /// Set the service name being queried. Not required if only using [`InitSystem::list`].
    fn set_service_name(&mut self, name: &str);

    /// Returns the absolute paths of all generated scripts.
    ///
    /// - [`Systemd`]: "$HOME/.config/systemd/user/cres.<name>.service" and "$HOME/.config/systemd/user/cres.<name>.socket"
    fn get_scripts_paths(&self) -> Vec<String>;

    /// Reload the init system.
    ///
    /// - [`Systemd`]: runs `daemon-reload`.
    fn reload(&self) -> Result<()>;

    /// Checks if the service is running.
    fn is_running(&self) -> Result<bool>;

    /// Checks if the service is enabled for startup.
    fn is_enabled(&self) -> Result<bool>;

    /// Create necessary file(s) for the service.
    ///
    /// - [`Systemd`]: generates the service and socket units.
    fn create(&self, cmd: &str) -> Result<()>;

    /// Start the service.
    fn start(&self) -> Result<()>;

    /// Stop the service.
    ///
    /// - [`Systemd`]: sends `stop` to the socket.
    fn stop(&self) -> Result<()>;

    /// Send a signal to the service.
    fn kill(&self, signal: i32) -> Result<()>;

    /// Restart the service.
    fn restart(&self) -> Result<()>;

    /// Enable the service for startup.
    fn enable(&self) -> Result<()>;

    /// Disable the service for startup.
    fn disable(&self) -> Result<()>;

    /// Request the status of the service.
    fn status(&self, raw: bool) -> Result<StatusOutput>;

    /// List basic infomation of all services.
    ///
    /// Does not require [`InitSystem::set_service_name`].
    fn list(&self) -> Result<Vec<String>>;
}
