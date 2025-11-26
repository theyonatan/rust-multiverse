use crate::universe::id::UniverseId;

#[derive(Debug)]
pub enum UniverseEvent {
    Shatter(i32),                // damage the universe a bit
    Heal(i32),       // heals universe. UniverseId must be "brother" for this to apply.
    Crash,      // the universe is damaged so bad, it crashed. UniverseId must be "Enemy" for this to apply.
    UniverseCollapsed(UniverseId),
}