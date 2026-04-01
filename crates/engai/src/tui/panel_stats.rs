use crossterm::event::KeyCode;

use super::app::{App, StatsData};
use crate::state::AppState;

pub async fn load_stats(state: &AppState, app: &mut App) {
    let word_count = state.word_repo.word_count().await.unwrap_or(0);
    let phrase_count = state.phrase_repo.phrase_count().await.unwrap_or(0);
    let pending = state.review_repo.pending_review_count().await.unwrap_or(0);
    let today = state.review_repo.review_count_today().await.unwrap_or(0);

    app.stats = Some(StatsData {
        word_count,
        phrase_count,
        pending_reviews: pending,
        reviewed_today: today,
    });
}

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    if code == KeyCode::Char('r') {
        load_stats(state, app).await;
        app.set_status("Stats refreshed");
    }
}
