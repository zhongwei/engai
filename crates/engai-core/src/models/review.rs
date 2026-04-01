use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Review {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub quality: i32,
    pub reviewed_at: NaiveDateTime,
}
