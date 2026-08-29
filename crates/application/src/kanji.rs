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
        character: char,
    ) -> Result<KanjiEntry, sqlx::Error> {
        let kanji = KanjiEntry::new(character);

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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("failed to create test database");

        storage::initialize_database(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn create_and_get_kanji() {
        let pool = setup().await;
        let repository = KanjiRepository::new(&pool);
        let service = KanjiService::new(repository);

        let kanji = service
            .create('学')
            .await
            .expect("failed to create kanji");

        let loaded = service
            .get(kanji.id)
            .await
            .expect("failed to get kanji")
            .expect("kanji was not found");

        assert_eq!(loaded, kanji);
    }

    #[tokio::test]
    async fn get_by_character_finds_kanji() {
        let pool = setup().await;
        let repository = KanjiRepository::new(&pool);
        let service = KanjiService::new(repository);

        let created = service
            .create('学')
            .await
            .expect("failed to create kanji");

        let loaded = service
            .get_by_character('学')
            .await
            .expect("failed to find kanji")
            .expect("kanji was not found");

        assert_eq!(loaded, created);
    }

    #[tokio::test]
    async fn get_by_character_returns_none_for_unknown_character() {
        let pool = setup().await;
        let repository = KanjiRepository::new(&pool);
        let service = KanjiService::new(repository);

        let result = service
            .get_by_character('学')
            .await
            .expect("failed to query kanji");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn exists_tracks_kanji() {
        let pool = setup().await;
        let repository = KanjiRepository::new(&pool);
        let service = KanjiService::new(repository);

        let kanji = service
            .create('学')
            .await
            .expect("failed to create kanji");

        assert!(
            service
                .exists(kanji.id)
                .await
                .expect("failed to check existence")
        );
    }

    #[tokio::test]
    async fn unknown_kanji_does_not_exist() {
        let pool = setup().await;
        let repository = KanjiRepository::new(&pool);
        let service = KanjiService::new(repository);

        let id = EntityId::new();

        assert!(
            !service
                .exists(id)
                .await
                .expect("failed to check existence")
        );
    }

    #[tokio::test]
    async fn delete_removes_kanji() {
        let pool = setup().await;
        let repository = KanjiRepository::new(&pool);
        let service = KanjiService::new(repository);

        let kanji = service
            .create('学')
            .await
            .expect("failed to create kanji");

        assert!(
            service
                .delete(kanji.id)
                .await
                .expect("failed to delete kanji")
        );

        assert!(
            !service
                .exists(kanji.id)
                .await
                .expect("failed to check existence")
        );

        assert!(
            service
                .get(kanji.id)
                .await
                .expect("failed to get kanji")
                .is_none()
        );
    }

    #[tokio::test]
    async fn deleting_unknown_kanji_returns_false() {
        let pool = setup().await;
        let repository = KanjiRepository::new(&pool);
        let service = KanjiService::new(repository);

        let id = EntityId::new();

        assert!(
            !service
                .delete(id)
                .await
                .expect("failed to delete kanji")
        );
    }

    #[tokio::test]
    async fn create_returns_kanji_with_expected_defaults() {
        let pool = setup().await;
        let repository = KanjiRepository::new(&pool);
        let service = KanjiService::new(repository);

        let kanji = service
            .create('学')
            .await
            .expect("failed to create kanji");

        assert_eq!(kanji.character, '学');
        assert!(kanji.on_readings.is_empty());
        assert!(kanji.kun_readings.is_empty());
        assert!(kanji.meanings.is_empty());
        assert_eq!(kanji.stroke_count, None);
        assert_eq!(kanji.grade, None);
        assert_eq!(kanji.jlpt_level, None);
    }
}