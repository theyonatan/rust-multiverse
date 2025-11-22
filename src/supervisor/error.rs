use std::fmt;
use crate::universe::UniverseId;

#[derive(Debug)]
pub enum UniverseLookupError {
    IdNotFoundForName(String),
    UniverseNotFoundForId(UniverseId),
}

impl fmt::Display for UniverseLookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UniverseLookupError::IdNotFoundForName(name) => {
                write!(f, "id not found for name: {}", name)
            }

            UniverseLookupError::UniverseNotFoundForId(id) => {
                write!(f, "universe not found for id: {}", id)
            }
        }
    }
}

impl std::error::Error for UniverseLookupError {}