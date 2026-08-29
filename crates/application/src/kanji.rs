use domain::{EntityId, KanjiEntry};
use storage::repositories::KanjiRepository;

pub struct KanjiService<'a> {
    repository: KanjiRepository<'a>,
}

impl<'a> KanjiService<'a> {
    pub fn new(repository: KanjiRepository<'a>) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        kanji: KanjiEntry,
    ) -> Result<KanjiEntry, sqlx::Error> {
        self.repository.insert(&kanji).await?;

        Ok(kanji)
    }

    pub async fn get(
        &self,
        id: EntityId,
    ) -> Result<Option<KanjiEntry>, sqlx::Error> {
        self.repository.get(id).await
    }

   pub async fn get_by_character(
    &self,
    character: char,
    ) -> Result<Option<KanjiEntry>, sqlx::Error> {
    self.repository.get_by_character(character).await
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