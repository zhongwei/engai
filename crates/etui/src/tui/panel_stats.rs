use crossterm::event::KeyCode;

use super::app::{App, StatsData};
use crate::api::ApiClient;

pub async fn load_stats(client: &ApiClient, app: &mut App) {
    match client.get_stats().await {
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

pub async fn handle_key(app: &mut App, client: &ApiClient, code: KeyCode) {
    if code == KeyCode::Char('r') {
        load_stats(client, app).await;
        app.set_status("Stats refreshed");
    }
}
