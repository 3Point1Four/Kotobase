use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A stable unique identifier for any entity in Kotobase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Creates a new random entity ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an entity ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}