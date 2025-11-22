use std::sync::atomic::{AtomicUsize, Ordering};

pub type UniverseId = usize;

static NEXT_UNIVERSE_ID: AtomicUsize = AtomicUsize::new(1);
pub fn new_universe_id() -> UniverseId {
    NEXT_UNIVERSE_ID.fetch_add(1, Ordering::Relaxed)
}
