pub mod grammar;
pub mod id;
pub mod kanji;
pub mod reference;
pub mod relationship;
pub mod sentence;
pub mod vocabulary;

pub use grammar::GrammarPattern;
pub use id::EntityId;
pub use kanji::KanjiEntry;
pub use reference::{Reference, ReferenceLocation};
pub use relationship::{Relationship, RelationshipKind};
pub use sentence::{GrammarMatch, Sentence, Token};
pub use vocabulary::{PartOfSpeech, VocabularyEntry};

#[cfg(test)]
mod tests {
    use super::*;

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

        let reference = Reference {
            id: EntityId::new(),
            source,
            target,
            location: ReferenceLocation::CharacterRange {
                start: 5,
                end: 7,
            },
        };

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