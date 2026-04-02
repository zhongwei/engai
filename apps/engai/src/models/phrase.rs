use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Phrase {
    pub id: i64,
    pub phrase: String,
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
pub struct NewPhrase {
    pub phrase: String,
    pub meaning: Option<String>,
}
