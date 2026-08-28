use std::collections::HashMap;

use crate::{EntityId, Reference};

/// An in-memory index of references between entities.
///
/// The index does not own the entities themselves. It only provides
/// efficient access to references by their source or target.
#[derive(Debug, Default)]
pub struct ReferenceIndex {
    by_target: HashMap<EntityId, Vec<Reference>>,
    by_source: HashMap<EntityId, Vec<Reference>>,
}

impl ReferenceIndex {
    /// Creates an empty reference index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a reference to the index.
    pub fn add(&mut self, reference: Reference) {
        self.by_target
            .entry(reference.target)
            .or_default()
            .push(reference.clone());

        self.by_source
            .entry(reference.source)
            .or_default()
            .push(reference);
    }

    /// Returns all references pointing to an entity.
    pub fn by_target(&self, entity_id: EntityId) -> &[Reference] {
        self.by_target
            .get(&entity_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns all references originating from an entity.
    pub fn by_source(&self, entity_id: EntityId) -> &[Reference] {
        self.by_source
            .get(&entity_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Removes a reference from the index by its ID.
    pub fn remove(&mut self, reference_id: EntityId) -> bool {
        let mut removed = false;

        for references in self.by_target.values_mut() {
            let original_len = references.len();
            references.retain(|reference| reference.id != reference_id);

            if references.len() != original_len {
                removed = true;
            }
        }

        for references in self.by_source.values_mut() {
            references.retain(|reference| reference.id != reference_id);
        }

        self.by_target.retain(|_, references| !references.is_empty());
        self.by_source.retain(|_, references| !references.is_empty());

        removed
    }

    /// Returns the total number of indexed references.
    pub fn len(&self) -> usize {
        self.by_target.values().map(Vec::len).sum()
    }

    /// Returns whether the index contains no references.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}