use chrono::{Duration, Local};

pub struct ReviewResult {
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
    pub next_review: chrono::NaiveDateTime,
}

pub fn calculate_next_review(
    quality: i32,
    current_interval: i32,
    current_ease_factor: f64,
) -> ReviewResult {
    let q = quality.clamp(0, 5);

    if q <= 2 {
        return ReviewResult {
            familiarity: 0,
            interval: 1,
            ease_factor: (current_ease_factor - 0.2).max(1.3),
            next_review: Local::now().naive_local() + Duration::days(1),
        };
    }

    if q == 3 {
        return ReviewResult {
            familiarity: 1,
            interval: current_interval,
            ease_factor: current_ease_factor,
            next_review: Local::now().naive_local() + Duration::days(current_interval as i64),
        };
    }

    let ease_adjust = match q {
        4 => 0.15,
        _ => 0.20,
    };
    let new_ef = (current_ease_factor + ease_adjust).min(3.0);
    let new_interval = if current_interval == 0 {
        1
    } else {
        (current_interval as f64 * new_ef).round() as i32
    };

    ReviewResult {
        familiarity: (q - 2).min(5),
        interval: new_interval,
        ease_factor: new_ef,
        next_review: Local::now().naive_local() + Duration::days(new_interval as i64),
    }
}
