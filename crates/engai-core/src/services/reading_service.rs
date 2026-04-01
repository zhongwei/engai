use crate::db::ReadingRepository;
use crate::error::{AppError, Result};
use crate::models::Reading;
use crate::services::AiService;

#[derive(Clone)]
pub struct ReadingService {
    reading_repo: ReadingRepository,
    ai: AiService,
}

impl ReadingService {
    pub fn new(reading_repo: ReadingRepository, ai: AiService) -> Self {
        Self { reading_repo, ai }
    }

    pub async fn list_readings(&self, limit: i64, offset: i64) -> Result<Vec<Reading>> {
        Ok(self.reading_repo.list_readings(limit, offset).await?)
    }

    pub async fn add_reading(
        &self,
        title: Option<&str>,
        content: &str,
        source: Option<&str>,
    ) -> Result<Reading> {
        if content.trim().is_empty() {
            return Err(AppError::ValidationError(
                "reading content cannot be empty".into(),
            ));
        }
        Ok(self
            .reading_repo
            .add_reading(title, content, source)
            .await?)
    }

    pub async fn get_reading(&self, id: i64) -> Result<Reading> {
        self.reading_repo
            .get_reading(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("reading {} not found", id)))
    }

    pub async fn delete_reading(&self, id: i64) -> Result<()> {
        self.reading_repo.delete_reading(id).await?;
        Ok(())
    }

    pub async fn analyze_reading(&self, content: &str) -> Result<String> {
        self.ai
            .analyze_reading(content)
            .await
            .map_err(|e| AppError::AiError(e.to_string()))
    }
}
