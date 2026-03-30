use crossterm::event::KeyCode;

use super::app::{App, ReviewItem};
use crate::state::AppState;
use engai_core::review::calculate_next_review;

pub async fn load_review(state: &AppState, app: &mut App) {
    app.review_loading = true;
    let db = &state.db;

    let words = db.get_today_review_words().await.unwrap_or_default();
    let phrases = db.get_today_review_phrases().await.unwrap_or_default();

    let mut items: Vec<ReviewItem> = words
        .into_iter()
        .map(|w| ReviewItem {
            target_type: "word".to_string(),
            id: w.id,
            display: w.word,
            meaning: w.meaning,
            familiarity: w.familiarity,
            interval: w.interval,
            ease_factor: w.ease_factor,
        })
        .collect();
    items.extend(phrases.into_iter().map(|p| ReviewItem {
        target_type: "phrase".to_string(),
        id: p.id,
        display: p.phrase,
        meaning: p.meaning,
        familiarity: p.familiarity,
        interval: p.interval,
        ease_factor: p.ease_factor,
    }));

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
        _ => {}
    }
}

async fn submit_review(app: &mut App, state: &AppState, quality: i32) {
    if app.review_index >= app.review_items.len() {
        return;
    }

    let quality = quality.clamp(0, 5);

    let item = app.review_items[app.review_index].clone();
    let result =
        calculate_next_review(quality, item.interval, item.ease_factor);

    let db = &state.db;
    let _ = db
        .add_review(&item.target_type, item.id, quality)
        .await;

    match item.target_type.as_str() {
        "word" => {
            let _ = db
                .update_word(
                    item.id,
                    None,
                    None,
                    None,
                    Some(result.familiarity),
                    Some(result.next_review),
                    Some(result.interval),
                    Some(result.ease_factor),
                )
                .await;
        }
        "phrase" => {
            let _ = db
                .update_phrase(
                    item.id,
                    None,
                    None,
                    Some(result.familiarity),
                    Some(result.next_review),
                    Some(result.interval),
                    Some(result.ease_factor),
                )
                .await;
        }
        _ => {}
    }

    app.review_index += 1;
    app.review_show_answer = false;

    if app.review_index >= app.review_items.len() {
        app.set_status(format!(
            "Review complete! {} items reviewed.",
            app.review_items.len()
        ));
    }
}
