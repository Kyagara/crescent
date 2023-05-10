use crate::application;
use anyhow::{anyhow, Result};
use libc::pid_t;
use log::{error, info};
use std::{
    ffi::c_int,
    fs::{self, OpenOptions},
    io::{ErrorKind, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    str::from_utf8,
    thread,
};
use subprocess::{Exec, Redirection};
use sysinfo::{Pid, ProcessExt, System, SystemExt};

pub fn start(name: &String, interpreter: &String, args: &[String]) -> Result<()> {
    info!("Starting subprocess.");

    let mut subprocess = Exec::cmd(interpreter)
        .args(args)
        .stdout(Redirection::Merge)
        .stdin(Redirection::Pipe)
        .env("CRESCENT_APP_NAME", name)
        .popen()?;

    info!("Subprocess started.");

    let stdin = match subprocess.stdin.take() {
        Some(stdin) => stdin,
        None => {
            error!("Subprocess stdin was empty, terminating subprocess.");
            subprocess.terminate()?;
            return Err(anyhow!("Shutting down."));
        }
    };

    let application_path = match application::app_dir_by_name(name) {
        Ok(path) => path,
        Err(err) => {
            error!("Error retrieving application path: {err}");
            subprocess.terminate()?;
            return Err(anyhow!("Shutting down."));
        }
    };

    let socket_address = application_path.join(name.clone() + ".sock");
    let pid_path = application_path.join(name.clone() + ".pid");

    drop(application_path);

    if let Some(status) = subprocess.poll() {
        error!(
            "Checked if subprocess was running and it returned {:?}.",
            status
        );

        return Err(anyhow!("Shutting down."));
    }

    let pid = subprocess.pid().unwrap();

    if let Err(err) = append_pid(&pid_path, &pid) {
        error!("Error appending PID to file: {err}");
        subprocess.terminate()?;
        return Err(anyhow!("Shutting down."));
    };

    let socket_addr = socket_address.clone();
    let app_name = name.clone();
    let pid_parsed = Pid::from(pid as usize);

    thread::Builder::new()
        .name(String::from("subprocess_socket"))
        .spawn(move || {
            let listener = UnixListener::bind(&socket_addr).unwrap();

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut stdin_clone = stdin.try_clone().unwrap();
                        let socket = socket_addr.clone();
                        let app_name_clone = app_name.clone();

                        thread::spawn(move || {
                            let mut recv_buf = vec![0u8; 1024];
                            let mut bytes_read: usize = 0;

                            while read_stream(&mut stream, &mut recv_buf, &mut bytes_read) > 0 {
                                match from_utf8(&recv_buf[..bytes_read]) {
                                    Ok(str) => {
                                        info!("Command received: '{}'", str.trim());

                                        if let Err(err) = stdin_clone.write_all(str.as_bytes()) {
                                            error!("Error writing to subprocess stdin: {err}.");

                                            // Should any error here shutdown and exit?
                                            // Only exiting if the pipe is closed for now
                                            if err.kind() == ErrorKind::BrokenPipe {
                                                info!("Sending SIGTERM to subprocess.");
                                                terminate(&app_name_clone, &pid_parsed, &socket);
                                                break;
                                            }
                                        }

                                        if let Err(err) = stdin_clone.flush() {
                                            error!("Error flushing subprocess stdin: {err}.");
                                        }
                                    }
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
        })?;

    match subprocess.wait() {
        Ok(status) => {
            info!("Subprocess exited with status: {:?}.", status);
        }
        Err(err) => {
            error!("Error waiting: {err}.");
        }
    }

    terminate(name, &pid_parsed, &socket_address);

    info!("Shutting down.");

    Ok(())
}

fn append_pid(pid_path: &PathBuf, pid: &u32) -> Result<()> {
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
fn terminate(app_name: &String, subprocess_pid: &Pid, socket_path: &PathBuf) {
    if let Err(err) = check_and_send_signal(app_name, subprocess_pid, &15) {
        if err.to_string() != "Process does not exist." {
            error!("{err}");
        }
    }

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

pub fn check_and_send_signal(app_name: &String, pid: &Pid, signal: &u8) -> Result<()> {
    let is_running = is_running(app_name, pid)?;

    if is_running {
        let subprocess_pid: usize = (*pid).into();
        let result = unsafe { libc::kill(subprocess_pid as pid_t, *signal as c_int) };

        if result == 0 {
            return Ok(());
        }

        return Err(anyhow!("Error sending signal: errno {result}"));
    }

    Ok(())
}

fn is_running(name: &String, pid: &Pid) -> Result<bool> {
    let mut system = System::new();
    system.refresh_process(*pid);

    match system.process(*pid) {
        Some(subprocess) => {
            let envs = subprocess.environ();

            let env = envs.iter().find(|string| {
                let env: Vec<&str> = string.split('=').collect();
                env[0] == "CRESCENT_APP_NAME"
            });

            if let Some(file_name) = env {
                let values: Vec<&str> = file_name.split('=').collect();
                return Ok(name == values[1]);
            };

            Ok(false)
        }
        None => Err(anyhow!("Process does not exist.")),
    }
}
