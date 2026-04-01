use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatEntry {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}
