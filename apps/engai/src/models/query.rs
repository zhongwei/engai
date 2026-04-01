use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordQuery {
    pub search: Option<String>,
    pub familiarity: Option<i32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhraseQuery {
    pub search: Option<String>,
    pub familiarity: Option<i32>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteQuery {
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub pagination: Option<Pagination>,
}
