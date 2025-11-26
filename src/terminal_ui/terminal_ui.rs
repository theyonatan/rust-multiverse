use ratatui::{
    backend::CrosstermBackend,
    text::{Span, Line},
    Terminal,
    widgets::{Block, Borders, Paragraph, ListState},
    layout::{Layout, Constraint, Direction},
};
use crossterm::{
    execute,
    event::{self, KeyCode, KeyEventKind, Event},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use tokio::sync::broadcast;
use crate::logging::subscribe;
use crate::supervisor::log_messages::*;
use crate::supervisor::user_supervisor::UserSupervisor;
use crate::universe::{UniverseCommand, UniverseEvent};

pub struct TerminalUI<'a> {
    supervisor: &'a mut UserSupervisor,
    input: String,
    logs: Vec<Vec<Span<'static>>>,
    log_receiver: broadcast::Receiver<Vec<Span<'static>>>,
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
            log_receiver: subscribe(),
            mode: UiMode::Main,
            log_state,
        }
    }

    pub async fn run(&mut self) {
        let mut terminal = self.init_terminal();

        loop {
            // draw terminal
            self.draw(&mut terminal);
            if let Some(cmd) = self.poll_input().unwrap() {
                if self.handle_input(cmd).await {
                    break;
                }
            }

            // collect logs for terminal
            while let Ok(line) = self.log_receiver.try_recv() {
                self.logs.push(line);
                if self.logs.len() > 1000 {
                    self.logs.remove(0);
                }
            }

            // process any incoming universe events (intents)
            self.supervisor.process_universe_events().await;
        }

        // end of program, shutdown all universes and clean terminal
        self.supervisor.shut_down_all().await;
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
                .split(f.area());

            // Left: commands + input
            let left = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(10), Constraint::Min(1), Constraint::Length(3)])
                .split(chunks[0]);

            let help = Paragraph::new(self.mode_text())
                .block(Block::default().borders(Borders::ALL).title("Commands"));
            f.render_widget(help, left[0]);

            let input = Paragraph::new(self.input.as_str())
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, left[2]);

            // Right: logs
            let log_lines: Vec<Line> = self.logs.iter()
                .map(|spans| Line::from(spans.clone()))
                .collect();

            // Calculate how much to scroll to show bottom
            let area = chunks[1];
            let inner_height = area.height.saturating_sub(2) as usize; // minus border
            let total_lines = log_lines.len();
            let scroll_offset = if total_lines > inner_height {
                (total_lines - inner_height) as u16
            } else {
                0
            };

            let logs_paragraph = Paragraph::new(log_lines)
                .block(Block::default().borders(Borders::ALL).title("Logs"))
                .wrap(ratatui::widgets::Wrap { trim: false })
                .scroll((scroll_offset, 0));

            f.render_widget(logs_paragraph, chunks[1]);
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
            UiMode::EventMenu { name } => self.handle_user_event(name.clone(), parts).await,
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
                        Log::info("Name cannot be empty");
                    } else {
                        self.supervisor.new_universe(name.clone()).await;
                    }
                } else {
                    Log::info("Usage: new <name>");
                }
            }
            "manage" => {
                if parts.len() > 1 {
                    let name = parts[1].trim().to_string();
                    Log::info(format!("Now managing '{}'", name));
                    self.mode = UiMode::Manage { name };
                } else {
                    Log::info("Usage: manage <name>");
                }
            }
            "list" => {
                let list = self.supervisor.get_list_universes();
                Log::info(format!("Universes: {:?}", list));
            }
            "shutdown" => return true,
            _ => Log::info(format!("Unknown command: '{}'", parts[0]))
        }
        false
    }

    async fn handle_manage(&mut self, name: String, p: Vec<&str>) -> bool {
        match p[0].to_lowercase().as_str() {
            "back" => self.mode = UiMode::Main,
            "resume" => {
                Log::info(format!("Resuming {}", name));
                self.supervisor.supervisor.send_universe_command(name.clone(), UniverseCommand::Start).await;
            }
            "pause" => {
                Log::info(format!("Pausing {}", name));
                self.supervisor.supervisor.send_universe_command(name.clone(), UniverseCommand::Stop).await;
            }
            "event" => self.mode = UiMode::EventMenu { name },
            "collapse" => {
                Log::info(format!("Collapsing {}", name));
                self.supervisor.supervisor.send_universe_command(name.clone(), UniverseCommand::Shutdown).await;
            }
            _ => Log::info("Unknown manage command"),
        }
        false
    }

    async fn handle_user_event(&mut self, name: String, p: Vec<&str>) -> bool {
        // get user handle for logs color
        let universe_color = self.supervisor.supervisor.existing_universes
            .get(&self.supervisor.supervisor.universes_via_name[&name])
            .unwrap().color;

        match p[0].to_lowercase().as_str() {
            "back" => self.mode = UiMode::Manage { name },
            "shatter" => {
                let strength = 20;

                Log::user_action("You", "shattered", &name, universe_color);

                self.supervisor.supervisor.send_universe_command(
                    name.clone(),
                    UniverseCommand::InjectEvent(UniverseEvent::Shatter(strength)),
                ).await;
            }
            "heal" => {
                let strength = 20;

                Log::user_action("You", "healed", &name, universe_color);

                self.supervisor.supervisor.send_universe_command(
                    name.clone(),
                    UniverseCommand::InjectEvent(UniverseEvent::Heal(strength)),
                ).await;
            }
            "crash" => {
                Log::user_action("You", "CRASHED", &name, universe_color);

                self.supervisor.supervisor.send_universe_command(
                    name.clone(),
                    UniverseCommand::InjectEvent(UniverseEvent::Crash),
                ).await;
            }
            _ => Log::info("Unknown event command"),
        }
        false
    }
}
