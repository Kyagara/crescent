use super::signal;
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
        let signal = signal::SignalArgs {
            name: self.name,
            signal: 9,
        };

        signal.run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_kill_run() -> Result<()> {
        let name = "kill_run".to_string();
        let command = KillArgs { name };

        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application does not exist.");
        Ok(())
    }
}
