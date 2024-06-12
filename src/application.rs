use std::{
    fs,
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    str::FromStr,
};

use crate::{commands::start::StartArgs, crescent, subprocess::SocketEvent};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::Pid;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Application {
    pub name: String,
    pub file_path: PathBuf,
    pub stop_command: Option<String>,
    pub cmd: Vec<String>,
    pub start_args: StartArgs,
}

pub fn check_app_exists(name: &String) -> Result<PathBuf> {
    let app_dir = app_dir_by_name(name)?;
    if !app_dir.exists() {
        return Err(anyhow!("Application does not exist."));
    }
    Ok(app_dir)
}

pub fn app_dir_by_name(name: &String) -> Result<PathBuf> {
    let mut app_dir = crescent::crescent_dir()?;
    app_dir.push("apps");
    app_dir.push(name);
    Ok(app_dir)
}

pub fn get_app_socket(name: &String) -> Result<PathBuf> {
    let mut socket_dir = app_dir_by_name(name)?;
    socket_dir.push(format!("{}.sock", name));
    Ok(socket_dir)
}

pub fn app_pids_by_name(name: &String) -> Result<Vec<Pid>> {
    let mut application_path = app_dir_by_name(name)?;

    let app_name = application_path
        .file_name()
        .context("Error extracting file name.")?
        .to_str()
        .context("Error converting OsStr to str.")?
        .to_string();

    application_path.push(app_name + ".pid");

    if !application_path.exists() {
        return Ok(vec![]);
    }

    let pid_file =
        fs::read_to_string(application_path).context("Error reading PID file to string.")?;

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

pub fn app_already_running(name: &String) -> Result<bool> {
    match app_pids_by_name(name) {
        Ok(pids) => {
            if pids.is_empty() || pids.len() < 2 {
                return Ok(false);
            }

            match ping_app(name) {
                Ok(_) => Ok(true),
                Err(err) => {
                    // This looks horrible
                    // If the error contains 'Error connecting to'
                    // it means that an error from socket has occurred.
                    // This might happen if you kill the crescent process
                    // which will not delete the socket file, and will
                    // error out if you try connecting to it.
                    if err.to_string().contains("Error connecting to") {
                        return Ok(false);
                    }
                    Err(err)
                }
            }
        }
        Err(err) => Err(err),
    }
}

pub fn get_app_info(name: &String) -> Result<Application> {
    let socket_dir = get_app_socket(name)?;

    let mut stream = UnixStream::connect(socket_dir)
        .context(format!("Error connecting to '{}' socket.", name))?;

    let event = serde_json::to_vec(&SocketEvent::RetrieveAppInfo(Box::default()))?;

    stream.write_all(&event)?;

    let mut received = vec![0u8; 1024];

    loop {
        let read = stream.read(&mut received)?;
        if read > 0 {
            return Ok(serde_json::from_slice::<Application>(&received[..read])?);
        }
    }
}

fn ping_app(name: &String) -> Result<SocketEvent> {
    let socket_dir = get_app_socket(name)?;

    let mut stream = UnixStream::connect(socket_dir)
        .context(format!("Error connecting to '{}' socket.", name))?;

    let event = serde_json::to_vec(&SocketEvent::Ping)?;

    stream.write_all(&event)?;

    let mut received = vec![0u8; 1024];

    loop {
        let read = stream.read(&mut received)?;
        if read > 0 {
            return Ok(serde_json::from_slice::<SocketEvent>(&received[..read])?);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crescent::crescent_dir;
    use std::fs::remove_dir_all;

    #[test]
    fn unit_app_dir_by_name() -> Result<()> {
        let mut home_path = crescent_dir()?;
        home_path.push("apps/test_app_dir_by_name");
        let app_name = String::from("test_app_dir_by_name");
        fs::create_dir_all(home_path.clone())?;

        assert_eq!(app_dir_by_name(&app_name)?, home_path);
        remove_dir_all(home_path)?;
        Ok(())
    }
}
