use crate::universe::id::UniverseId;

#[derive(Debug)]
pub enum UniverseEvent {
    Empty,
    ChangeState(String),
    Shatter,                // damage the universe a bit
    Crash(UniverseId),      // the universe is damaged so bad, it crashed. UniverseId must be "Enemy" for this to apply.
    Heal(UniverseId),       // heals universe. UniverseId must be "brother" for this to apply.
    Ping(UniverseId),       // just sends a ping, expect a pong
    Pong(UniverseId),       // after a ping, this is the response
    Shutdown(UniverseId),   // same as crash but via force, UniverseId must be "brother" for this to apply.
}

impl From<&String> for UniverseEvent {
    fn from(s: &String) -> Self {
        match s.to_lowercase().as_str() {
            "ChangeState" => UniverseEvent::ChangeState(s.clone()),
            "Shatter" => UniverseEvent::Shatter,
            "Crash" => UniverseEvent::Crash(0),
            "Ping" => UniverseEvent::Ping(0),
            "Pong" => UniverseEvent::Pong(0),
            "Shutdown" => UniverseEvent::Shutdown(0),
            _ => UniverseEvent::Empty,
        }
    }
}