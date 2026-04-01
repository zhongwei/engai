use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Example {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub sentence: String,
    pub source: Option<String>,
}
