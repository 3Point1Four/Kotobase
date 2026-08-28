use std::collections::HashMap;

use crate::{EntityId, Relationship, RelationshipKind};

/// An in-memory index of semantic relationships between entities.
///
/// The index does not own the entities themselves. It only provides
/// efficient access to relationships by source, target, or kind.
#[derive(Debug, Default)]
pub struct RelationshipIndex {
    by_source: HashMap<EntityId, Vec<Relationship>>,
    by_target: HashMap<EntityId, Vec<Relationship>>,
}

impl RelationshipIndex {
    /// Creates an empty relationship index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a relationship to the index.
    pub fn add(&mut self, relationship: Relationship) {
        self.by_source
            .entry(relationship.source)
            .or_default()
            .push(relationship.clone());

        self.by_target
            .entry(relationship.target)
            .or_default()
            .push(relationship);
    }

    /// Returns all relationships originating from an entity.
    pub fn by_source(&self, entity_id: EntityId) -> &[Relationship] {
        self.by_source
            .get(&entity_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns all relationships pointing to an entity.
    pub fn by_target(&self, entity_id: EntityId) -> &[Relationship] {
        self.by_target
            .get(&entity_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns relationships of a specific kind originating from an entity.
    pub fn by_source_kind(
        &self,
        entity_id: EntityId,
        kind: RelationshipKind,
    ) -> Vec<&Relationship> {
        self.by_source(entity_id)
            .iter()
            .filter(|relationship| relationship.kind == kind)
            .collect()
    }

    /// Returns relationships of a specific kind pointing to an entity.
    pub fn by_target_kind(
        &self,
        entity_id: EntityId,
        kind: RelationshipKind,
    ) -> Vec<&Relationship> {
        self.by_target(entity_id)
            .iter()
            .filter(|relationship| relationship.kind == kind)
            .collect()
    }

    /// Removes a relationship from the index by its source, target and kind.
    ///
    /// Returns `true` if a relationship was removed.
    pub fn remove(
        &mut self,
        source: EntityId,
        target: EntityId,
        kind: RelationshipKind,
    ) -> bool {
        let mut removed = false;

        if let Some(relationships) = self.by_source.get_mut(&source) {
            let original_len = relationships.len();

            relationships.retain(|relationship| {
                !(relationship.source == source
                    && relationship.target == target
                    && relationship.kind == kind)
            });

            if relationships.len() != original_len {
                removed = true;
            }
        }

        if let Some(relationships) = self.by_target.get_mut(&target) {
            relationships.retain(|relationship| {
                !(relationship.source == source
                    && relationship.target == target
                    && relationship.kind == kind)
            });
        }

        self.by_source.retain(|_, relationships| !relationships.is_empty());
        self.by_target.retain(|_, relationships| !relationships.is_empty());

        removed
    }

    /// Returns the total number of indexed relationships.
    pub fn len(&self) -> usize {
        self.by_source.values().map(Vec::len).sum()
    }

    /// Returns whether the index contains no relationships.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}