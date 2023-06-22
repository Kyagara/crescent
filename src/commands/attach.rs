use crate::{
    application,
    subprocess::{self, SocketEvent},
    tail,
};
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
    sync::Mutex,
    thread,
    time::Duration,
    vec,
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
    input: Mutex<Input>,
    running: bool,
    history: Vec<String>,
}

impl AttachTerminal {
    fn new(app_name: String) -> AttachTerminal {
        AttachTerminal {
            app_name,
            logger: TuiWidgetState::new(),
            running: true,
            input: Mutex::new(Input::default()),
            history: Vec::new(),
        }
    }
}

enum TerminalEvent {
    CrosstermEvent(Event),
    Log(Vec<String>),
    Stats(String),
    SocketEvent(SocketEvent),
}

impl AttachArgs {
    pub fn run(self) -> Result<()> {
        application::check_app_exists(&self.name)?;

        if !application::app_already_running(&self.name)? {
            return Err(anyhow!("Application not running."));
        }

        if !cfg!(test) {
            init_logger(LevelFilter::Debug).unwrap();
            set_default_level(LevelFilter::Debug);
        }

        let pids = application::app_pids_by_name(&self.name)?;

        let (sender, receiver) = unbounded();
        let (socket_sender, socket_receiver): (Sender<SocketEvent>, Receiver<SocketEvent>) =
            unbounded();
        let (log_sender, log_receiver): (Sender<String>, Receiver<String>) = unbounded();

        let app_dir = application::app_dir_by_name(&self.name)?;
        let log_dir = app_dir.join(self.name.clone() + ".log");
        let socket_dir = app_dir.join(self.name.clone() + ".sock");

        let mut app = AttachTerminal::new(self.name);
        let mut stats_list = String::from("Waiting for stats.");

        event_read_handler(sender.clone());
        log_handler(log_dir, sender.clone(), log_sender, log_receiver)?;
        stats_handler(pids[1], sender.clone());
        socket_handler(socket_dir, sender, socket_receiver)?;

        socket_sender.send(SocketEvent::CommandHistory(vec![]))?;

        if cfg!(test) {
            return Ok(());
        }

        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| ui(f, &mut app, &stats_list))?;

        let mut history_pos: i16 = -1;

