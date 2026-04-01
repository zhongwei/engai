use crossterm::event::KeyCode;

use super::app::{App, VocabDetail, VocabTab};
use crate::state::AppState;

pub async fn load_vocab(state: &AppState, app: &mut App) {
    match state.word_repo.list_words(None, None, 200, 0).await {
        Ok(words) => app.words = words,
        Err(e) => app.set_status(format!("Failed to load words: {}", e)),
    }
    match state.phrase_repo.list_phrases(None, None, 200, 0).await {
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

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    if app.vocab_detail.is_some() {
        match code {
            KeyCode::Char('e') => {
                let detail = app.vocab_detail.clone().unwrap();
                app.set_status("Requesting AI explanation...");
                let ai = state.ai_client.clone();
                let prompt = state.prompt_engine.clone();
                let result = match &detail {
                    VocabDetail::Word(w) => ai.explain_word(&w.word, &prompt).await,
                    VocabDetail::Phrase(p) => ai.explain_phrase(&p.phrase, &prompt).await,
                };
                match result {
                    Ok(_explanation) => {
                        app.set_status("AI explanation received");
                    }
                    Err(e) => app.set_status(format!("AI error: {}", e)),
                }
            }
            _ => return,
        }
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
