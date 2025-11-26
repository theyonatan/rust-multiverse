use crate::universe::relationship::Relationship;
use crate::universe::universe_event::UniverseEvent;
use crate::universe::UniverseId;

#[derive(Debug)]
pub enum UniverseCommand {
    Start, // Resume
    Stop, // Pause
    InjectEvent(UniverseEvent), // TODO: Supervisor sends an event to the Universe which he unpacks and deals with.
    RequestState(), // TODO: Supervisor sends a request to get an event, the Universe somehow returns a response.
    Shutdown, // Shuts down entirely
    SetRelationship(UniverseId, Relationship),
    UnknownCommand,
}