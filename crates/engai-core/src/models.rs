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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Example {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub sentence: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Review {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub quality: i32,
    pub reviewed_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reading {
    pub id: i64,
    pub title: Option<String>,
    pub content: String,
    pub source: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWord {
    pub word: String,
    pub phonetic: Option<String>,
    pub meaning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPhrase {
    pub phrase: String,
    pub meaning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatEntry {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}
