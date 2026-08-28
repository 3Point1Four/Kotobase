pub mod grammar;
pub mod id;
pub mod kanji;
pub mod reference;
pub mod relationship;
pub mod sentence;
pub mod vocabulary;
pub mod reference_index;
pub mod relationship_index;

pub use grammar::GrammarPattern;
pub use id::EntityId;
pub use kanji::KanjiEntry;
pub use reference::{EntityType, Reference, ReferenceLocation};
pub use relationship::{Relationship, RelationshipKind, RelationshipMetadata};
pub use sentence::{GrammarMatch, Sentence, Token};
pub use vocabulary::{PartOfSpeech, VocabularyEntry};
pub use reference_index::ReferenceIndex;
pub use relationship_index::RelationshipIndex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
fn relationship_index_can_query_by_source() {
    let source = EntityId::new();
    let target = EntityId::new();

    let relationship =
        Relationship::new(source, target, RelationshipKind::Uses);

    let mut index = RelationshipIndex::new();
    index.add(relationship.clone());

    let relationships = index.by_source(source);

    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0].target, target);
    assert_eq!(relationships[0].kind, RelationshipKind::Uses);
}

#[test]
fn relationship_index_can_query_by_target() {
    let source = EntityId::new();
    let target = EntityId::new();

    let relationship =
        Relationship::new(source, target, RelationshipKind::References);

    let mut index = RelationshipIndex::new();
    index.add(relationship.clone());

    let relationships = index.by_target(target);

    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0].source, source);
    assert_eq!(relationships[0].kind, RelationshipKind::References);
}

#[test]
fn relationship_index_can_filter_by_kind() {
    let source = EntityId::new();
    let target_a = EntityId::new();
    let target_b = EntityId::new();

    let uses = Relationship::new(
        source,
        target_a,
        RelationshipKind::Uses,
    );

    let explains = Relationship::new(
        source,
        target_b,
        RelationshipKind::Explains,
    );

    let mut index = RelationshipIndex::new();

    index.add(uses);
    index.add(explains);

    let results = index.by_source_kind(
        source,
        RelationshipKind::Uses,
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].target, target_a);
}

#[test]
fn relationship_index_supports_bidirectional_queries() {
    let source = EntityId::new();
    let target = EntityId::new();

    let relationship =
        Relationship::new(source, target, RelationshipKind::ExampleOf);

    let mut index = RelationshipIndex::new();
    index.add(relationship);

    assert_eq!(index.by_source(source).len(), 1);
    assert_eq!(index.by_target(target).len(), 1);
}

#[test]
fn relationship_index_can_remove_relationship() {
    let source = EntityId::new();
    let target = EntityId::new();

    let relationship =
        Relationship::new(source, target, RelationshipKind::Contains);

    let mut index = RelationshipIndex::new();
    index.add(relationship);

    assert_eq!(index.len(), 1);

    assert!(index.remove(
        source,
        target,
        RelationshipKind::Contains,
    ));

    assert_eq!(index.len(), 0);
    assert!(index.by_source(source).is_empty());
    assert!(index.by_target(target).is_empty());
}

#[test]
fn relationship_index_returns_empty_for_unknown_entities() {
    let index = RelationshipIndex::new();
    let unknown = EntityId::new();

    assert!(index.by_source(unknown).is_empty());
    assert!(index.by_target(unknown).is_empty());
    assert!(index.is_empty());
}

    #[test]
fn reference_index_can_query_by_target() {
    let source = EntityId::new();
    let target = EntityId::new();

    let reference = Reference::new(
        source,
        EntityType::Note,
        target,
        EntityType::Vocabulary,
        ReferenceLocation::CharacterRange {
            start: 0,
            end: 2,
        },
    );

    let mut index = ReferenceIndex::new();
    index.add(reference.clone());

    let references = index.by_target(target);

    assert_eq!(references.len(), 1);
    assert_eq!(references[0].id, reference.id);
}

