use crossterm::event::KeyCode;

use super::app::{App, ReviewItem};
use crate::state::AppState;

pub async fn load_review(state: &AppState, app: &mut App) {
    app.review_loading = true;

    let entries = state.review_service.get_today_reviews().await.unwrap_or_default();

    let items: Vec<ReviewItem> = entries
        .into_iter()
        .map(|e| ReviewItem {
            target_type: e.target_type,
            id: e.id,
            display: e.display,
            meaning: e.meaning,
            familiarity: e.familiarity,
            interval: e.interval,
            ease_factor: e.ease_factor,
        })
        .collect();

    app.review_items = items;
    app.review_index = 0;
    app.review_show_answer = false;
    app.review_loading = false;
}

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    if app.review_loading || app.review_items.is_empty() {
        return;
    }

    if app.review_index >= app.review_items.len() {
        return;
    }

    match code {
        KeyCode::Char(' ') => {
            app.review_show_answer = !app.review_show_answer;
        }
        KeyCode::Char('0') => submit_review(app, state, 0).await,
        KeyCode::Char('1') => submit_review(app, state, 1).await,
        KeyCode::Char('2') => submit_review(app, state, 2).await,
        KeyCode::Char('3') => submit_review(app, state, 3).await,
        KeyCode::Char('4') => submit_review(app, state, 4).await,
        KeyCode::Char('5') => submit_review(app, state, 5).await,
        KeyCode::Char('n') => {
            if app.review_index < app.review_items.len() {
                app.review_index += 1;
                app.review_show_answer = false;
                app.set_status("Skipped");
            }
        }
        _ => {}
    }
}

async fn submit_review(app: &mut App, state: &AppState, quality: i32) {
    if app.review_index >= app.review_items.len() {
        return;
    }

    let quality = quality.clamp(0, 5);

    let item = app.review_items[app.review_index].clone();
    let _ = state
        .review_service
        .submit_review(&item.target_type, item.id, quality)
        .await;

    app.review_index += 1;
    app.review_show_answer = false;

    if app.review_index >= app.review_items.len() {
        app.set_status(format!(
            "Review complete! {} items reviewed.",
            app.review_items.len()
        ));
    }
}
