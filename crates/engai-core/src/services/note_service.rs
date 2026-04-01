use crate::db::NoteRepository;
use crate::error::{AppError, Result};
use crate::models::Note;

#[derive(Clone)]
pub struct NoteService {
    note_repo: NoteRepository,
}

impl NoteService {
    pub fn new(note_repo: NoteRepository) -> Self {
        Self { note_repo }
    }

    pub async fn list_notes(&self, target_type: &str, target_id: i64) -> Result<Vec<Note>> {
        Ok(self.note_repo.get_notes(target_type, target_id).await?)
    }

    pub async fn add_note(
        &self,
        target_type: &str,
        target_id: i64,
        content: &str,
    ) -> Result<Note> {
        let valid = ["word", "phrase", "reading"];
        if !valid.contains(&target_type) {
            return Err(AppError::ValidationError(format!(
                "Invalid target_type '{}'. Must be one of: {}",
                target_type,
                valid.join(", ")
            )));
        }
        if content.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Note content cannot be empty".into(),
            ));
        }
        Ok(self
            .note_repo
            .add_note(target_type, target_id, content)
            .await?)
    }

    pub async fn update_note(
        &self,
        id: i64,
        target_type: &str,
        target_id: i64,
        content: &str,
    ) -> Result<Note> {
        self.note_repo.delete_note(id).await?;
        Ok(self
            .note_repo
            .add_note(target_type, target_id, content)
            .await?)
    }

    pub async fn delete_note(&self, id: i64) -> Result<()> {
        self.note_repo.delete_note(id).await?;
        Ok(())
    }
}
