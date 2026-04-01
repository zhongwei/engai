use crossterm::event::KeyCode;

use super::app::{App, StatsData};
use crate::state::AppState;

pub async fn load_stats(state: &AppState, app: &mut App) {
    match state.stats_service.get_stats().await {
        Ok(s) => {
            app.stats = Some(StatsData {
                word_count: s.word_count,
                phrase_count: s.phrase_count,
                pending_reviews: s.pending_reviews,
                reviewed_today: s.reviewed_today,
            });
        }
        Err(_) => {
            app.stats = None;
        }
    }
}

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    if code == KeyCode::Char('r') {
        load_stats(state, app).await;
        app.set_status("Stats refreshed");
    }
}
