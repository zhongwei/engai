use engai_core::db::Db;
use engai_core::markdown::{MarkdownWord, MarkdownPhrase};
use engai_core::review::calculate_next_review;

#[tokio::test]
async fn test_full_word_lifecycle() {
    let db = Db::new_in_memory().await.unwrap();

    let word = db
        .add_word("test", Some("/test/"), Some("a test word"))
        .await
        .unwrap();
    assert_eq!(word.word, "test");
    assert_eq!(word.phonetic.as_deref(), Some("/test/"));
    assert_eq!(word.meaning.as_deref(), Some("a test word"));

    let fetched = db.get_word("test").await.unwrap().unwrap();
    assert_eq!(fetched.id, word.id);
    assert_eq!(fetched.word, "test");
    assert_eq!(fetched.phonetic.as_deref(), Some("/test/"));
    assert_eq!(fetched.meaning.as_deref(), Some("a test word"));

    db.add_example("word", word.id, "this is a test", Some("unit test"))
        .await
        .unwrap();
    let examples = db.get_examples("word", word.id).await.unwrap();
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].sentence, "this is a test");
    assert_eq!(examples[0].source.as_deref(), Some("unit test"));

    db.add_note("word", word.id, "my note about test")
        .await
        .unwrap();
    let notes = db.get_notes("word", word.id).await.unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "my note about test");

    let review_result = calculate_next_review(5, 0, 2.5);
    assert!(review_result.familiarity > 0);
    assert!(review_result.interval > 0);

    let updated = db
        .update_word(
            word.id,
            None,
            None,
            None,
            Some(review_result.familiarity),
            Some(review_result.next_review),
            Some(review_result.interval),
            Some(review_result.ease_factor),
        )
        .await
        .unwrap()
        .unwrap();
    assert!(updated.familiarity > 0);
    assert!(updated.interval > 0);

    let review = db
        .add_review("word", word.id, 5)
        .await
        .unwrap();
    assert_eq!(review.quality, 5);
    assert_eq!(review.target_type, "word");
    assert_eq!(review.target_id, word.id);

    let reviews = db.get_reviews("word", word.id).await.unwrap();
    assert_eq!(reviews.len(), 1);

    let md_word = MarkdownWord {
        word: updated.word.clone(),
        phonetic: updated.phonetic.clone(),
        familiarity: updated.familiarity,
        interval: updated.interval,
        next_review: updated.next_review,
        meaning: updated.meaning.clone(),
        examples: examples.into_iter().map(|e| e.sentence).collect(),
        synonyms: vec![],
        ai_explanation: None,
        my_notes: notes.into_iter().map(|n| n.content).collect(),
        reviews: reviews
            .into_iter()
            .map(|r| format!("quality={}", r.quality))
            .collect(),
    };

    let md_string = md_word.to_markdown_string();
    let parsed = MarkdownWord::parse(&md_string).unwrap();
    assert_eq!(parsed.word, md_word.word);
    assert_eq!(parsed.familiarity, md_word.familiarity);

    let deleted = db.delete_word(word.id).await.unwrap();
    assert!(deleted);
    let gone = db.get_word("test").await.unwrap();
    assert!(gone.is_none());
}

#[tokio::test]
async fn test_full_phrase_lifecycle() {
    let db = Db::new_in_memory().await.unwrap();

    let phrase = db
        .add_phrase("give up", Some("to stop trying"))
        .await
        .unwrap();
    assert_eq!(phrase.phrase, "give up");
    assert_eq!(phrase.meaning.as_deref(), Some("to stop trying"));

    let fetched = db.get_phrase("give up").await.unwrap().unwrap();
    assert_eq!(fetched.id, phrase.id);
    assert_eq!(fetched.phrase, "give up");
    assert_eq!(fetched.meaning.as_deref(), Some("to stop trying"));

    db.add_example("phrase", phrase.id, "Don't give up on your dreams", None)
        .await
        .unwrap();
    let examples = db.get_examples("phrase", phrase.id).await.unwrap();
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].sentence, "Don't give up on your dreams");

    let review_result = calculate_next_review(4, 0, 2.5);
    let updated = db
        .update_phrase(
            phrase.id,
            None,
            None,
            Some(review_result.familiarity),
            Some(review_result.next_review),
            Some(review_result.interval),
            Some(review_result.ease_factor),
        )
        .await
        .unwrap()
        .unwrap();
    assert!(updated.familiarity > 0);
    assert!(updated.interval > 0);

    let md_phrase = MarkdownPhrase {
        phrase: updated.phrase.clone(),
        familiarity: updated.familiarity,
        interval: updated.interval,
        next_review: updated.next_review,
        meaning: updated.meaning.clone(),
        examples: examples.into_iter().map(|e| e.sentence).collect(),
        ai_explanation: None,
        my_notes: vec![],
        reviews: vec![],
    };

    let md_string = md_phrase.to_markdown_string();
    let parsed = MarkdownPhrase::parse(&md_string).unwrap();
    assert_eq!(parsed.phrase, md_phrase.phrase);
    assert_eq!(parsed.familiarity, md_phrase.familiarity);

    let deleted = db.delete_phrase(phrase.id).await.unwrap();
    assert!(deleted);
    let gone = db.get_phrase("give up").await.unwrap();
    assert!(gone.is_none());
}
