use crate::{application, tail};
use anyhow::{anyhow, Result};
use clap::Args;
use crossbeam::channel::{tick, unbounded, Receiver, Sender};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, LevelFilter};
use std::{
    io::{self, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    thread,
    time::Duration,
};
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use tui_input::Input;
use tui_logger::{init_logger, set_default_level, TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};

#[derive(Args)]
#[command(about = "Attach to an application.")]
pub struct AttachArgs {
    #[arg(help = "Application name.")]
    pub name: String,
}

struct AttachTerminal {
    app_name: String,
    logger: TuiWidgetState,
    input: Input,
    running: bool,
}

impl AttachTerminal {
    fn new(app_name: String) -> AttachTerminal {
        AttachTerminal {
            app_name,
            input: Input::default(),
            logger: TuiWidgetState::new(),
            running: true,
        }
    }
}

enum TerminalEvent {
    CrosstermEvent(Event),
    Log(String),
    Stats(String),
}

impl AttachArgs {
    pub fn run(self) -> Result<()> {
        if !application::app_already_running(&self.name)? {
            return Err(anyhow!("Application not running."));
        }

        init_logger(LevelFilter::Debug).unwrap();
        set_default_level(LevelFilter::Debug);

        let pids = application::app_pids_by_name(&self.name)?;

        let (sender, receiver) = unbounded();
        let (socket_sender, socket_receiver): (Sender<String>, Receiver<String>) = unbounded();
        let (log_sender, log_receiver): (Sender<String>, Receiver<String>) = unbounded();

        let app_dir = application::app_dir_by_name(&self.name)?;
        let log_dir = app_dir.join(self.name.clone() + ".log");
        let socket_dir = app_dir.join(self.name.clone() + ".sock");

        event_read_handler(sender.clone());
        log_handler(log_dir, sender.clone(), log_sender, log_receiver)?;
        stats_handler(pids[1], sender);
        socket_handler(socket_dir, socket_receiver)?;

        let mut app = AttachTerminal::new(self.name);
        let mut stats_list = String::from("Waiting for stats.");

        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| ui(f, &mut app, &stats_list))?;

        while app.running {
            match receiver.recv()? {
                TerminalEvent::CrosstermEvent(event) => match event {
                    Event::Key(key) => {
                        match key.code {
                            KeyCode::Enter => {
                                let input = app.input.value().to_string();

                                if input.is_empty() {
                                    continue;
                                }

                                app.input.reset();
                                let message = format!("{}\n", input);
                                socket_sender.send(message)?;
                                app.logger.transition(&TuiWidgetEvent::EscapeKey);
                            }
                            KeyCode::PageUp => {
                                app.logger.transition(&TuiWidgetEvent::PrevPageKey);
                            }
                            KeyCode::PageDown => {
                                app.logger.transition(&TuiWidgetEvent::NextPageKey);
                            }
                            KeyCode::Esc => break,
                            _ => {
                                tui_input::backend::crossterm::EventHandler::handle_event(
                                    &mut app.input,
                                    &event,
                                );
                            }
                        };

                        terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                    }
                    Event::Mouse(mouse) => {
                        match mouse.kind {
                            MouseEventKind::ScrollDown => {
                                app.logger.transition(&TuiWidgetEvent::NextPageKey);
                            }
                            MouseEventKind::ScrollUp => {
                                app.logger.transition(&TuiWidgetEvent::PrevPageKey);
                            }
                            _ => {}
                        };

                        terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                    }
                    Event::Resize(_, _) => {
                        terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                    }
                    _ => {
                        terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                    }
                },
                TerminalEvent::Log(content) => {
                    if content.is_empty() {
                        continue;
                    }

                    debug!("{content}");

                    terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                }
                TerminalEvent::Stats(stats) => {
                    stats_list = stats;
                    terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                }
            }
        }

        terminal::disable_raw_mode()?;

        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;

        terminal.show_cursor()?;

        Ok(())
    }
}

fn event_read_handler(input: Sender<TerminalEvent>) {
    thread::spawn(move || loop {
        input
            .send(TerminalEvent::CrosstermEvent(
                crossterm::event::read().unwrap(),
            ))
            .unwrap()
    });
}

