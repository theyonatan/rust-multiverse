use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Paragraph, List, ListItem, ListState},
    layout::{Layout, Constraint, Direction},
};
use crossterm::{
    execute,
    event::{self, KeyCode, KeyEventKind, Event},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};

use crate::supervisor::supervisor::UserSupervisor;
use crate::universe::{UniverseCommand, UniverseEvent};

pub struct TerminalUI<'a> {
    supervisor: &'a mut UserSupervisor,
    input: String,
    logs: Vec<String>,
    mode: UiMode,
    log_state: ListState,
}

#[derive(Clone)]
enum UiMode {
    Main,
    Manage { name: String },
    EventMenu { name: String },
}

impl<'a> TerminalUI<'a> {
    pub fn new(supervisor: &'a mut UserSupervisor) -> Self {
        let mut log_state = ListState::default();
        log_state.select(Some(0)); // will be updated dynamically
        Self {
            supervisor,
            input: String::new(),
            logs: vec![],
            mode: UiMode::Main,
            log_state,
        }
    }

    pub async fn run(&mut self) {
        let mut terminal = self.init_terminal();
        loop {
            self.draw(&mut terminal);
            if let Some(cmd) = self.poll_input().unwrap() {
                if self.handle_input(cmd).await {
                    break;
                }
            }
        }
        self.shutdown_terminal(&mut terminal);
    }

    fn init_terminal(&self) -> Terminal<CrosstermBackend<Stdout>> {
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        Terminal::new(CrosstermBackend::new(stdout)).unwrap()
    }

    fn shutdown_terminal(&self, t: &mut Terminal<CrosstermBackend<Stdout>>) {
        disable_raw_mode().unwrap();
        execute!(t.backend_mut(), LeaveAlternateScreen).unwrap();
    }

    fn draw(&mut self, t: &mut Terminal<CrosstermBackend<Stdout>>) {
        let _ = t.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());

            // Left: commands + input
            let left = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Min(1), Constraint::Length(3)])
                .split(chunks[0]);

            let help = Paragraph::new(self.mode_text())
                .block(Block::default().borders(Borders::ALL).title("Commands"));
            f.render_widget(help, left[0]);

            let input = Paragraph::new(self.input.as_str())
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, left[2]);

            // Right: logs
            let log_items: Vec<ListItem> = self
                .logs
                .iter()
                .map(|l| ListItem::new(l.clone()))
                .collect();

            let mut list = List::new(log_items)
                .block(Block::default().borders(Borders::ALL).title("Logs"));

            // Keep selection at the bottom (newest entry)
            if !self.logs.is_empty() {
                self.log_state.select(Some(self.logs.len().saturating_sub(1)));
            }

            f.render_stateful_widget(list, chunks[1], &mut self.log_state);
        });
    }

    fn poll_input(&mut self) -> io::Result<Option<String>> {
        if !event::poll(std::time::Duration::from_millis(16))? {
            return Ok(None);
        }

        match event::read()? {
            Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                KeyCode::Enter => {
                    let cmd = self.input.clone();
                    self.input.clear();
                    Ok(Some(cmd))
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                    Ok(None)
                }
                KeyCode::Backspace => {
                    self.input.pop();
                    Ok(None)
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    fn mode_text(&self) -> String {
        match &self.mode {
            UiMode::Main => "new <name>\nlist\nmanage <name>\nshutdown".into(),
            UiMode::Manage { name } => format!(
                "Managing '{}':\nresume\npause\nevent\nstate\ncollapse\nback",
                name
            ),
            UiMode::EventMenu { name } => format!(
                "Event on '{}':\nshatter\ncrash\nheal\nping\npong\nback",
                name
            ),
        }
    }

    async fn handle_input(&mut self, line: String) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        let mode = &self.mode;

        match mode {
            UiMode::Main => self.handle_main(parts).await,
            UiMode::Manage { name } => self.handle_manage(name.clone(), parts).await,
            UiMode::EventMenu { name } => self.handle_event(name.clone(), parts).await,
        }
    }

    async fn handle_main(&mut self, p: Vec<&str>) -> bool {
        let input = p.join(" "); // reconstruct full line
        let parts: Vec<&str> = input.splitn(2, ' ').collect(); // only split once

        match parts[0].to_lowercase().as_str() {
            "new" => {
                if parts.len() > 1 {
                    let name = parts[1].trim().to_string();
                    if name.is_empty() {
                        self.log("Name cannot be empty".into());
                    } else {
                        self.supervisor.new_universe(name.clone()).await;
                        self.log(format!("Created universe '{}'", name));
                    }
                } else {
                    self.log("Usage: new <name>".into());
                }
            }
            "manage" => {
                if parts.len() > 1 {
                    let name = parts[1].trim().to_string();
                    self.log(format!("Now managing '{}'", name));
                    self.mode = UiMode::Manage { name };
                } else {
                    self.log("Usage: manage <name>".into());
                }
            }
            "list" => {
                let list = self.supervisor.handle_list_universes();
                self.log(format!("Universes: {:?}", list));
            }
            "shutdown" => return true,
            _ => self.log("Unknown command. Type 'new', 'list', 'manage <name>', or 'shutdown'".into()),
        }
        false
    }

    async fn handle_manage(&mut self, name: String, p: Vec<&str>) -> bool {
        match p[0].to_lowercase().as_str() {
            "back" => self.mode = UiMode::Main,
            "resume" => {
                self.supervisor.supervisor.send_universe_command(name.clone(), UniverseCommand::Start).await;
            }
            "pause" => {
                self.supervisor.supervisor.send_universe_command(name.clone(), UniverseCommand::Stop).await;
            }
            "event" => self.mode = UiMode::EventMenu { name },
            "state" => {
                self.supervisor.supervisor.send_universe_command(name.clone(), UniverseCommand::RequestState()).await;
            }
            "collapse" => {
                self.supervisor.supervisor.send_universe_command(name.clone(), UniverseCommand::Shutdown).await;
            }
            _ => self.log("Unknown manage command".into()),
        }
        false
    }

    async fn handle_event(&mut self, name: String, p: Vec<&str>) -> bool {
        match p[0].to_lowercase().as_str() {
            "back" => self.mode = UiMode::Manage { name },
            "shatter" => {
                self.supervisor.supervisor.send_universe_command(
                    name.clone(),
                    UniverseCommand::InjectEvent(UniverseEvent::Shatter),
                ).await;
            }
            "crash" => {
                self.supervisor.supervisor.send_universe_command(
                    name.clone(),
                    UniverseCommand::InjectEvent(UniverseEvent::Crash(0)),
                ).await;
            }
            "heal" => {
                self.supervisor.supervisor.send_universe_command(
                    name.clone(),
                    UniverseCommand::InjectEvent(UniverseEvent::Heal(0)),
                ).await;
            }
            "ping" => {
                self.supervisor.supervisor.send_universe_command(
                    name.clone(),
                    UniverseCommand::InjectEvent(UniverseEvent::Ping(0)),
                ).await;
            }
            "pong" => {
                self.supervisor.supervisor.send_universe_command(
                    name.clone(),
                    UniverseCommand::InjectEvent(UniverseEvent::Pong(0)),
                ).await;
            }
            _ => self.log("Unknown event command".into()),
        }
        false
    }

    fn log(&mut self, msg: String) {
        self.logs.push(msg);
        if self.logs.len() > 300 {
            self.logs.remove(0);
        }
    }
}
