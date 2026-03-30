use crossterm::event::KeyCode;

use super::app::{App, ReadingDetail};
use crate::state::AppState;

pub async fn load_readings(state: &AppState, app: &mut App) {
    match state.db.list_readings(100, 0).await {
        Ok(readings) => app.readings = readings,
        Err(e) => app.set_status(format!("Failed to load readings: {}", e)),
    }
}

pub async fn handle_key(app: &mut App, _state: &AppState, code: KeyCode) {
    if app.reading_detail.is_some() {
        return;
    }

    match code {
        KeyCode::Enter => {
            if let Some(reading) = app.readings.get(app.reading_list_index) {
                let reading_clone = reading.clone();
                app.reading_detail = Some(ReadingDetail {
                    reading: reading_clone,
                    analysis: None,
                });
            }
        }
        KeyCode::Up => {
            if app.reading_list_index > 0 {
                app.reading_list_index -= 1;
            }
        }
        KeyCode::Down => {
            if !app.readings.is_empty() && app.reading_list_index < app.readings.len() - 1 {
                app.reading_list_index += 1;
            }
        }
        _ => {}
    }
}