#[test]
fn reference_index_can_query_by_source() {
    let source = EntityId::new();
    let first_target = EntityId::new();
    let second_target = EntityId::new();

    let first = Reference::new(
        source,
        EntityType::Note,
        first_target,
        EntityType::Vocabulary,
        ReferenceLocation::CharacterRange {
            start: 0,
            end: 2,
        },
    );

    let second = Reference::new(
        source,
        EntityType::Note,
        second_target,
        EntityType::Kanji,
        ReferenceLocation::CharacterRange {
            start: 3,
            end: 4,
        },
    );

    let mut index = ReferenceIndex::new();

    index.add(first);
    index.add(second);

    assert_eq!(index.by_source(source).len(), 2);
}

#[test]
fn reference_index_supports_bidirectional_queries() {
    let note = EntityId::new();
    let vocabulary = EntityId::new();

    let reference = Reference::new(
        note,
        EntityType::Note,
        vocabulary,
        EntityType::Vocabulary,
        ReferenceLocation::CharacterRange {
            start: 5,
            end: 7,
        },
    );

    let mut index = ReferenceIndex::new();
    index.add(reference.clone());

    assert_eq!(index.by_source(note).len(), 1);
    assert_eq!(index.by_target(vocabulary).len(), 1);

    assert_eq!(index.by_source(note)[0].id, reference.id);
    assert_eq!(index.by_target(vocabulary)[0].id, reference.id);
}

#[test]
fn reference_index_can_remove_a_reference() {
    let source = EntityId::new();
    let target = EntityId::new();

    let reference = Reference::new(
        source,
        EntityType::Note,
        target,
        EntityType::Vocabulary,
        ReferenceLocation::CharacterRange {
            start: 0,
            end: 2,
        },
    );

    let reference_id = reference.id;

    let mut index = ReferenceIndex::new();
    index.add(reference);

    assert_eq!(index.len(), 1);

    assert!(index.remove(reference_id));

    assert_eq!(index.len(), 0);
    assert!(index.by_source(source).is_empty());
    assert!(index.by_target(target).is_empty());
}

#[test]
fn reference_index_returns_empty_for_unknown_entities() {
    let index = ReferenceIndex::new();
    let unknown = EntityId::new();

    assert!(index.by_source(unknown).is_empty());
    assert!(index.by_target(unknown).is_empty());
    assert!(index.is_empty());
}

    #[test]
fn relationship_has_no_metadata_by_default() {
    let source = EntityId::new();
    let target = EntityId::new();

    let relationship =
        Relationship::new(source, target, RelationshipKind::References);

    assert_eq!(relationship.metadata.label, None);
    assert_eq!(relationship.metadata.context, None);
}

#[test]
fn relationship_can_store_metadata() {
    let source = EntityId::new();
    let target = EntityId::new();

    let mut metadata = RelationshipMetadata::new();
    metadata.label = Some("Important vocabulary".to_string());
    metadata.context = Some("Words I regularly confuse".to_string());

    let relationship = Relationship::with_metadata(
        source,
        target,
        RelationshipKind::References,
        metadata,
    );

    assert_eq!(
        relationship.metadata.label.as_deref(),
        Some("Important vocabulary")
    );

    assert_eq!(
        relationship.metadata.context.as_deref(),
        Some("Words I regularly confuse")
    );
}

