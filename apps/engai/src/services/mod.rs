pub mod ai_service;
pub mod chat_service;
pub mod note_service;
pub mod phrase_service;
pub mod reading_service;
pub mod review_service;
pub mod stats_service;
pub mod sync_service;
pub mod word_service;

pub use ai_service::AiService;
pub use chat_service::ChatService;
pub use note_service::NoteService;
pub use phrase_service::PhraseService;
pub use reading_service::ReadingService;
pub use review_service::{ReviewEntry, ReviewResult, ReviewService, ReviewStats};
pub use stats_service::{StatsData, StatsService};
pub use sync_service::SyncService;
pub use word_service::WordService;

use sqlx::SqlitePool;

use crate::db::{
    ChatRepository, ExampleRepository, NoteRepository, PhraseRepository, ReadingRepository,
    ReviewRepository, WordRepository,
};

pub struct Services {
    pub word: WordService,
    pub phrase: PhraseService,
    pub review: ReviewService,
    pub reading: ReadingService,
    pub note: NoteService,
    pub chat: ChatService,
    pub stats: StatsService,
}

impl Services {
    pub fn new(pool: SqlitePool, ai: AiService) -> Self {
        let word_repo = WordRepository::new(pool.clone());
        let phrase_repo = PhraseRepository::new(pool.clone());
        let example_repo = ExampleRepository::new(pool.clone());
        let review_repo = ReviewRepository::new(pool.clone());
        let reading_repo = ReadingRepository::new(pool.clone());
        let note_repo = NoteRepository::new(pool.clone());
        let chat_repo = ChatRepository::new(pool);

        Self {
            word: WordService::new(
                word_repo.clone(),
                example_repo.clone(),
                ai.clone(),
            ),
            phrase: PhraseService::new(
                phrase_repo.clone(),
                example_repo,
                ai.clone(),
            ),
            review: ReviewService::new(word_repo.clone(), phrase_repo.clone(), review_repo.clone()),
            reading: ReadingService::new(reading_repo, ai.clone()),
            note: NoteService::new(note_repo),
            chat: ChatService::new(chat_repo, ai),
            stats: StatsService::new(word_repo, phrase_repo, review_repo),
        }
    }
}
