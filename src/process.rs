use crate::directory::{self};
use anyhow::{anyhow, Context, Result};
use crossbeam::channel::{unbounded, Receiver, Sender};
use daemonize::Daemonize;
use log::{error, info};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    process::exit,
    str::{from_utf8, FromStr},
    thread,
};
use subprocess::{Exec, Redirection};
use sysinfo::Pid;

pub struct Application {
    pub name: String,
    pub interpreter: String,
    pub args: Vec<String>,
    pub file_path: String,
    pub app_dir: PathBuf,
    pub work_dir: PathBuf,
}

impl Application {
    pub fn new(
        file_path: String,
        app_name: Option<String>,
        interpreter: Option<String>,
        arguments: Option<String>,
    ) -> Result<Application> {
        let file_path_buf = PathBuf::from(&file_path);

        if !file_path_buf.exists() {
            return Err(anyhow!(format!("File '{}' not found", &file_path)));
        }

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

        let app_dir = directory::application_dir_by_name(&name)?;

        let mut interpreter = interpreter.unwrap_or(String::new());

        let file_path_str = file_path.clone();

        let mut args = vec![];

        match interpreter.as_str() {
            "java" => {
                interpreter = "java".to_string();
                args.push(String::from("-jar"));
                args.push(file_path_str)
            }
            "" => interpreter = file_path_str,
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
            interpreter,
            args,
        })
    }

    pub fn start(self) -> Result<()> {
        daemonize(
            self.app_dir.clone(),
            self.name.clone(),
            self.work_dir.clone(),
        )?;

        self.start_subprocess()?;

        Ok(())
    }

    fn start_subprocess(self) -> Result<()> {
        info!("Starting subprocess.");

        let mut process = Exec::cmd(&self.interpreter)
            .args(&self.args)
            .stdout(Redirection::Merge)
            .stdin(Redirection::Pipe)
            .popen()?;

        let mut stdin = process.stdin.take().unwrap();

        let process_path = directory::application_dir_by_name(&self.name)?;

        let address = process_path.join(self.name.clone() + ".sock");

        let socket_path = address.clone();

        let pid_path = process_path.join(self.name + ".pid");
        let mut pid_file = OpenOptions::new().read(true).append(true).open(&pid_path)?;

        let pid = process.pid().unwrap().to_string();

        pid_file.write_all(pid.as_bytes())?;

        thread::spawn(move || {
            info!("Subprocess started.");

            match process.wait() {
                Ok(status) => {
                    info!("Process exited with status: {:?}.", status);
                }
                Err(err) => {
                    error!("Error opening process: {err}.");
                }
            }

            info!("Removing socket.");
            match fs::remove_file(socket_path) {
                Ok(_) => {
                    info!("Socket file removed.");
                }
                Err(err) => {
                    error!("Error removing socket file: {err}.");
                }
            }

            info!("Shutting down crescent.");
            exit(0)
        });

        let (sender, receiver): (Sender<String>, Receiver<String>) = unbounded();

        thread::spawn(move || {
            for received in receiver {
                info!("Command sent: '{}'", received.trim());
                stdin.write_all(received.as_bytes()).unwrap();
                stdin.flush().unwrap();
            }
        });

        let socket = UnixListener::bind(address)?;

        for stream in socket.incoming() {
            match stream {
                Ok(mut stream) => {
                    let sender_clone = sender.clone();

                    thread::spawn(move || {
                        let mut recv_buf = vec![0u8; 1024];
                        let mut bytes_read: usize = 0;

                        // https://codereview.stackexchange.com/questions/243693/rust-multi-cliented-tcp-server-library
                        while read_stream(&mut stream, &mut recv_buf, &mut bytes_read) > 0 {
                            match from_utf8(&recv_buf[..bytes_read]) {
                                Ok(str) => sender_clone.send(str.to_string()).unwrap(),
                                Err(err) => {
                                    error!("Error converting message to a string slice: {err}")
                                }
                            };
                        }
                    });
                }
                Err(err) => {
                    error!("Socket error: {err}")
                }
            }
        }

        Ok(())
    }
}

fn read_stream(stream: &mut UnixStream, recv_buf: &mut [u8], read_bytes: &mut usize) -> usize {
    *read_bytes = stream.read(recv_buf).unwrap_or(0);
    *read_bytes
}

pub fn process_pid_by_name(name: &String) -> Result<Vec<Pid>> {
    let application_path = directory::application_dir_by_name(name)?;

    let app_name = application_path
        .file_name()
        .context("Error extracting file name.")?
        .to_str()
        .context("Error converting OsStr to str.")?
        .to_string();

    let mut pid_path = application_path;
    pid_path.push(app_name + ".pid");

    let pid_file = fs::read_to_string(pid_path).context("Error reading PID file.")?;

    let pids: Vec<Pid> = pid_file
        .split('\n')
        .map(|x| Pid::from_str(x).context("Error parsing Pid.").unwrap())
        .collect();

    Ok(pids)
}

fn daemonize(app_dir: PathBuf, app_name: String, work_dir: PathBuf) -> Result<()> {
    let log = File::create(app_dir.join(app_name.clone() + ".log"))?;

    println!("Starting daemon.");

    let daemonize = Daemonize::new()
        .pid_file(app_dir.join(app_name + ".pid"))
        .working_directory(work_dir)
        .stderr(log);

    daemonize.start()?;

    info!("Daemon started.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_application_struct() -> Result<()> {
        let name = String::from("app_name");
        let t = temp_file::empty();
        let file_path = t.path().to_str().unwrap().to_string();
        let interpreter = String::from("java");
        let arguments = String::from("argument");

        let mut application = Application::new(
            file_path.clone(),
            Some(name),
            Some(interpreter),
            Some(arguments),
        )?;

        assert_eq!(application.name, "app_name");
        assert_eq!(application.file_path, file_path);
        assert_eq!(application.interpreter, "java");
        assert_eq!(application.args, vec!["-jar", &file_path, "argument"]);

        application = Application::new(file_path.clone(), None, None, None)?;

        assert_eq!(
            application.name,
            t.path().file_stem().unwrap().to_str().unwrap().to_string()
        );
        assert_eq!(application.file_path, file_path);
        assert_eq!(application.interpreter, file_path);
        assert_eq!(application.args, Vec::<String>::new());

        Ok(())
    }
}