fn socket_handler(
    socket_dir: PathBuf,
    socket_receiver: Receiver<String>,
) -> Result<(), anyhow::Error> {
    if !socket_dir.exists() {
        return Err(anyhow!("Socket file does not exist."));
    }

    let mut stream = UnixStream::connect(socket_dir)?;

    thread::spawn(move || {
        for received in socket_receiver {
            stream.write_all(received.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    });

    Ok(())
}

fn log_handler(
    log_dir: PathBuf,
    sender: Sender<TerminalEvent>,
    log_sender: Sender<String>,
    log_receiver: Receiver<String>,
) -> Result<(), anyhow::Error> {
    let mut log = tail::Tail::new(log_dir)?;

    let lines = log.read_lines(200)?;

    for line in lines {
        sender.send(TerminalEvent::Log(line))?;
    }

    thread::spawn(move || log.watch(&log_sender).unwrap());

    thread::spawn(move || {
        for line in log_receiver {
            sender.send(TerminalEvent::Log(line)).unwrap();
        }
    });

    Ok(())
}

fn stats_handler(pid: Pid, sender: Sender<TerminalEvent>) {
    let ticker = tick(Duration::from_secs(2));

    let mut system = System::new();
    system.refresh_processes();
    system.refresh_memory();

    let cpu_count = system.physical_core_count().unwrap() as f32;

    thread::spawn(move || {
        while ticker.recv().is_ok() {
            system.refresh_processes();
            system.refresh_memory();

            let process = match system.process(pid) {
                Some(process) => process,
                None => {
                    sender
                        .send(TerminalEvent::Stats(String::from(
                            "Error retrieving process information.",
                        )))
                        .unwrap();
                    break;
                }
            };

            let memory = process.memory() as f64 / system.total_memory() as f64 * 100.0;

            let load = system.load_average();

            let info = format!(
                "cpu: {:.2}% | mem: {:.2}% ({} Mb) | system load: {}, {}, {}",
                process.cpu_usage() / cpu_count,
                memory,
                process.memory() / 1024 / 1024,
                load.one,
                load.five,
                load.fifteen,
            );

            sender.send(TerminalEvent::Stats(info)).unwrap();
        }
    });
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut AttachTerminal, stats_list: &String) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Max(3), Constraint::Max(3)].as_ref())
        .split(f.size());

    let text_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::LightCyan);

    let app_name = Span::styled(app.app_name.as_str(), text_style);

    let tui_logger = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(app_name)
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL),
        )
        .output_level(None)
        .output_timestamp(None)
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(&app.logger);

    f.render_widget(tui_logger, chunks[0]);

    let stats_text = Span::styled("Stats", text_style);

    let stats = Paragraph::new(stats_list.to_owned())
        .block(Block::default().borders(Borders::ALL).title(stats_text));

    f.render_widget(stats, chunks[1]);

    let input_text = Span::styled("Input", text_style);

    let tui_input = Paragraph::new(app.input.value())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title(input_text));

    f.render_widget(tui_input, chunks[2]);

    f.set_cursor(
        chunks[2].x + app.input.visual_cursor() as u16 + 1,
        chunks[2].y + 1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use assert_cmd::Command;
    use predicates::{prelude::predicate, Predicate};
    use std::{
        env::{self, temp_dir},
        fs::{self, remove_file, File},
        os::unix::net::UnixListener,
    };

    #[test]
    fn unit_attach_handlers() -> Result<()> {
        let temp_dir = temp_dir();
        let socket_dir = temp_dir.join("crescent_temp_attach_socket_handler.sock");
        let log_dir = temp_dir.join("crescent_temp_attach_log_handler.log");

        if socket_dir.exists() {
            remove_file(&socket_dir)?;
        }

        if log_dir.exists() {
            remove_file(&log_dir)?;
        }

        File::create(&log_dir)?.write_all(b"data")?;

        let _socket = UnixListener::bind(&socket_dir)?;

        let (sender, receiver) = unbounded();
        let (socket_sender, socket_receiver): (Sender<String>, Receiver<String>) = unbounded();
        let (log_sender, log_receiver): (Sender<String>, Receiver<String>) = unbounded();
        let pid = Pid::from(std::process::id() as usize);

        event_read_handler(sender.clone());
        log_handler(log_dir, sender.clone(), log_sender.clone(), log_receiver)?;
        stats_handler(pid, sender);
        socket_handler(socket_dir, socket_receiver)?;

        loop {
            if let TerminalEvent::Stats(a) = receiver.recv_timeout(Duration::from_secs(3))? {
                let p = predicates::str::contains("load");
                assert!(p.eval(&a));
                break;
            }
        }

        log_sender.send("log".to_string())?;
        socket_sender.send("input".to_string())?;

        Ok(())
    }

    #[test]
    fn unit_attach_draw_ui() -> Result<()> {
        let mut cmd = Command::cargo_bin("cres")?;
        let name = String::from("attach_draw_ui");
        let args = [
            "start",
            "./tools/long_running_service.py",
            "-i",
            "python3",
            "-n",
            &name,
        ];

        cmd.args(args);

        cmd.assert()
            .success()
            .stderr(predicate::str::contains("Starting daemon."));

        // Sleeping to make sure the process started
        thread::sleep(std::time::Duration::from_secs(1));

        let mut app = AttachTerminal::new(name.clone());
        let stats_list = String::from("Waiting for stats.");

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| ui(f, &mut app, &stats_list))?;

        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        let mut cmd = Command::cargo_bin("cres")?;

        cmd.args(["signal", &name, "15"]);

        cmd.assert().success().stdout("Signal sent.\n");

        let home = env::var("HOME").context("Error getting HOME env.")?;
        let mut crescent_dir = PathBuf::from(home);
        crescent_dir.push(".crescent/apps");

        if !crescent_dir.exists() {
            fs::create_dir_all(&crescent_dir)?;
        }

        crescent_dir.push(name.clone());

        if crescent_dir.exists() {
            fs::remove_dir_all(&crescent_dir)?;
        }

        Ok(())
    }
}
