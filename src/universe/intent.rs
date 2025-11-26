use crate::universe::id::UniverseId;

// events that conclude the result of a universe action, sent to himself.
#[derive(Debug, Clone)]
pub enum UniverseIntent {
    Attack { target: UniverseId, damage: i32 },
    Heal   { target: UniverseId, amount: i32 },
    Dead   { target: UniverseId, },
}