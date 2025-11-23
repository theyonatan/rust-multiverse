use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use crate::supervisor::supervisor::SupervisorHandle;
use crate::universe::logger::LogMessage;
use crate::universe::universe_command::UniverseCommand;
use crate::universe::universe_event::UniverseEvent;

use std::io;
use std::io::Stdout;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

/// The state of the command menu
#[derive(PartialEq)]
pub enum AppState {
    MainMenu,
    ManagingUniverse(String), // Stores the name of the universe being managed
}

pub struct App {
    /// Text input field content
    pub input: String,
    /// Cursor position in the input field
    pub cursor_position: usize,
    /// Logs to display on the right panel
    pub logs: Vec<LogMessage>,
    /// Current navigation state (Main Menu vs Managing)
    pub state: AppState,
    /// Connection to the Logic Supervisor
    supervisor: SupervisorHandle,
    /// Receiver for logs coming from the Supervisor/Universes
    log_rx: UnboundedReceiver<LogMessage>,
    /// Sender (kept to clone into supervisor if needed, though we pass it on creation)
    _log_tx: UnboundedSender<LogMessage>,
}

impl App {
    pub fn new() -> App {
        let (log_tx, log_rx) = unbounded_channel();

        // Initialize the supervisor with the log channel
        let mut supervisor = SupervisorHandle::new(log_tx.clone());
        supervisor.start();

        App {
            input: String::new(),
            cursor_position: 0,
            logs: Vec::new(),
            state: AppState::MainMenu,
            supervisor,
            log_rx,
            _log_tx: log_tx,
        }
    }

    /// Main Function
    pub async fn run(&mut self) {
        let mut terminal = self.setup_terminal();

        self.main_loop(&mut terminal).await.unwrap_or_else(|_| eprintln!("Error starting main loop"));

        self.teardown_terminal(&mut terminal);
    }

    /// Setup terminal
    fn setup_terminal(&mut self) -> Terminal<CrosstermBackend<Stdout>>{
        enable_raw_mode().unwrap_or_else(|_| eprintln!("Error starting new terminal using ratatui!"));
        let mut stdout = io::stdout();
        let _ = execute!(stdout, EnterAlternateScreen);  // Remove EnableMouseCapture
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap_or_else(|_| panic!("Error creating terminal"));

        terminal
    }

    fn teardown_terminal(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
        disable_raw_mode().unwrap_or_else(|_| eprintln!("Error tearing down terminal using ratatui!"));
        let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        );
    }

    /// The Main Event Loop
    async fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        self.input.clear();
        self.cursor_position = 0;

        loop {
            // Draw UI
            terminal.draw(|f| crate::ui::ui::ui::<CrosstermBackend<Stdout>>(f, self))?;

            // Drain logs
            while let Ok(log) = self.log_rx.try_recv() {
                self.logs.push(log);
                if self.logs.len() > 200 {
                    self.logs.remove(0);
                }
            }

            // Always accept keyboard input
            if crossterm::event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind != KeyEventKind::Press { continue; }

                    match key.code {
                        KeyCode::Enter => {
                            if !self.input.trim().is_empty() {
                                self.submit_message().await;
                            }
                            self.input.clear();
                            self.cursor_position = 0;
                        }
                        KeyCode::Char(c) => {
                            self.input.insert(self.cursor_position, c);
                            self.cursor_position += 1;
                        }
                        KeyCode::Backspace => {
                            if self.cursor_position > 0 {
                                self.input.remove(self.cursor_position - 1);
                                self.cursor_position -= 1;
                            }
                        }
                        KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                        KeyCode::Right => {
                            if self.cursor_position < self.input.len() {
                                self.cursor_position += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Process the text entered by the user
    async fn submit_message(&mut self) {
        let cmd_line = self.input.trim().to_string();
        self.input.clear();
        self.reset_cursor();

        if cmd_line.is_empty() { return; }

        match &self.state {
            AppState::MainMenu => self.handle_main_menu_command(&cmd_line).await,
            AppState::ManagingUniverse(name) => self.handle_universe_command(&name.clone(), &cmd_line).await,
        }
    }

    async fn handle_main_menu_command(&mut self, input: &str) {
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts[0].to_lowercase().as_str() {
            "new" => {
                if parts.len() > 1 {
                    let name = parts[1..].join(" ");
                    self.supervisor.add_new_universe(name).await;
                } else {
                    self.add_system_log("Usage: new <Name>");
                }
            },
            "list" | "get" => {
                let names = self.supervisor.get_all_existing_universes();
                self.add_system_log("--- Active Universes ---");
                if names.is_empty() {
                    self.add_system_log("None");
                } else {
                    for name in names {
                        // Trigger state request for rich info
                        self.supervisor.send_universe_command(name.clone(), UniverseCommand::RequestState()).await;
                    }
                    self.add_system_log("(State requested — check logs)");
                }
            },
            "manage" | "command" => {
                if parts.len() > 1 {
                    let name = parts[1..].join(" ");
                    if self.supervisor.exists(&name) {
                        self.state = AppState::ManagingUniverse(name);
                    } else {
                        self.add_system_log(&format!("Universe '{}' not found.", name));
                    }
                } else {
                    self.add_system_log("Usage: manage <Name>");
                }
            },
            _ => self.add_system_log("Unknown command. Try: new, list, manage")
        }
    }

    async fn handle_universe_command(&mut self, name: &str, input: &str) {
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts[0].to_lowercase().as_str() {
            "back" => self.state = AppState::MainMenu,
            "shutdown" => {
                self.supervisor.send_universe_command(name.to_string(), UniverseCommand::Shutdown).await;
                // It will be removed asynchronously, go back to menu
                self.state = AppState::MainMenu;
            },
            "start" => self.supervisor.send_universe_command(name.to_string(), UniverseCommand::Start).await,
            "stop" => self.supervisor.send_universe_command(name.to_string(), UniverseCommand::Stop).await,
            "state" => {
                self.add_system_log(&format!("Requesting state from '{}'...", name));
                self.supervisor.send_universe_command(name.to_string(), UniverseCommand::RequestState()).await;
            }
            // Event Injection
            "shatter" => self.inject(name, UniverseEvent::Shatter(10)).await,
            "heal" => self.inject(name, UniverseEvent::Heal(50)).await,
            "ping" => self.inject(name, UniverseEvent::Ping).await,
            "crash" => self.inject(name, UniverseEvent::Crash(0)).await,
            _ => self.add_system_log("Unknown universe command. Try: start, stop, shatter, heal, back")
        }
    }

    async fn inject(&mut self, name: &str, event: UniverseEvent) {
        self.supervisor.send_universe_command(name.to_string(), UniverseCommand::InjectEvent(event)).await;
    }

    fn add_system_log(&mut self, msg: &str) {
        self.logs.push(LogMessage::Info(msg.to_string()));
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
}