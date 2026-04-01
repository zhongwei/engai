use crossterm::event::KeyCode;

use super::app::{App, ReadingDetail};
use crate::api::ApiClient;

pub async fn load_readings(client: &ApiClient, app: &mut App) {
    match client.list_readings(100, 0).await {
        Ok(readings) => app.readings = readings,
        Err(e) => app.set_status(format!("Failed to load readings: {}", e)),
    }
}

pub async fn handle_key(app: &mut App, _client: &ApiClient, code: KeyCode) {
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
