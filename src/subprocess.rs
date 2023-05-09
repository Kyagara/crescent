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

pub fn start(name: &String, interpreter: &String, args: &[String]) -> Result<()> {
    info!("Starting subprocess.");

    let mut subprocess = Exec::cmd(interpreter)
        .args(args)
        .stdout(Redirection::Merge)
        .stdin(Redirection::Pipe)
        .popen()?;

    info!("Subprocess started.");

    let stdin = match subprocess.stdin.take() {
        Some(stdin) => stdin,
        None => {
            error!("Subprocess stdin broken, terminating subprocess.");
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

    if let Err(err) = append_pid(pid_path, pid) {
        error!("Error appending PID to file: {err}");
        subprocess.terminate()?;
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

                                        if let Err(err) = stdin_clone.write_all(str.as_bytes()) {
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
