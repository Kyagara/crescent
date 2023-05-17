use super::signal;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Send a SIGTERM signal to the application subprocess.")]
pub struct StopArgs {
    #[arg(help = "Application name.")]
    pub name: String,
}

impl StopArgs {
    pub fn run(self) -> Result<()> {
        let signal = signal::SignalArgs {
            name: self.name,
            signal: 15,
        };

        signal.run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_stop_run() -> Result<()> {
        let command = StopArgs {
            name: "stop_run".to_string(),
        };
        assert_eq!(command.name, "stop_run");
        let err = command.run().unwrap_err();
        assert_eq!(format!("{}", err), "Application does not exist.");
        Ok(())
    }
}
