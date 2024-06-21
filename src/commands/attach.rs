use std::{
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use crate::{
    application::Application,
    logger::{LogSystem, Logger},
    service::{InitSystem, Service, StatusOutput},
};

use anyhow::Result;
use clap::Args;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, LevelFilter};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use sysinfo::{Pid, System};
use tui_input::Input;
use tui_logger::{TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};

#[derive(Args)]
#[command(about = "Attach to an application.")]
pub struct AttachArgs {
    #[arg(help = "Service name.")]
    pub name: String,
}

impl AttachArgs {
    pub fn run(self) -> Result<()> {
        let application = Application::from(&self.name);
        application.exists()?;

        if !cfg!(test) {
            tui_logger::init_logger(LevelFilter::Debug).expect("Failed to initialize logger");
            tui_logger::set_default_level(LevelFilter::Debug);
        }

        let (terminal_sender, terminal_receiver) = mpsc::channel();
        let (stdin_sender, stdin_receiver) = mpsc::channel();

        let mut attach = AttachTerminal::new(application.name.clone());
        attach.history = application.read_command_history()?;

        let init_system = Service::get(Some(&self.name));
        let pid = match init_system.status(false)? {
            StatusOutput::Pretty(status) => Some(status.pid),
            StatusOutput::Raw(_) => None,
        };

        // Handles sending crossterm events to the main loop.
        crossterm_event_handler(terminal_sender.clone());

        // Handles notifying the main loop to redraw when there is a new log.
        log_handler(application.name.clone(), terminal_sender.clone());

        match pid {
            Some(pid) => {
                // Handles sending service stats like CPU and memory usage to the main loop.
                stats_handler(pid, terminal_sender.clone());

                // Handles sending commands to the service and writing to the history.
                stdin_handler(application, stdin_receiver, terminal_sender)?;
            }
            None => {
                attach.stats = String::from("PID not found, unable to query stats");
            }
        }

        if cfg!(test) {
            return Ok(());
        }

        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut history_pos: i16 = -1;

        while attach.running {
            match terminal_receiver.recv()? {
                TerminalEvent::Stats(stats) => {
                    attach.stats = stats;
                }
                TerminalEvent::CrosstermEvent(event) => match event {
                    Event::Key(key) => {
                        match key.code {
                            KeyCode::Enter => {
                                let content = attach.input.value().to_string();

                                if content.trim().is_empty() {
                                    continue;
                                }

                                attach.input.reset();

                                attach.history.insert(0, content.clone());
                                history_pos = -1;
                                stdin_sender.send(content)?;
                                attach.logger.transition(TuiWidgetEvent::EscapeKey);
                            }
                            KeyCode::PageUp => {
                                attach.logger.transition(TuiWidgetEvent::PrevPageKey);
                            }
                            KeyCode::PageDown => {
                                attach.logger.transition(TuiWidgetEvent::NextPageKey);
                            }
                            KeyCode::Up => {
                                if history_pos < attach.history.len() as i16 - 1 {
                                    history_pos += 1;

                                    attach.input = attach.input.with_value(
                                        attach.history[history_pos as usize].to_string(),
                                    );
                                }
                            }
                            KeyCode::Down => {
                                if history_pos > 0 && history_pos < attach.history.len() as i16 {
                                    history_pos -= 1;

                                    attach.input = attach.input.with_value(
                                        attach.history[history_pos as usize].to_string(),
                                    );
                                }
                            }
                            KeyCode::Esc => break,
                            _ => {
                                tui_input::backend::crossterm::EventHandler::handle_event(
                                    &mut attach.input,
                                    &event,
                                );
                            }
                        };
                    }
                    Event::Mouse(mouse) => {
                        match mouse.kind {
                            MouseEventKind::ScrollDown => {
                                attach.logger.transition(TuiWidgetEvent::NextPageKey);
                            }
                            MouseEventKind::ScrollUp => {
                                attach.logger.transition(TuiWidgetEvent::PrevPageKey);
                            }
                            _ => {}
                        };
                    }
                    _ => {}
                },
                _ => {}
            };

            terminal.draw(|f| ui(f, &mut attach))?;
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

struct AttachTerminal {
    application_name: String,
    running: bool,

    stats: String,
    history: Vec<String>,

    logger: TuiWidgetState,
    input: Input,
}

impl AttachTerminal {
    fn new(application_name: String) -> AttachTerminal {
        AttachTerminal {
            application_name,
            logger: TuiWidgetState::new(),
            stats: String::from("Waiting for stats."),
            running: true,
            input: Input::default(),
            history: Vec::new(),
        }
    }
}

enum TerminalEvent {
    CrosstermEvent(Event),
    Stats(String),
    Redraw,
}

fn crossterm_event_handler(terminal_sender: Sender<TerminalEvent>) {
    thread::spawn(move || loop {
        terminal_sender
            .send(TerminalEvent::CrosstermEvent(
                crossterm::event::read().expect("Error reading a crossterm event"),
            ))
            .expect("Failed to send crossterm event to terminal");
    });
}

fn log_handler(application_name: String, terminal_sender: Sender<TerminalEvent>) {
    thread::spawn(move || {
        let logger = Logger::get(application_name);

        let process = logger.follow().expect("Failed to start logger process");
        let stdout = process.stdout.expect("Failed to capture stdout");

        let reader = BufReader::new(stdout);

        for line in reader.lines().flatten() {
            debug!("{line}");

            terminal_sender
                .send(TerminalEvent::Redraw)
                .expect("Failed to send log to terminal");
        }
    });
}

fn stats_handler(pid: u32, terminal_sender: Sender<TerminalEvent>) {
    let mut system = System::new();
    system.refresh_processes();
    system.refresh_memory();

    let cpu_count = system
        .physical_core_count()
        .expect("Failed to get cpu count") as f32;

    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(1500));

        system.refresh_processes();
        system.refresh_memory();

        let pid = Pid::from_u32(pid);

        let process = match system.process(pid) {
            Some(process) => process,
            None => {
                terminal_sender
                    .send(TerminalEvent::Stats(String::from(
                        "Error retrieving process information.",
                    )))
                    .expect("Failed to send stats error to terminal");
                break;
            }
        };

        let memory = process.memory() as f64 / system.total_memory() as f64 * 100.0;
        let load = sysinfo::System::load_average();

        let stats = format!(
            "cpu: {:.2}% | mem: {:.2}% ({} Mb) | system load: {}, {}, {}",
            process.cpu_usage() / cpu_count,
            memory,
            process.memory() / 1024 / 1024,
            load.one,
            load.five,
            load.fifteen,
        );

        terminal_sender
            .send(TerminalEvent::Stats(stats))
            .expect("Failed to send stats to terminal");
    });
}

