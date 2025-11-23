use crate::universe::universe_event::UniverseEvent;
use crate::universe::UniverseId;

#[derive(Debug)]
pub enum UniverseCommand {
    Start,
    Stop,
    InjectEvent(UniverseEvent),
    RequestState(),
    Shutdown,
    // New: Discovery
    MeetPeer { id: UniverseId, name: String },
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