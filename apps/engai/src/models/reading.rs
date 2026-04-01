use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reading {
    pub id: i64,
    pub title: Option<String>,
    pub content: String,
    pub source: Option<String>,
    pub created_at: NaiveDateTime,
}
