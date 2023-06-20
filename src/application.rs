use crate::{crescent, subprocess::SocketEvent};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    str::FromStr,
};
use sysinfo::Pid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplicationStatus {
    pub name: String,
    pub interpreter_args: Vec<String>,
    pub application_args: Vec<String>,
    pub profile: String,
    pub cmd: Vec<String>,
}

impl ApplicationStatus {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            interpreter_args: vec![],
            application_args: vec![],
            profile: String::new(),
            cmd: vec![],
        }
    }
}

pub fn app_dir_by_name(name: &String) -> Result<PathBuf> {
    let mut crescent_dir = crescent::crescent_dir()?;
    crescent_dir.push("apps");
    crescent_dir.push(name);
    Ok(crescent_dir)
}

pub fn get_app_socket(name: &String) -> Result<PathBuf> {
    let mut socket_dir = app_dir_by_name(name)?;
    socket_dir.push(format!("{}.sock", name));
    Ok(socket_dir)
}

pub fn app_pids_by_name(name: &String) -> Result<Vec<Pid>> {
    let application_path = app_dir_by_name(name)?;

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

pub fn get_app_status(name: &String) -> Result<ApplicationStatus> {
    let socket_dir = get_app_socket(name)?;

    let mut stream = UnixStream::connect(socket_dir)
        .context(format!("Error connecting to '{}' socket.", name))?;

    let event = serde_json::to_vec(&SocketEvent::RetrieveStatus(ApplicationStatus::new()))?;

    stream.write_all(&event)?;
    stream.flush()?;

    let mut received = vec![0u8; 2024];
    let read = stream.read(&mut received)?;

    Ok(serde_json::from_slice::<ApplicationStatus>(
        &received[..read],
    )?)
}

pub fn ping_app(name: &String) -> Result<SocketEvent> {
    let socket_dir = get_app_socket(name)?;

    let mut stream = UnixStream::connect(socket_dir)
        .context(format!("Error connecting to '{}' socket.", name))?;

    let event = serde_json::to_vec(&SocketEvent::Ping())?;

    stream.write_all(&event)?;
    stream.flush()?;

    let mut received = vec![0u8; 1024];
    let read = stream.read(&mut received)?;

    Ok(serde_json::from_slice::<SocketEvent>(&received[..read])?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crescent::crescent_dir;
    use std::fs::remove_dir_all;

    #[test]
    fn unit_application_dir_created() -> Result<()> {
        let mut home_path = crescent_dir()?;
        home_path.push("apps/test_app");
        let app_name = String::from("test_app");

        fs::create_dir_all(home_path.clone())?;

        assert_eq!(app_dir_by_name(&app_name)?, home_path);

        remove_dir_all(home_path)?;

        Ok(())
    }
}
