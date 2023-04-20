use anyhow::{anyhow, Context, Result};
use daemonize::Daemonize;
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::net::UnixListener,
    path::PathBuf,
    process::exit,
    str::FromStr,
    sync::mpsc::{Receiver, Sender},
    thread,
};
use subprocess::{Exec, Redirection};
use sysinfo::Pid;

use crate::directory::{self};

pub struct Application {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub file_path: String,
    pub app_dir: PathBuf,
    pub work_dir: PathBuf,
}

impl Application {
    pub fn new(
        file_path: String,
        app_name: Option<String>,
        command: Option<String>,
        arguments: Option<String>,
    ) -> Result<Application> {
        let file_path_buf = PathBuf::from(file_path.clone());
        let work_dir = file_path_buf.parent().unwrap().to_path_buf();

        let name = match app_name {
            Some(name) => name,
            None => file_path_buf
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        };

        let app_dir = directory::application_dir_by_name(name.clone())?;

        let mut command = match command {
            Some(command) => command,
            None => String::new(),
        };

        let file_path_str = file_path.clone();

        let mut args = vec![];

        match command.as_str() {
            "java" => {
                command = "java".to_string();
                args.push(String::from("-jar"));
                args.push(file_path_str)
            }
            "" => command = file_path_str,
            _ => args.push(file_path_str),
        }

        if let Some(arguments) = arguments {
            args.push(arguments);
        }

        Ok(Application {
            file_path,
            name,
            app_dir,
            work_dir,
            command,
            args,
        })
    }

    pub fn start(self) -> Result<()> {
        if self.app_dir.is_dir() {
            fs::remove_dir_all(&self.app_dir).context("Couldn't reset directory.")?;
        }

        fs::create_dir_all(&self.app_dir).context("Couldn't create app directory.")?;

        daemonize(
            self.app_dir.clone(),
            self.name.clone(),
            self.work_dir.clone(),
        )?;

        self.start_subprocess()?;

        Ok(())
    }

    fn start_subprocess(self) -> Result<()> {
        let mut process = Exec::cmd(&self.command)
            .args(&self.args)
            .stdout(Redirection::Merge)
            .stdin(Redirection::Pipe)
            .popen()?;

        let mut stdin = process.stdin.take().unwrap();

        thread::spawn(move || {
            process.wait().unwrap();
            exit(0)
        });

        let (sender, receiver): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();

        thread::spawn(move || {
            for received in receiver {
                stdin.write_all(received.as_bytes()).unwrap();
                stdin.flush().unwrap();
            }
        });

        let process_path = match directory::application_dir_by_name(self.name.clone()) {
            Ok(dir) => dir,
            Err(err) => {
                return Err(anyhow!("Error getting directory path: {err}"));
            }
        };

        let address = process_path.join(self.name + ".sock");

        let socket = UnixListener::bind(&address)?;

        loop {
            match socket.accept() {
                Ok((mut stream, _)) => {
                    let mut message = String::new();
                    stream.read_to_string(&mut message).unwrap();

                    // Sending command to stdin
                    sender.send(message).unwrap();
                }
                Err(e) => {
                    eprintln!("Socket error: {}", e)
                }
            }
        }
    }
}

pub fn process_pid_by_name(name: String) -> Result<Pid> {
    let application_path = directory::application_dir_by_name(name)?;

    let app_name = application_path
        .file_name()
        .context("Should contain the directory name as string")?
        .to_str()
        .context("Should be a valid string")?
        .to_string();

    let mut pid_path = application_path;
    pid_path.push(app_name + ".pid");

    let pid_file = fs::read_to_string(pid_path).context("Error reading PID file")?;

    let pid = Pid::from_str(pid_file.trim()).context("Error trimming PID file")?;

    Ok(pid)
}

fn daemonize(app_dir: PathBuf, app_name: String, work_dir: PathBuf) -> Result<()> {
    let log = File::create(app_dir.join(app_name.clone() + ".log"))?;

    println!("Starting daemon");

    let daemonize = Daemonize::new()
        .pid_file(app_dir.join(app_name + ".pid"))
        .working_directory(work_dir)
        .stderr(log);

    daemonize.start()?;

    Ok(())
}
