use super::signal::generic_send_signal_command;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Send a SIGKILL signal to the application subprocess.")]
pub struct KillArgs {
    #[arg(help = "Application name.")]
    pub name: String,
}

impl KillArgs {
    pub fn run(self) -> Result<()> {
        generic_send_signal_command(&self.name, &9)
    }
}