        while app.running {
            match receiver.recv()? {
                TerminalEvent::CrosstermEvent(event) => match event {
                    Event::Key(key) => {
                        match key.code {
                            KeyCode::Enter => {
                                let mut input = app.input.lock().unwrap();
                                let content = input.value().to_string();

                                if content.trim().is_empty() {
                                    continue;
                                }

                                input.reset();

                                app.history.insert(0, content.clone());
                                history_pos = -1;
                                socket_sender.send(SocketEvent::WriteStdin(content))?;
                                app.logger.transition(&TuiWidgetEvent::EscapeKey);
                            }
                            KeyCode::PageUp => {
                                app.logger.transition(&TuiWidgetEvent::PrevPageKey);
                            }
                            KeyCode::PageDown => {
                                app.logger.transition(&TuiWidgetEvent::NextPageKey);
                            }
                            KeyCode::Up => {
                                if history_pos < app.history.len() as i16 - 1 {
                                    history_pos += 1;
                                    let mut input = app.input.lock().unwrap();
                                    let input_clone = input.clone();
                                    *input = input_clone
                                        .with_value(app.history[history_pos as usize].to_string());
                                }
                            }
                            KeyCode::Down => {
                                if history_pos > 0 && history_pos < app.history.len() as i16 {
                                    history_pos -= 1;
                                    let mut input = app.input.lock().unwrap();
                                    let input_clone = input.clone();
                                    *input = input_clone
                                        .with_value(app.history[history_pos as usize].to_string());
                                }
                            }
                            KeyCode::Esc => break,
                            _ => {
                                tui_input::backend::crossterm::EventHandler::handle_event(
                                    &mut *app.input.lock().unwrap(),
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
                TerminalEvent::Log(lines) => {
                    if lines.is_empty() {
                        continue;
                    }

                    for line in lines {
                        if line.is_empty() {
                            continue;
                        }

                        debug!("{line}");
                    }

                    terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                }
                TerminalEvent::Stats(stats) => {
                    stats_list = stats;
                    terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                }
                TerminalEvent::SocketEvent(message) => {
                    if let SocketEvent::CommandHistory(history) = message {
                        app.history = history
                    }
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
    sender: Sender<TerminalEvent>,
    socket_receiver: Receiver<SocketEvent>,
) -> Result<(), anyhow::Error> {
    if !socket_dir.exists() {
        return Err(anyhow!("Socket file does not exist."));
    }

    let mut stream = UnixStream::connect(socket_dir)?;
    let mut s = stream.try_clone()?;

    thread::spawn(move || {
        for received in socket_receiver {
            let json = match received {
                SocketEvent::CommandHistory(_content) => {
                    Some(serde_json::to_vec(&SocketEvent::CommandHistory(_content)))
                }
                SocketEvent::WriteStdin(command) => {
                    Some(serde_json::to_vec(&SocketEvent::WriteStdin(command)))
                }
                _ => None,
            };

            if let Some(event) = json {
                match event {
                    Ok(event) => {
                        s.write_all(&event).unwrap();
                    }
                    Err(err) => {
                        debug!("Error serializing event: {err}")
                    }
                }
            }
        }
    });

    let mut received = vec![0u8; 1024];
    let mut read: usize = 0;

    thread::spawn(move || {
        while subprocess::read_socket_stream(&mut stream, &mut received, &mut read) > 0 {
            match serde_json::from_slice::<SocketEvent>(&received[..read]) {
                Ok(message) => {
                    sender.send(TerminalEvent::SocketEvent(message)).unwrap();
                }
                Err(err) => {
                    debug!("Error converting socket message to struct: {err}")
                }
            };
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

    sender.send(TerminalEvent::Log(lines))?;

    thread::spawn(move || log.watch(&log_sender));

    thread::spawn(move || {
        for line in log_receiver {
            sender.send(TerminalEvent::Log([line].to_vec())).unwrap();
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

    let input = app.input.lock().unwrap();

    let tui_input = Paragraph::new(input.value())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title(input_text));

    f.render_widget(tui_input, chunks[2]);

    f.set_cursor(
        chunks[2].x + input.visual_cursor() as u16 + 1,
        chunks[2].y + 1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test_utils;
    use predicates::Predicate;
    use std::{
        env::temp_dir,
        fs::{remove_file, File},
        io::Write,
        os::unix::net::UnixListener,
    };
    use tui::backend::TestBackend;

    #[test]
    fn unit_attach_run() -> Result<()> {
        let name = "attach_run".to_string();
        test_utils::start_long_running_service(&name)?;
        assert!(test_utils::check_app_is_running(&name)?);

        let command_args = AttachArgs { name: name.clone() };

        command_args.run()?;

        test_utils::shutdown_long_running_service(&name)?;
        test_utils::delete_app_folder(&name)?;
        Ok(())
    }

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
        let (socket_sender, socket_receiver): (Sender<SocketEvent>, Receiver<SocketEvent>) =
            unbounded();
        let (log_sender, log_receiver): (Sender<String>, Receiver<String>) = unbounded();
        let pid = Pid::from(std::process::id() as usize);

        event_read_handler(sender.clone());
        log_handler(log_dir, sender.clone(), log_sender.clone(), log_receiver)?;
        stats_handler(pid, sender.clone());
        socket_handler(socket_dir, sender, socket_receiver)?;

        if let TerminalEvent::Stats(a) = receiver.recv_timeout(Duration::from_secs(3))? {
            let p = predicates::str::contains("load");
            assert!(p.eval(&a));
        }

        log_sender.send("log".to_string())?;
        socket_sender.send(SocketEvent::WriteStdin("".to_string()))?;

        Ok(())
    }

    #[test]
    fn unit_attach_draw_ui() -> Result<()> {
        let name = "attach_draw_ui";
        test_utils::start_long_running_service(name)?;
        assert!(test_utils::check_app_is_running(name)?);

        let mut app = AttachTerminal::new(name.to_string());
        let stats_list = String::from("Waiting for stats.");

        let backend = TestBackend::new(16, 16);
        let mut terminal = Terminal::new(backend)?;

        terminal.draw(|f| ui(f, &mut app, &stats_list))?;

        test_utils::shutdown_long_running_service(name)?;
        test_utils::delete_app_folder(name)?;
        Ok(())
    }
}
