use std::fmt;
use crate::universe::UniverseId;

/// Errors that can occur when looking up universes
#[derive(Debug, Clone)]
pub enum UniverseLookupError {
    IdNotFoundForName(String),
    UniverseNotFoundForId(UniverseId),
    UniverseAlreadyExists(String),
    UniverseIsDead(UniverseId),
}

impl fmt::Display for UniverseLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UniverseLookupError::IdNotFoundForName(name) => {
                write!(f, "Universe name '{}' not found", name)
            }
            UniverseLookupError::UniverseNotFoundForId(id) => {
                write!(f, "Universe with ID '{}' not found", id)
            }
            UniverseLookupError::UniverseAlreadyExists(name) => {
                write!(f, "Universe '{}' already exists", name)
            }
            UniverseLookupError::UniverseIsDead(id) => {
                write!(f, "Universe '{}' has COLLAPSED and is no more", id)
            }
        }
    }
}

impl std::error::Error for UniverseLookupError {}