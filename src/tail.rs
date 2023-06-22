use anyhow::{anyhow, Result};
use crossbeam::channel::{unbounded, Sender};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

pub struct Tail {
    pub file: File,
    pub length: usize,
    pub path: PathBuf,
}

impl Tail {
    pub fn new(file_path: PathBuf) -> Result<Tail> {
        let log_file = File::open(&file_path)?;

        let metadata = log_file.metadata()?;

        Ok(Tail {
            file: log_file,
            length: metadata.len() as usize,
            path: file_path,
        })
    }

    pub fn read_lines(&mut self, lines: usize) -> Result<Vec<String>> {
        let mut bytes = lines * 200;

        if bytes > self.length {
            bytes = self.length;
        }

        self.file.seek(SeekFrom::End(-(bytes as i64)))?;

        let mut buffer = String::new();

        self.file.read_to_string(&mut buffer)?;

        let mut content: Vec<String> = buffer
            .lines()
            .rev()
            .take(lines)
            .map(|line| line.to_string())
            .collect();

        content.reverse();

        Ok(content)
    }

    pub fn watch(self, event_sender: &Sender<String>) -> Result<()> {
        let (sender, receiver) = unbounded();

        let mut watcher = RecommendedWatcher::new(sender, Config::default())?;

        watcher.watch(&self.path, RecursiveMode::NonRecursive)?;

        let mut content = String::new();
        let mut cursor = self.length as u64;

        for message in receiver {
            match message {
                Ok(event) => {
                    if event.kind.is_modify() {
                        let mut file = File::open(&self.path)?;
                        file.seek(SeekFrom::Start(cursor))?;

                        if file.metadata()?.len() > self.length as u64 {
                            cursor = file.metadata()?.len();

                            content.clear();
                            file.read_to_string(&mut content)?;
                            event_sender.send(content.clone())?;
                        }
                    }
                }
                Err(err) => return Err(anyhow!("{err}")),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env::temp_dir, fs::remove_file, io::Write};

    #[test]
    fn unit_tail_read_lines() -> Result<()> {
        let mut file_path = temp_dir();
        file_path.push("crescent_temp_log_file_test.txt");
        let mut log_file = File::create(&file_path)?;
        log_file.write_all(b"LOG")?;
        log_file.flush()?;

        let mut file = Tail::new(file_path.clone())?;

        let lines_read = file.read_lines(1)?;

        assert_eq!(lines_read.len(), 1);
        assert_eq!(lines_read.first().unwrap(), "LOG");

        remove_file(file_path)?;
        Ok(())
    }
}
