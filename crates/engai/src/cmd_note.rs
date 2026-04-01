use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::{Db, NoteRepository};

#[derive(clap::Subcommand)]
pub enum NoteAction {
    Add {
        target_type: String,
        target_id: i64,
        content: Vec<String>,
    },
}

pub async fn run(action: NoteAction) -> Result<()> {
    match action {
        NoteAction::Add {
            target_type,
            target_id,
            content,
        } => {
            let valid = ["word", "phrase", "reading"];
            if !valid.contains(&target_type.as_str()) {
                anyhow::bail!(
                    "Invalid target_type '{}'. Must be one of: {}",
                    target_type,
                    valid.join(", ")
                );
            }
            let note_content = content.join(" ");
            if note_content.trim().is_empty() {
                anyhow::bail!("Note content cannot be empty");
            }

            let config = Config::load_global()?;
            let db = Db::new(&config.db_path()).await?;
            let note_repo = NoteRepository::new(db.pool().clone());
            let note = note_repo.add_note(&target_type, target_id, &note_content).await?;
            println!(
                "Note added (id: {}) for {} #{}",
                note.id, target_type, target_id
            );
        }
    }
    Ok(())
}
