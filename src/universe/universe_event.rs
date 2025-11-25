use crate::universe::id::UniverseId;

#[derive(Debug)]
pub enum UniverseEvent {
    Empty,
    ChangeState(String),
    Shatter(i32),                // damage the universe a bit
    Crash,      // the universe is damaged so bad, it crashed. UniverseId must be "Enemy" for this to apply.
    Heal(i32),       // heals universe. UniverseId must be "brother" for this to apply.
    Shutdown,   // same as crash but via force, UniverseId must be "brother" for this to apply.
}