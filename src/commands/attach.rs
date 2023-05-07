use crate::{directory, process::process_pid_by_name, tail::Tail};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use crossbeam::{
    channel::{tick, unbounded, Receiver, Sender},
    select,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, LevelFilter};
use std::{
    io::{self, Stdout, Write},
    os::unix::net::UnixStream,
    path::Path,
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
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_logger::{init_logger, set_default_level, TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};

#[derive(Args)]
#[command(about = "Attach to an application.")]
pub struct AttachArgs {
    #[arg(help = "The application name")]
    pub name: String,
}

enum InputMode {
    Normal,
    Editing,
}

struct AttachedTerminal {
    name: String,
    logger_state: TuiWidgetState,
    input: Input,
    input_mode: InputMode,
}

impl AttachedTerminal {
    fn new(name: String) -> AttachedTerminal {
        AttachedTerminal {
            name,
            input: Input::default(),
            input_mode: InputMode::Normal,
            logger_state: TuiWidgetState::new(),
        }
    }
}

impl AttachArgs {
    pub fn run(self) -> Result<()> {
        init_logger(LevelFilter::Debug).unwrap();
        set_default_level(LevelFilter::Debug);

        let application_socket = application_socket(&self.name)?;
        let log_events = log_tail_events(&self.name)?;
        let pids = process_pid_by_name(&self.name)?;
        let stats_ticker = stats_update(pids[1])?;
        let ui_events = ui_events()?;
        let ticker = tick(Duration::from_millis(20));

        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);

        let mut terminal = Terminal::new(backend)?;

        let mut app = AttachedTerminal::new(self.name);

        let mut stats_list = String::from("Waiting for stats.");

        terminal.draw(|f| ui(f, &mut app, &stats_list))?;

        loop {
            select! {
                recv(ticker) -> _ => {
                    terminal.draw(|f| ui(f, &mut app, &stats_list))?;
                },
                recv(log_events)-> event => {
                    if let Ok(content) = event {
                        if content.is_empty() {
                            continue;
                        }

                        debug!("{content}");
                    }
                },
                recv(stats_ticker)-> event => {
                    if let Ok(content) = event {
                        stats_list = content;
                    }
                },
                recv(ui_events) -> event => {
                    if let Event::Key(key) = event? {
                        match app.input_mode {
                            InputMode::Normal => match key.code {
                                KeyCode::Char('e') => {
                                    app.input_mode = InputMode::Editing;
                                }
                                KeyCode::PageUp | KeyCode::Up => {
                                    app.logger_state.transition(&TuiWidgetEvent::PrevPageKey);
                                },
                                KeyCode::PageDown | KeyCode::Down => {
                                    app.logger_state.transition(&TuiWidgetEvent::NextPageKey);
                                },
                                KeyCode::Char('q') => {
                                    break
                                }
                                _ => {}
                            },
                            InputMode::Editing => match key.code {
                                KeyCode::Enter => {
                                    let input = app.input.value().to_string();

                                    if input.is_empty() {
                                        continue;
                                    }

                                    app.input.reset();

                                    let message = format!("{}\n", input);
                                    application_socket.send(message)?;
                                }
                                KeyCode::PageUp => {
                                    app.logger_state.transition(&TuiWidgetEvent::PrevPageKey);
                                },
                                KeyCode::PageDown => {
                                    app.logger_state.transition(&TuiWidgetEvent::NextPageKey);
                                },
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                _ => {
                                    app.input.handle_event(&Event::Key(key));
                                }
                            },
                        }
                    }
                }
            }
        }

        close_terminal(&mut terminal)
    }
}

fn close_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal::disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}

fn application_socket(name: &String) -> Result<Sender<String>> {
    let mut socket_dir = directory::application_dir_by_name(name)?;

    if !socket_dir.exists() {
        return Err(anyhow!("Application does not exist."));
    }

    socket_dir.push(name.clone() + ".sock");

    if !Path::new(&socket_dir).exists() {
        return Err(anyhow!("Socket file does not exist."));
    }

    let (sender, receiver): (Sender<String>, Receiver<String>) = unbounded();

    let mut stream = UnixStream::connect(socket_dir)?;

    thread::spawn(move || {
        for received in receiver {
            stream.write_all(received.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    });

    Ok(sender)
}

fn log_tail_events(name: &String) -> Result<Receiver<String>> {
    let mut log_dir = directory::application_dir_by_name(name)?;

    if !log_dir.exists() {
        return Err(anyhow!("Application does not exist."));
    }

    log_dir.push(name.clone() + ".log");

    let mut log = Tail::new(log_dir)?;

    let (sender, receiver) = unbounded();

    let lines = log.read_lines(200)?;

    for line in lines {
        sender.send(line)?;
    }

    thread::spawn(move || log.watch(&sender).unwrap());

    Ok(receiver)
}

fn stats_update(pid: Pid) -> Result<Receiver<String>> {
    let (sender, receiver) = unbounded();

    let ticker = tick(Duration::from_secs(2));

    let mut system = System::new();
    system.refresh_all();

    let cpu_count = system.physical_core_count().unwrap() as f32;

    thread::spawn(move || {
        while ticker.recv().is_ok() {
            system.refresh_all();

            let process = system
                .process(pid)
                .context("Error retrieving process information.")
                .unwrap();

            let memory = process.memory() as f64 / system.total_memory() as f64 * 100.0;

            let load = system.load_average();

            sender
                .send(format!(
                    "cpu: {:.2}% | mem: {:.2}% ({} Mb) | system load: {}, {}, {}",
                    process.cpu_usage() / cpu_count,
                    memory,
                    process.memory() / 1024 / 1024,
                    load.one,
                    load.five,
                    load.fifteen,
                ))
                .unwrap();
        }
    });

    Ok(receiver)
}

fn ui_events() -> Result<Receiver<Event>> {
    let (sender, receiver) = unbounded();

    thread::spawn(move || loop {
        sender.send(crossterm::event::read().unwrap()).unwrap()
    });

    Ok(receiver)
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut AttachedTerminal, stats_list: &String) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Max(3), Constraint::Max(3)].as_ref())
        .split(f.size());

    let help_message = match app.input_mode {
        InputMode::Normal => vec![
            Span::raw("Press "),
            Span::styled(
                "q",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::raw(" to exit, "),
            Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to start editing."),
        ],
        InputMode::Editing => vec![
            Span::raw("Press "),
            Span::styled(
                "Esc",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC),
            ),
            Span::raw(" to stop editing."),
        ],
    };

    let tui_logger = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(app.name.as_str())
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL),
        )
        .output_level(None)
        .output_timestamp(None)
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(&app.logger_state);

    f.render_widget(tui_logger, chunks[0]);

    let stats = Paragraph::new(stats_list.to_owned())
        .block(Block::default().borders(Borders::ALL).title("Stats"));

    f.render_widget(stats, chunks[1]);

    let tui_input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title(help_message));

    f.render_widget(tui_input, chunks[2]);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => f.set_cursor(
            chunks[2].x + app.input.visual_cursor() as u16 + 1,
            chunks[2].y + 1,
        ),
    }
}
