use chrono::Local;
use log::{Log, Metadata, Record};

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        eprintln!(
            "[{}] [crescent] {} - {}",
            Local::now().time().format("%H:%M:%S"),
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use log::{Level, LevelFilter, MetadataBuilder};

    static LOGGER: Logger = Logger;

    #[test]
    fn unit_logger() -> Result<()> {
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::Info);

        let metadata = MetadataBuilder::new()
            .target("crescent")
            .level(Level::Error)
            .build();

        assert!(LOGGER.enabled(&metadata));

        let record = Record::builder()
            .args(format_args!(""))
            .level(Level::Error)
            .target("crescent")
            .build();

        LOGGER.log(&record);

        LOGGER.flush();

        Ok(())
    }
}
