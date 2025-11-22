use crate::universe::universe_event::UniverseEvent;

#[derive(Debug)]
pub enum UniverseCommand {
    Start, // Resume
    Stop, // Pause
    InjectEvent(UniverseEvent), // TODO: Supervisor sends an event to the Universe which he unpacks and deals with.
    RequestState(), // TODO: Supervisor sends a request to get an event, the Universe somehow returns a response.
    Shutdown, // Shuts down entirely
    UnknownCommand,
}

impl From<&String> for UniverseCommand {
    fn from(s: &String) -> Self {
        match s.to_lowercase().as_str() {
            "start" => UniverseCommand::Start,
            "stop" => UniverseCommand::Stop,
            "do_event" | "event" | "do" => UniverseCommand::InjectEvent(UniverseEvent::Empty),
            "request_state" | "state" | "request" => UniverseCommand::RequestState(),
            "shutdown" => UniverseCommand::Shutdown,
            _ => UniverseCommand::UnknownCommand,
        }
    }
}