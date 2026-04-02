use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Word {
    pub id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
    pub familiarity: i32,
    pub next_review: Option<NaiveDateTime>,
    pub interval: i32,
    pub ease_factor: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct NewWord {
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
}
