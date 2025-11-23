use crate::universe::id::UniverseId;
use crossterm::style::Color; // Add this dependency

#[derive(Debug, Clone)]
pub enum UniverseEvent {
    Empty,
    ChangeState,
    Shatter(u32),        // Now carries damage
    Heal(u32),
    PeerDied,
    Crash(UniverseId),
    Ping,
    Pong,
    Shutdown,
}


#[derive(Debug)]
pub enum UniverseOutboundEvent {
    // Modified to carry visual info
    Log { name: String, color: Color, message: String },
    MessagePeer { target_id: UniverseId, event: UniverseEvent },
    BroadcastDeath(UniverseId),
    BroadcastPeerDied(UniverseId), // name of the dead one
}

// ... rest is same

impl From<&String> for UniverseEvent {
    fn from(s: &String) -> Self {
        match s.to_lowercase().as_str() {
            "changestate" => UniverseEvent::ChangeState,
            "shatter" => UniverseEvent::Shatter(10),
            "crash" => UniverseEvent::Crash(0),
            "ping" => UniverseEvent::Ping,
            "pong" => UniverseEvent::Pong,
            "shutdown" => UniverseEvent::Shutdown,
            _ => UniverseEvent::Empty,
        }
    }
}