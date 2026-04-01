use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct Reading {
    pub id: i64,
    pub title: Option<String>,
    pub content: String,
    pub source: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEntry {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEntry {
    pub target_type: String,
    pub id: i64,
    pub display: String,
    pub meaning: Option<String>,
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub next_review: NaiveDateTime,
    pub interval: i32,
    pub ease_factor: f64,
    pub familiarity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsData {
    pub word_count: i64,
    pub phrase_count: i64,
    pub pending_reviews: i64,
    pub reviewed_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub role: String,
    pub content: String,
}
