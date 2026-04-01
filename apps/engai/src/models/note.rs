use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
}
