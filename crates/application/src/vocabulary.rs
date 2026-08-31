use domain::{
    EntityId,
    PartOfSpeech,
    VocabularyEntry,
};
use storage::repositories::VocabularyRepository;

pub struct VocabularyService<'a> {
    repository: VocabularyRepository<'a>,
}

impl<'a> VocabularyService<'a> {
    pub fn new(repository: VocabularyRepository<'a>) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        written_forms: Vec<String>,
        readings: Vec<String>,
        meanings: Vec<String>,
        parts_of_speech: Vec<PartOfSpeech>,
        source: Option<String>,
    ) -> Result<VocabularyEntry, sqlx::Error> {
        let mut vocabulary = VocabularyEntry::new();

        vocabulary.written_forms = written_forms;
        vocabulary.readings = readings;
        vocabulary.meanings = meanings;
        vocabulary.parts_of_speech = parts_of_speech;
        vocabulary.source = source;

        self.repository.insert(&vocabulary).await?;

        Ok(vocabulary)
    }

    pub async fn get(
        &self,
        id: EntityId,
    ) -> Result<Option<VocabularyEntry>, sqlx::Error> {
        self.repository.get(id).await
    }

    pub async fn exists(
        &self,
        id: EntityId,
    ) -> Result<bool, sqlx::Error> {
        self.repository.exists(id).await
    }

    pub async fn delete(
        &self,
        id: EntityId,
    ) -> Result<bool, sqlx::Error> {
        self.repository.delete(id).await
    }
}