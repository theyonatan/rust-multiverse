use crate::universe::relationship::Relationship;
use crate::universe::universe_event::UniverseEvent;
use crate::universe::UniverseId;

#[derive(Debug)]
pub enum UniverseCommand {
    Start, // Resume
    Stop, // Pause
    InjectEvent(UniverseEvent),
    Shutdown, // Shuts down entirely
    SetRelationship(UniverseId, Relationship),
}