use crate::{directory, tail::Tail};
use anyhow::{anyhow, Result};
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
    thread,
    time::Duration,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use tui_input::{backend::crossterm as input_backend, Input};
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

        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);

        let mut terminal = Terminal::new(backend)?;

        let mut app = AttachedTerminal::new(self.name.clone());

        let ticker = tick(Duration::from_millis(30));
        let ui_events = ui_events()?;

        let log_events = match log_tail_events(&self.name) {
            Ok(log_events) => log_events,
            Err(err) => {
                close_terminal(&mut terminal)?;
                return Err(err);
            }
        };

        let application_socket = match application_socket(&self.name) {
            Ok(application_socket) => application_socket,
            Err(err) => {
                close_terminal(&mut terminal)?;
                return Err(err);
            }
        };

        terminal.draw(|f| ui(f, &mut app))?;

        loop {
            select! {
                recv(log_events)-> event => {
                    if let Ok(content) = event {
                        debug!("{content}");
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

                                    debug!(">> Sent: {}", &input);

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
                                    input_backend::to_input_request(&Event::Key(key)).map(|req| app.input.handle(req));
                                }
                            },
                        }
                    }
                },
                recv(ticker) -> _ => {
                    terminal.draw(|f| ui(f, &mut app))?;
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

fn ui_events() -> Result<Receiver<Event>> {
    let (sender, receiver) = unbounded();

    thread::spawn(move || loop {
        sender.send(crossterm::event::read().unwrap()).unwrap()
    });

    Ok(receiver)
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut AttachedTerminal) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default(),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing."),
            ],
            Style::default(),
        ),
    };

    let mut text = Text::from(Spans::from(msg));

    text.patch_style(style);

    let help_message = Paragraph::new(text);

    f.render_widget(help_message, chunks[0]);

    let logger = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(app.name.as_str())
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
        )
        .output_level(None)
        .output_timestamp(None)
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(&app.logger_state);

    f.render_widget(logger, chunks[1]);

    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(input, chunks[2]);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => {
            f.set_cursor(chunks[2].x + app.input.cursor() as u16 + 1, chunks[2].y + 1)
        }
    }
}
