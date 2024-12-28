use anyhow::Result;

/// Init system interface.
///
/// For now, only [`Systemd`][`crate::init_systems::systemd::Systemd`] is supported.
pub trait InitSystem {
    /// Updates the name of the service being queried.
    fn set_name(&mut self, name: &str);

    /// Returns the absolute paths of all generated scripts.
    ///
    /// - [`Systemd`][`crate::init_systems::systemd::Systemd`]:
    ///     - `/etc/systemd/system/cres.<name>.service` and `/etc/systemd/system/cres.<name>.socket`
    ///     - If using --user flag:
    ///     - `$HOME/.config/systemd/user/cres.<name>.service` and `$HOME/.config/systemd/user/cres.<name>.socket`
    fn get_scripts_paths(&self) -> Vec<String>;

    /// Reload the init system.
    ///
    /// - [`Systemd`][`crate::init_systems::systemd::Systemd`]: runs `daemon-reload`.
    fn reload(&self) -> Result<()>;

    /// Check if the service is currently running.
    fn is_running(&self) -> Result<bool>;

    /// Check if the service is enabled to start at boot.
    fn is_enabled(&self) -> Result<bool>;

    /// Create the necessary file(s) for a new service.
    ///
    /// - [`Systemd`][`crate::init_systems::systemd::Systemd`]: generates the service and socket units.
    fn create(&self, cmd: &str) -> Result<()>;

    /// Start the service.
    fn start(&self) -> Result<()>;

    /// Stop the service.
    ///
    /// - [`Systemd`][`crate::init_systems::systemd::Systemd`]: sends `stop` to the *socket*.
    fn stop(&self) -> Result<()>;

    /// Send a signal to the service.
    fn kill(&self, signal: i32) -> Result<()>;

    /// Restart the service.
    fn restart(&self) -> Result<()>;

    /// Enable a service to start at boot.
    fn enable(&self) -> Result<()>;

    /// Disable a service from starting at boot.
    fn disable(&self) -> Result<()>;

    /// Request the status of a service. Returns [`StatusOutput::Raw`][`crate::init_system::StatusOutput::Raw`] if the service is not running.
    fn status(&self, raw: bool) -> Result<StatusOutput>;

    /// List basic infomation of all services.
    fn list(&self) -> Result<Vec<String>>;
}

/// Types of status output.
pub enum StatusOutput {
    Pretty(Status),
    Raw(String),
}

/// Struct for the status command from multiple init systems. Multiple commands might be used to retrieve these values.
pub struct Status {
    pub script: String,
    pub stdin: String,
    pub pid: u32,
    pub active: String,
    pub cmd: String,
}
