use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct WordSummary {
    pub id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<NaiveDateTime>,
    pub meaning: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct PhraseSummary {
    pub id: i64,
    pub phrase: String,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<NaiveDateTime>,
    pub meaning: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct ReviewInfo {
    pub reviewed_at: NaiveDateTime,
    pub quality: i32,
}
