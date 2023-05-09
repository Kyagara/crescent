use crate::directory;
use anyhow::{anyhow, Context, Result};
use libc::pid_t;
use log::{error, info};
use std::{
    ffi::c_int,
    fs::{self, OpenOptions},
    io::{ErrorKind, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
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

        let file_full_path = fs::canonicalize(&file_path_buf)?;

        if !file_full_path.exists() {
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

        let file_path_str = file_full_path.to_str().unwrap().to_string();

        let mut args = vec![];

        match interpreter.as_str() {
            "java" => {
                interpreter = "java".to_string();
                args.push(String::from("-jar"));
                args.push(file_path_str.clone())
            }
            "" => interpreter = file_path_str.clone(),
            _ => args.push(file_path_str.clone()),
        }

        if let Some(arguments) = arguments {
            args.push(arguments);
        }

        Ok(Application {
            file_path: file_path_str,
            name,
            app_dir,
            work_dir,
            interpreter,
            args,
        })
    }

    pub fn start(self) -> Result<()> {
        info!("Starting subprocess.");

        let mut process = Exec::cmd(&self.interpreter)
            .args(&self.args)
            .stdout(Redirection::Merge)
            .stdin(Redirection::Pipe)
            .popen()?;

        info!("Subprocess started.");

        let stdin = match process.stdin.take() {
            Some(stdin) => stdin,
            None => {
                error!("Subprocess stdin broken, terminating subprocess.");
                process.terminate()?;
                return Err(anyhow!("Shutting down."));
            }
        };

        let application_path = match directory::application_dir_by_name(&self.name) {
            Ok(path) => path,
            Err(err) => {
                error!("Error retrieving application path: {err}");
                process.terminate()?;
                return Err(anyhow!("Shutting down."));
            }
        };

        let socket_address = application_path.join(self.name.clone() + ".sock");
        let pid_path = application_path.join(self.name.clone() + ".pid");

        drop(application_path);
        drop(self);

        if let Some(status) = process.poll() {
            error!(
                "Checked if subprocess was running and it returned {:?}.",
                status
            );

            return Err(anyhow!("Shutting down."));
        }

        let pid = process.pid().unwrap();

        if let Err(err) = append_pid(pid_path, pid) {
            error!("Error appending PID to file: {err}");
            process.terminate()?;
            return Err(anyhow!("Shutting down."));
        };

        let socket_addr = socket_address.clone();

        thread::Builder::new()
            .name(String::from("subprocess_socket"))
            .spawn(move || {
                let listener = UnixListener::bind(&socket_addr).unwrap();

                for stream in listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                            let mut stdin_clone = stdin.try_clone().unwrap();
                            let socket = socket_addr.clone();

                            thread::spawn(move || {
                                let mut recv_buf = vec![0u8; 1024];
                                let mut bytes_read: usize = 0;

                                while read_stream(&mut stream, &mut recv_buf, &mut bytes_read) > 0 {
                                    match from_utf8(&recv_buf[..bytes_read]) {
                                        Ok(str) => {
                                            info!("Command received: '{}'", str.trim());

                                            if let Err(err) = stdin_clone.write_all(str.as_bytes())
                                            {
                                                error!("Error writing to subprocess stdin: {err}.");

                                                // Should any error here shutdown and exit?
                                                // Only exiting if the pipe is closed for now
                                                if err.kind() == ErrorKind::BrokenPipe {
                                                    terminate(pid, &socket);
                                                    break;
                                                }
                                            }

                                            if let Err(err) = stdin_clone.flush() {
                                                error!("Error flushing subprocess stdin: {err}.");
                                            }
                                        }
                                        Err(err) => {
                                            error!(
                                                "Error converting message to a string slice: {err}"
                                            )
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
            })?;

        match process.wait() {
            Ok(status) => {
                info!("Subrocess exited with status: {:?}.", status);
            }
            Err(err) => {
                error!("Error waiting: {err}.");
            }
        }

        terminate(pid, &socket_address);

        info!("Shutting down.");

        Ok(())
    }
}

fn append_pid(pid_path: PathBuf, pid: u32) -> Result<()> {
    let mut pid_file = OpenOptions::new().read(true).append(true).open(pid_path)?;
    pid_file.write_all(pid.to_string().as_bytes())?;
    pid_file.flush()?;
    Ok(())
}

// https://codereview.stackexchange.com/questions/243693/rust-multi-cliented-tcp-server-library
fn read_stream(stream: &mut UnixStream, recv_buf: &mut [u8], read_bytes: &mut usize) -> usize {
    *read_bytes = stream.read(recv_buf).unwrap_or(0);
    *read_bytes
}

// This might be called two times if the stdin pipe is broken
fn terminate(subprocess_pid: u32, socket_path: &PathBuf) {
    unsafe {
        let process_exists = libc::kill(subprocess_pid as pid_t, 0 as c_int);

        if process_exists == 0 {
            info!("Sending SIGTERM to subprocess.");
            libc::kill(subprocess_pid as pid_t, 15 as c_int);
        }
    };

    if socket_path.exists() {
        info!("Removing socket.");
        match fs::remove_file(socket_path) {
            Ok(_) => {
                info!("Socket file removed.");
            }
            Err(err) => {
                error!("Error removing socket file: {err}.");
            }
        };
    }
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

    if !pid_path.exists() {
        return Ok(vec![]);
    }

    let pid_file = fs::read_to_string(pid_path).context("Error reading PID file to string.")?;

    let mut pid_strs: Vec<&str> = pid_file.split('\n').collect();
    pid_strs.retain(|&x| !x.is_empty());

    if pid_strs.is_empty() {
        return Ok(vec![]);
    }

    let cres_pid =
        Pid::from_str(pid_strs[0]).with_context(|| format!("Error parsing PID {}", pid_strs[0]))?;

    if pid_strs.len() == 1 {
        return Ok(vec![cres_pid]);
    }

    let app_pid =
        Pid::from_str(pid_strs[1]).with_context(|| format!("Error parsing PID {}", pid_strs[1]))?;

    let pids: Vec<Pid> = vec![cres_pid, app_pid];

    Ok(pids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_new_application_struct() -> Result<()> {
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
