use crossterm::event::KeyCode;

use super::app::{App, VocabDetail, VocabTab};
use crate::state::AppState;

pub async fn load_vocab(state: &AppState, app: &mut App) {
    let db = &state.db;
    match db.list_words(None, None, 200, 0).await {
        Ok(words) => app.words = words,
        Err(e) => app.set_status(format!("Failed to load words: {}", e)),
    }
    match db.list_phrases(None, None, 200, 0).await {
        Ok(phrases) => app.phrases = phrases,
        Err(e) => app.set_status(format!("Failed to load phrases: {}", e)),
    }
}

pub fn handle_tab(app: &mut App) {
    match app.vocab_tab {
        VocabTab::Words => {
            app.vocab_tab = VocabTab::Phrases;
            app.vocab_list_index = 0;
        }
        VocabTab::Phrases => {
            app.vocab_tab = VocabTab::Words;
            app.vocab_list_index = 0;
        }
    }
}

pub async fn handle_key(app: &mut App, _state: &AppState, code: KeyCode) {
    if app.vocab_detail.is_some() {
        return;
    }

    match code {
        KeyCode::Enter => {
            match app.vocab_tab {
                VocabTab::Words => {
                    if let Some(w) = app.words.get(app.vocab_list_index) {
                        app.vocab_detail = Some(VocabDetail::Word(w.clone()));
                    }
                }
                VocabTab::Phrases => {
                    if let Some(p) = app.phrases.get(app.vocab_list_index) {
                        app.vocab_detail = Some(VocabDetail::Phrase(p.clone()));
                    }
                }
            }
        }
        KeyCode::Up => {
            if app.vocab_list_index > 0 {
                app.vocab_list_index -= 1;
            }
        }
        KeyCode::Down => {
            let max = match app.vocab_tab {
                VocabTab::Words => app.words.len(),
                VocabTab::Phrases => app.phrases.len(),
            };
            if max > 0 && app.vocab_list_index < max - 1 {
                app.vocab_list_index += 1;
            }
        }
        _ => {}
    }
}
