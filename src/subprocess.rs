use anyhow::{anyhow, Result};
use libc::pid_t;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    ffi::c_int,
    fs,
    io::{ErrorKind, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use subprocess::{Popen, PopenConfig, Redirection};
use sysinfo::{Pid, ProcessExt, System, SystemExt};

#[derive(Serialize, Deserialize)]
pub struct SocketMessage {
    pub event: SocketEvent,
}

#[derive(Serialize, Deserialize)]
pub enum SocketEvent {
    CommandHistory(Vec<String>),
    WriteStdin(String),
}

pub fn start(name: String, args: Vec<String>, app_dir: PathBuf) -> Result<()> {
    info!("Subprocess arguments: '{}'", args.join(" "));

    info!("Starting subprocess.");

    let mut subprocess = match Popen::create(
        &args,
        PopenConfig {
            stdout: Redirection::Merge,
            stdin: Redirection::Pipe,
            ..Default::default()
        },
    ) {
        Ok(subprocess) => subprocess,
        Err(err) => {
            error!("Error starting subprocess: {err}");
            return Err(anyhow!("Shutting down."));
        }
    };

    info!("Subprocess started.");

    let stdin = match subprocess.stdin.take() {
        Some(stdin) => stdin,
        None => {
            error!("Subprocess stdin was empty, terminating subprocess.");
            subprocess.terminate()?;
            return Err(anyhow!("Shutting down."));
        }
    };

    if let Some(status) = subprocess.poll() {
        error!(
            "Checked if subprocess was running and it returned: {:?}.",
            status
        );

        return Err(anyhow!("Shutting down."));
    }

    let socket_address = app_dir.join(name.clone() + ".sock");
    let pid_path = app_dir.join(name + ".pid");

    drop(app_dir);

    let pid = subprocess.pid().unwrap();

    if let Err(err) = append_pid(&pid_path, &pid) {
        error!("Error appending PID to file: {err}");
        subprocess.terminate()?;
        return Err(anyhow!("Shutting down."));
    };

    let socket_addr = socket_address.clone();
    let pid_parsed = Pid::from(pid as usize);

    let command_history = Arc::new(Mutex::new(Vec::new()));

    thread::Builder::new()
        .name(String::from("subprocess_socket"))
        .spawn(move || {
            let listener = match UnixListener::bind(&socket_addr) {
                Ok(socket) => socket,
                Err(err) => {
                    error!("Error connecting to socket: {err}");
                    terminate(&pid_parsed);
                    return;
                }
            };

            for client in listener.incoming() {
                match client {
                    Ok(mut stream) => {
                        let mut stdin_clone = stdin.try_clone().unwrap();
                        let history = command_history.clone();

                        thread::spawn(move || {
                            let mut history = history.lock().unwrap();
                            let mut received = vec![0u8; 1024];
                            let mut read: usize = 0;

                            while read_socket_stream(&mut stream, &mut received, &mut read) > 0 {
                                match serde_json::from_slice::<SocketMessage>(&received[..read]) {
                                    Ok(message) => {
                                        match message.event {
                                            SocketEvent::WriteStdin(content) => {
                                                info!("Command received: '{}'", &content);
                                                history.insert(0, content.to_string());

                                                let cmd = content.trim().to_owned() + "\n";

                                                if let Err(err) =
                                                    stdin_clone.write_all(cmd.as_bytes())
                                                {
                                                    error!(
                                                        "Error writing to subprocess stdin: {err}."
                                                    );

                                                    // Should any error here shutdown and exit?
                                                    // Only exiting if the pipe is closed for now
                                                    if err.kind() == ErrorKind::BrokenPipe {
                                                        terminate(&pid_parsed);
                                                        break;
                                                    }
                                                }

                                                if let Err(err) = stdin_clone.flush() {
                                                    error!(
                                                        "Error flushing subprocess stdin: {err}."
                                                    );
                                                }
                                            }
                                            SocketEvent::CommandHistory(_) => {
                                                let event = serde_json::to_vec(&SocketMessage {
                                                    event: SocketEvent::CommandHistory(
                                                        history.clone(),
                                                    ),
                                                })
                                                .unwrap();
                                                stream.write_all(&event).unwrap();
                                                stream.flush().unwrap();
                                            }
                                        }
                                    }
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

fn append_pid(pid_path: &PathBuf, pid: &u32) -> Result<()> {
    let mut pid_file = fs::OpenOptions::new()
        .read(true)
        .append(true)
        .open(pid_path)?;
    pid_file.write_all(pid.to_string().as_bytes())?;
    pid_file.flush()?;
    Ok(())
}

// https://codereview.stackexchange.com/questions/243693/rust-multi-cliented-tcp-server-library
pub fn read_socket_stream(stream: &mut UnixStream, received: &mut [u8], read: &mut usize) -> usize {
    *read = stream.read(received).unwrap_or(0);
    *read
}

fn terminate(subprocess_pid: &Pid) {
    info!("Sending SIGTERM to subprocess.");

    if let Err(err) = check_and_send_signal(subprocess_pid, &15) {
        error!("{err}");
    }
}

pub fn check_and_send_signal(pid: &Pid, signal: &u8) -> Result<bool> {
    match get_app_process_envs(pid)? {
        Some(_) => {
            let subprocess_pid: usize = (*pid).into();
            let result = unsafe { libc::kill(subprocess_pid as pid_t, *signal as c_int) };

            if result == 0 {
                return Ok(true);
            }

            Err(anyhow!("Error sending signal: errno {result}."))
        }
        None => Ok(false),
    }
}

pub fn get_app_process_envs(pid: &Pid) -> Result<Option<(String, String, String, String)>> {
    let mut system = System::new();
    system.refresh_process(*pid);

    match system.process(*pid) {
        Some(process) => {
            let envs = process.environ();

            let env: Vec<&String> = envs
                .iter()
                .filter(|string| {
                    let env: Vec<&str> = string.split('=').collect();
                    env[0].starts_with("CRESCENT_APP_")
                })
                .collect();

            if !env.is_empty() {
                let app_name = env
                    .iter()
                    .map(|string| {
                        let env: Vec<&str> = string.split('=').collect();

                        if env[0] == "CRESCENT_APP_NAME" {
                            return env[1].to_string();
                        }

                        "".to_string()
                    })
                    .collect();

                let interpreter_args = env
                    .iter()
                    .map(|string| {
                        let env: Vec<&str> = string.split('=').collect();

                        if env[0] == "CRESCENT_APP_INTERPRETER_ARGS" {
                            return env[1].to_string();
                        }

                        "".to_string()
                    })
                    .collect();

                let application_args = env
                    .iter()
                    .map(|string| {
                        let env: Vec<&str> = string.split('=').collect();

                        if env[0] == "CRESCENT_APP_ARGS" {
                            return env[1].to_string();
                        }

                        "".to_string()
                    })
                    .collect();

                let profile = env
                    .iter()
                    .map(|string| {
                        let env: Vec<&str> = string.split('=').collect();

                        if env[0] == "CRESCENT_APP_PROFILE" {
                            return env[1].to_string();
                        }

                        "".to_string()
                    })
                    .collect();

                return Ok(Some((
                    app_name,
                    interpreter_args,
                    application_args,
                    profile,
                )));
            }

            Err(anyhow!("Process did not return any crescent envs."))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{application::app_pids_by_name, test_util::util};

    #[test]
    fn unit_subprocess_terminate() -> Result<()> {
        let name = "subprocess_terminate";
        util::start_long_running_service(name)?;
        assert!(util::check_app_is_running(name)?);

        let pids = app_pids_by_name(&name.to_string())?;

        terminate(&pids[1]);

        assert!(!util::check_app_is_running(name)?);
        util::delete_app_folder(name)?;
        Ok(())
    }
}
