use super::signal::generic_send_signal_command;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Sends a SIGKILL signal to an application.")]
pub struct KillArgs {
    #[arg(help = "The application name.")]
    pub name: String,
}

impl KillArgs {
    pub fn run(self) -> Result<()> {
        generic_send_signal_command(&self.name, &9)
    }
}