#[test]
fn relationship_supports_semantic_relationship_kinds() {
    let source = EntityId::new();
    let target = EntityId::new();

    let kinds = [
        RelationshipKind::Contains,
        RelationshipKind::References,
        RelationshipKind::Uses,
        RelationshipKind::ExampleOf,
        RelationshipKind::Explains,
        RelationshipKind::Translates,
        RelationshipKind::DerivedFrom,
        RelationshipKind::RelatedTo,
    ];

    for kind in kinds {
        let relationship = Relationship::new(source, target, kind);

        assert_eq!(relationship.source, source);
        assert_eq!(relationship.target, target);
        assert_eq!(relationship.kind, kind);
    }
}

        #[test]
    fn reference_identifies_source_and_target_types() {
        let note = EntityId::new();
        let vocabulary = EntityId::new();

        let reference = Reference::new(
            note,
            EntityType::Note,
            vocabulary,
            EntityType::Vocabulary,
            ReferenceLocation::CharacterRange {
                start: 10,
                end: 12,
            },
        );

        assert_eq!(reference.source, note);
        assert_eq!(reference.source_type, EntityType::Note);

        assert_eq!(reference.target, vocabulary);
        assert_eq!(reference.target_type, EntityType::Vocabulary);
    }

    #[test]
    fn reference_can_point_to_a_file_location() {
        let file = EntityId::new();
        let vocabulary = EntityId::new();

        let reference = Reference::new(
            file,
            EntityType::File,
            vocabulary,
            EntityType::Vocabulary,
            ReferenceLocation::FileLocation {
                page: Some(42),
                start: Some(1200),
                end: Some(1202),
            },
        );

        assert_eq!(
            reference.location,
            ReferenceLocation::FileLocation {
                page: Some(42),
                start: Some(1200),
                end: Some(1202),
            }
        );
    }

    #[test]
    fn reference_has_unique_identity() {
        let source = EntityId::new();
        let target = EntityId::new();

        let first = Reference::new(
            source,
            EntityType::Note,
            target,
            EntityType::Vocabulary,
            ReferenceLocation::CharacterRange {
                start: 0,
                end: 2,
            },
        );

        let second = Reference::new(
            source,
            EntityType::Note,
            target,
            EntityType::Vocabulary,
            ReferenceLocation::CharacterRange {
                start: 0,
                end: 2,
            },
        );

        assert_ne!(first.id, second.id);
    }

        #[test]
    fn token_can_reference_vocabulary() {
        let vocabulary = VocabularyEntry::new();

        let mut token = Token::new("学校", 0, 2);
        token.vocabulary_id = Some(vocabulary.id);

        assert_eq!(token.vocabulary_id, Some(vocabulary.id));
    }

    #[test]
    fn grammar_match_can_reference_grammar_pattern() {
        let grammar = GrammarPattern::new("〜ている");

        let grammar_match = GrammarMatch::new(grammar.id, 3, 7);

        assert_eq!(grammar_match.grammar_id, grammar.id);
        assert_eq!(grammar_match.start, 3);
        assert_eq!(grammar_match.end, 7);
    }

    #[test]
    fn sentence_can_contain_tokens_and_grammar_matches() {
        let vocabulary = VocabularyEntry::new();
        let grammar = GrammarPattern::new("〜ている");

        let mut sentence = Sentence::new("日本語を勉強しています。");

        let mut token = Token::new("勉強", 3, 5);
        token.vocabulary_id = Some(vocabulary.id);

        sentence.tokens.push(token);
        sentence
            .grammar_matches
            .push(GrammarMatch::new(grammar.id, 7, 11));

        assert_eq!(sentence.tokens.len(), 1);
        assert_eq!(sentence.grammar_matches.len(), 1);

        assert_eq!(
            sentence.tokens[0].vocabulary_id,
            Some(vocabulary.id)
        );

        assert_eq!(
            sentence.grammar_matches[0].grammar_id,
            grammar.id
        );
    }

    #[test]
    fn kanji_has_its_own_identity() {
        let kanji = KanjiEntry::new('学');

        assert_eq!(kanji.character, '学');
        assert_ne!(kanji.id, EntityId::new());
    }

    #[test]
    fn entity_ids_are_unique() {
        let first = EntityId::new();
        let second = EntityId::new();

        assert_ne!(first, second);
    }

    #[test]
    fn relationship_preserves_entities_and_kind() {
        let source = EntityId::new();
        let target = EntityId::new();

        let relationship =
            Relationship::new(source, target, RelationshipKind::References);

        assert_eq!(relationship.source, source);
        assert_eq!(relationship.target, target);
        assert_eq!(relationship.kind, RelationshipKind::References);
    }

#[test]
fn reference_preserves_location() {
    let source = EntityId::new();
    let target = EntityId::new();

    let reference = Reference::new(
        source,
        EntityType::Note,
        target,
        EntityType::Vocabulary,
        ReferenceLocation::CharacterRange {
            start: 5,
            end: 7,
        },
    );

    assert_eq!(reference.source, source);
    assert_eq!(reference.target, target);
    assert_eq!(
        reference.location,
        ReferenceLocation::CharacterRange {
            start: 5,
            end: 7,
        }
    );
}
}