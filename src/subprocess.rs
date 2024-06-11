use crate::application::Application;
use anyhow::{anyhow, Result};
use libc::pid_t;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    ffi::c_int,
    fs::{self, File},
    io::{Error, ErrorKind, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use subprocess::{Popen, PopenConfig, Redirection};
use sysinfo::Pid;

#[derive(Serialize, Deserialize)]
pub enum SocketEvent {
    RetrieveAppInfo(Box<Application>),
    CommandHistory(Vec<String>),
    WriteStdin(String),
    Stop,
    Ping,
}

pub fn start(app_info: Application, app_dir: PathBuf) -> Result<()> {
    info!("Subprocess arguments: '{}'", app_info.cmd.join(" "));

    let socket_address = app_dir.join(app_info.name.clone() + ".sock");
    let pid_path = app_dir.join(app_info.name.clone() + ".pid");

    drop(app_dir);

    info!("Starting subprocess.");

    let (mut subprocess, stdin, pid) = match exec_subprocess(pid_path, app_info.cmd.clone()) {
        Ok(subprocess) => subprocess,
        Err(err) => {
            error!("{err}");
            return Err(anyhow!("Shutting down."));
        }
    };

    info!("Subprocess started.");

    let listener = match UnixListener::bind(&socket_address) {
        Ok(socket) => socket,
        Err(err) => {
            error!("Error starting socket listener: {err}.");
            subprocess.terminate()?;
            return Err(anyhow!("Shutting down."));
        }
    };

    let command_history = Arc::new(Mutex::new(Vec::new()));
    let app_info = Arc::new(Mutex::new(app_info));

    thread::Builder::new()
        .name(String::from("subprocess_socket"))
        .spawn(move || {
            for client in listener.incoming() {
                match client {
                    Ok(mut stream) => {
                        let mut stdin = stdin.try_clone().unwrap();
                        let history = command_history.clone();
                        let app_info = app_info.clone();

                        thread::spawn(move || {
                            let mut history = history.lock().unwrap();
                            let app_info = app_info.lock().unwrap();

                            let mut write_to_stdin = |cmd: String| {
                                if let Err(err) = stdin.write_all(cmd.as_bytes()) {
                                    error!("Error writing to subprocess stdin: {err}.");

                                    // Should any error here shutdown and exit?
                                    // Only exiting if the pipe is closed for now
                                    if err.kind() == ErrorKind::BrokenPipe {
                                        info!("Sending SIGTERM to subprocess.");
                                        if let Err(err) = send_unix_signal(pid, 15) {
                                            error!("{err}");
                                        }
                                    }
                                }
                            };

                            let mut received = vec![0u8; 1024];
                            let mut read: usize = 0;

                            while read_socket_stream(&mut stream, &mut received, &mut read) > 0 {
                                match serde_json::from_slice::<SocketEvent>(&received[..read]) {
                                    Ok(message) => match message {
                                        SocketEvent::WriteStdin(content) => {
                                            info!("Command received: '{}'", &content);
                                            history.insert(0, content.to_string());
                                            let cmd = content.trim().to_owned() + "\n";
                                            write_to_stdin(cmd)
                                        }
                                        SocketEvent::CommandHistory(_) => {
                                            let event = serde_json::to_vec(
                                                &SocketEvent::CommandHistory(history.clone()),
                                            )
                                            .unwrap();
                                            stream.write_all(&event).unwrap();
                                        }
                                        SocketEvent::RetrieveAppInfo(_) => {
                                            let event =
                                                serde_json::to_vec(&app_info.clone()).unwrap();
                                            stream.write_all(&event).unwrap();
                                        }
                                        SocketEvent::Ping => {
                                            let event =
                                                serde_json::to_vec(&SocketEvent::Ping).unwrap();
                                            stream.write_all(&event).unwrap();
                                        }

                                        SocketEvent::Stop => {
                                            info!("Received stop command.");

                                            match app_info.stop_command.clone() {
                                                Some(stop_command) => {
                                                    info!("Stop command found, forwarding it.");
                                                    write_to_stdin(stop_command)
                                                }
                                                None => {
                                                    info!("Sending SIGTERM to subprocess.");
                                                    if let Err(err) = send_unix_signal(pid, 15) {
                                                        error!("{err}");
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    },
                                    Err(err) => {
                                        error!("Error converting event to struct: {err}")
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

    match subprocess.wait() {
        Ok(status) => {
            info!("Subprocess exited with status: {:?}.", status);
        }
        Err(err) => {
            error!("Error waiting: {err}.");
        }
    }

    if socket_address.exists() {
        info!("Removing socket.");
        match fs::remove_file(socket_address) {
            Ok(_) => {
                info!("Socket file removed.");
            }
            Err(err) => {
                error!("Error removing socket file: {err}.");
            }
        };
    }

    info!("Shutting down.");

    Ok(())
}

fn exec_subprocess(pid_path: PathBuf, args: Vec<String>) -> Result<(Popen, File, Pid)> {
    let mut subprocess = match Popen::create(
        &args,
        PopenConfig {
            stdout: Redirection::Merge,
            stdin: Redirection::Pipe,
            ..Default::default()
        },
    ) {
        Ok(subprocess) => subprocess,
        Err(err) => return Err(anyhow!("Error starting subprocess: {err}.")),
    };

    if let Some(status) = subprocess.poll() {
        return Err(anyhow!(
            "Checked if subprocess was running and it returned status: {status:?}."
        ));
    }

    let stdin = match subprocess.stdin.take() {
        Some(stdin) => stdin,
        None => {
            subprocess.terminate()?;
            return Err(anyhow!(
                "Subprocess stdin was empty, terminating subprocess."
            ));
        }
    };

    let pid = subprocess.pid().expect("pid shouldn't be empty");

    if let Err(err) = append_pid(&pid_path, &pid) {
        subprocess.terminate()?;
        return Err(anyhow!("Error appending PID to file: {err}."));
    };

    Ok((subprocess, stdin, Pid::from(pid as usize)))
}

fn append_pid(pid_path: &PathBuf, pid: &u32) -> Result<()> {
    let mut pid_file = fs::OpenOptions::new()
        .read(true)
        .append(true)
        .open(pid_path)?;
    pid_file.write_all(pid.to_string().as_bytes())?;
    Ok(())
}

// https://codereview.stackexchange.com/questions/243693/rust-multi-cliented-tcp-server-library
pub fn read_socket_stream(stream: &mut UnixStream, received: &mut [u8], read: &mut usize) -> usize {
    *read = stream.read(received).unwrap_or(0);
    *read
}

pub fn send_unix_signal(pid: Pid, signal: u8) -> Result<()> {
    let subprocess_pid: usize = pid.into();

    let result = unsafe { libc::kill(subprocess_pid as pid_t, signal as c_int) };

    if result == 0 {
        return Ok(());
    }

    match Error::last_os_error().raw_os_error() {
        Some(errno) => {
            let error = match errno {
                1 => "(1) EPERM - An invalid signal was specified.".to_string(),
                3 => "(3) ESRCH - The calling process does not have permission to send the
                signal to any of the target processes."
                    .to_string(),
                22 => {
                    "(22) EINVAL - The target process or process group does not exist.".to_string()
                }
                _ => format!("({errno}) Unknown error - Error not documented by crescent."),
            };

            Err(anyhow!("Error sending signal: {error}"))
        }
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::app_pids_by_name;
    extern crate test_utils;

    #[test]
    fn unit_send_unix_signal() -> Result<()> {
        let name = "unit_send_unix_signal";
        test_utils::start_long_running_service(name)?;
        assert!(test_utils::check_app_is_running(name)?);

        let pids = app_pids_by_name(&name.to_string())?;
        send_unix_signal(pids[1], 15)?;
        test_utils::delete_app_folder(name)?;
        Ok(())
    }
}
