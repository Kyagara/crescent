use daemonize::Daemonize;
use std::{
    env,
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
use sysinfo::{Pid, System, SystemExt};

pub struct Application {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub file_path: String,
    pub temp_dir: PathBuf,
    pub work_dir: PathBuf,
}

impl Application {
    pub fn new(
        file_path: String,
        app_name: Option<String>,
        command: Option<String>,
        arguments: Option<String>,
    ) -> Self {
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

        let temp_dir = application_temp_dir_by_name(name.clone());

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

        Application {
            file_path,
            name,
            temp_dir,
            work_dir,
            command,
            args,
        }
    }

    pub fn start(self) {
        daemonize(
            self.temp_dir.clone(),
            self.name.clone(),
            self.work_dir.clone(),
        );

        self.start_subprocess()
    }

    fn start_subprocess(self) {
        let mut process = Exec::cmd(&self.command)
            .args(&self.args)
            .stdout(Redirection::Merge)
            .stdin(Redirection::Pipe)
            .popen()
            .unwrap();

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

        let process_path = application_temp_dir_by_name(self.name.clone());

        let address = process_path.join(self.name + ".sock");

        let socket = UnixListener::bind(&address).unwrap();

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

pub fn app_already_exist(name: String) -> bool {
    let pid = process_pid_by_name(name);

    match pid {
        Some(pid) => {
            let mut system = System::new();
            system.refresh_all();

            match system.process(pid).is_some() {
                true => true,
                false => false,
            }
        }
        None => false,
    }
}

pub fn process_pid_by_name(name: String) -> Option<Pid> {
    let application_path = application_temp_dir_by_name(name);
    let app_name = application_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mut pid_path = application_path;
    pid_path.push(app_name + ".pid");

    let pid = match fs::read_to_string(pid_path) {
        Ok(pid) => Some(Pid::from_str(pid.trim()).unwrap()),
        Err(_) => None,
    };

    pid
}

fn daemonize(process_temp_dir: PathBuf, app_name: String, work_dir: PathBuf) {
    let log = File::create(process_temp_dir.join(app_name.clone() + ".log")).unwrap();

    println!("Starting daemon");

    let daemonize = Daemonize::new()
        .pid_file(process_temp_dir.join(app_name + ".pid"))
        .working_directory(work_dir)
        .stderr(log);

    daemonize.start().unwrap();
}

pub fn crescent_temp_dir() -> PathBuf {
    let mut crescent_dir = env::temp_dir();

    crescent_dir.push("crescent");

    crescent_dir
}

pub fn application_temp_dir_by_name(name: String) -> PathBuf {
    let mut crescent_dir = crescent_temp_dir();

    crescent_dir.push(name);

    crescent_dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn crescent_dir() {
        assert_eq!(crescent_temp_dir(), PathBuf::from("/tmp/crescent"));
    }

    #[test]
    fn temp_dir_by_name() {
        assert_eq!(
            application_temp_dir_by_name(String::from("app")),
            PathBuf::from("/tmp/crescent/app")
        );
    }
}
