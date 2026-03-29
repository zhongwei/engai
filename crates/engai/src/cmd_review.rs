use std::io::{self, Write};

use anyhow::Result;

use engai_core::config::Config;
use engai_core::db::Db;
use engai_core::review::calculate_next_review;

pub async fn run(show_all: bool) -> Result<()> {
    let config = Config::load_global()?;
    let db = Db::new(&config.db_path()).await?;

    if show_all {
        let words = db.list_words(None, None, 1000, 0).await?;
        let phrases = db.list_phrases(None, None, 1000, 0).await?;

        println!("=== Words ({}) ===", words.len());
        for w in &words {
            let meaning = w.meaning.as_deref().unwrap_or("N/A");
            let review = w
                .next_review
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "N/A".to_string());
            println!(
                "  {} | familiarity: {} | next_review: {} | {}",
                w.word, w.familiarity, review, meaning
            );
        }

        println!("\n=== Phrases ({}) ===", phrases.len());
        for p in &phrases {
            let meaning = p.meaning.as_deref().unwrap_or("N/A");
            let review = p
                .next_review
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "N/A".to_string());
            println!(
                "  {} | familiarity: {} | next_review: {} | {}",
                p.phrase, p.familiarity, review, meaning
            );
        }

        return Ok(());
    }

    let words = db.get_today_review_words().await?;
    let phrases = db.get_today_review_phrases().await?;

    let total = words.len() + phrases.len();
    if total == 0 {
        println!("No items to review today. Great job!");
        return Ok(());
    }

    println!("Today's review: {} words, {} phrases", words.len(), phrases.len());

    for w in &words {
        let meaning = w.meaning.as_deref().unwrap_or("(no meaning set)");
        println!("\nWord: {}", w.word);
        println!("Meaning: {}", meaning);
        print!("Rate quality (0-5): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let quality: i32 = input.trim().parse().unwrap_or(0);
        let quality = quality.clamp(0, 5);

        let result = calculate_next_review(quality, w.interval, w.ease_factor);
        db.update_word(
            w.id,
            None,
            None,
            None,
            Some(result.familiarity),
            Some(result.next_review),
            Some(result.interval),
            Some(result.ease_factor),
        )
        .await?;
        db.add_review("word", w.id, quality).await?;
    }

    for p in &phrases {
        let meaning = p.meaning.as_deref().unwrap_or("(no meaning set)");
        println!("\nPhrase: {}", p.phrase);
        println!("Meaning: {}", meaning);
        print!("Rate quality (0-5): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let quality: i32 = input.trim().parse().unwrap_or(0);
        let quality = quality.clamp(0, 5);

        let result = calculate_next_review(quality, p.interval, p.ease_factor);
        db.update_phrase(
            p.id,
            None,
            None,
            Some(result.familiarity),
            Some(result.next_review),
            Some(result.interval),
            Some(result.ease_factor),
        )
        .await?;
        db.add_review("phrase", p.id, quality).await?;
    }

    println!("\nReview complete! {} items reviewed.", total);
    Ok(())
}