fn stdin_handler(
    application: Application,
    stdin_receiver: Receiver<String>,
    terminal_sender: Sender<TerminalEvent>,
) -> Result<()> {
    let stdin = application.stdin_path()?;
    let mut stdin = OpenOptions::new().write(true).open(stdin)?;

    let history = application.history_path()?;
    let mut history = OpenOptions::new().append(true).open(history)?;

    thread::spawn(move || {
        for mut received in stdin_receiver {
            debug!("> {received}");

            terminal_sender
                .send(TerminalEvent::Redraw)
                .expect("Failed to send stdin to terminal");

            received += "\n";
            let bytes = received.as_bytes();

            stdin.write_all(bytes).expect("Failed to write to stdin");

            history
                .write_all(bytes)
                .expect("Failed to write to history");
        }
    });

    Ok(())
}

fn ui(f: &mut Frame, attach: &mut AttachTerminal) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Max(3), Constraint::Max(3)].as_ref())
        .split(f.size());

    let text_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::LightCyan);

    let application_name = Span::styled(attach.application_name.as_str(), text_style);

    let tui_logger = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(application_name)
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL),
        )
        .output_level(None)
        .output_timestamp(None)
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(&attach.logger);

    f.render_widget(tui_logger, chunks[0]);

    let stats_text = Span::styled("Stats", text_style);

    let stats = Paragraph::new(attach.stats.to_owned())
        .block(Block::default().borders(Borders::ALL).title(stats_text));

    f.render_widget(stats, chunks[1]);

    let input_text = Span::styled("Input", text_style);

    let tui_input = Paragraph::new(attach.input.value())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title(input_text));

    f.render_widget(tui_input, chunks[2]);

    f.set_cursor(
        chunks[2].x + attach.input.visual_cursor() as u16 + 1,
        chunks[2].y + 1,
    )
}
