use engai_core::db::Db;

#[tokio::test]
async fn test_db_init_and_word_crud() {
    let db = Db::new_in_memory().await.unwrap();

    let word = db.add_word("hello", Some("/həˈloʊ/"), Some("a greeting")).await.unwrap();
    assert_eq!(word.word, "hello");
    assert_eq!(word.phonetic.as_deref(), Some("/həˈloʊ/"));
    assert_eq!(word.meaning.as_deref(), Some("a greeting"));
    assert_eq!(word.familiarity, 0);

    let fetched = db.get_word("hello").await.unwrap().unwrap();
    assert_eq!(fetched.id, word.id);

    let by_id = db.get_word_by_id(word.id).await.unwrap().unwrap();
    assert_eq!(by_id.word, "hello");

    let db2 = db.add_word("world", None, None).await.unwrap();

    let all = db.list_words(None, None, 100, 0).await.unwrap();
    assert_eq!(all.len(), 2);

    let searched = db.list_words(Some("ell"), None, 100, 0).await.unwrap();
    assert_eq!(searched.len(), 1);
    assert_eq!(searched[0].word, "hello");

    let familiar = db.list_words(None, Some(0), 100, 0).await.unwrap();
    assert_eq!(familiar.len(), 2);

    let limited = db.list_words(None, None, 1, 0).await.unwrap();
    assert_eq!(limited.len(), 1);

    let deleted = db.delete_word(word.id).await.unwrap();
    assert!(deleted);

    let gone = db.get_word("hello").await.unwrap();
    assert!(gone.is_none());

    let delete_again = db.delete_word(word.id).await.unwrap();
    assert!(!delete_again);

    let count = db.word_count().await.unwrap();
    assert_eq!(count, 1);

    drop(db2);
}

#[tokio::test]
async fn test_phrase_crud() {
    let db = Db::new_in_memory().await.unwrap();

    let phrase = db
        .add_phrase("how are you", Some("a common greeting"))
        .await
        .unwrap();
    assert_eq!(phrase.phrase, "how are you");
    assert_eq!(phrase.meaning.as_deref(), Some("a common greeting"));

    let fetched = db.get_phrase("how are you").await.unwrap().unwrap();
    assert_eq!(fetched.id, phrase.id);

    let by_id = db.get_phrase_by_id(phrase.id).await.unwrap().unwrap();
    assert_eq!(by_id.phrase, "how are you");

    db.add_phrase("good morning", None).await.unwrap();

    let all = db.list_phrases(None, None, 100, 0).await.unwrap();
    assert_eq!(all.len(), 2);

    let searched = db.list_phrases(Some("morning"), None, 100, 0).await.unwrap();
    assert_eq!(searched.len(), 1);
    assert_eq!(searched[0].phrase, "good morning");

    let count = db.phrase_count().await.unwrap();
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_examples_crud() {
    let db = Db::new_in_memory().await.unwrap();

    let word = db.add_word("run", Some("/rʌn/"), Some("to move fast")).await.unwrap();

    let ex = db
        .add_example("word", word.id, "I run every morning.", Some("self"))
        .await
        .unwrap();
    assert_eq!(ex.sentence, "I run every morning.");
    assert_eq!(ex.target_type, "word");
    assert_eq!(ex.target_id, word.id);

    db.add_example("word", word.id, "She runs a company.", None)
        .await
        .unwrap();

    let examples = db.get_examples("word", word.id).await.unwrap();
    assert_eq!(examples.len(), 2);

    let deleted = db.delete_examples("word", word.id).await.unwrap();
    assert_eq!(deleted, 2);

    let after = db.get_examples("word", word.id).await.unwrap();
    assert!(after.is_empty());
}

#[tokio::test]
async fn test_reading_crud() {
    let db = Db::new_in_memory().await.unwrap();

    let reading = db
        .add_reading(
            Some("Test Article"),
            "This is the content of the article.",
            Some("test source"),
        )
        .await
        .unwrap();
    assert_eq!(reading.title.as_deref(), Some("Test Article"));

    let fetched = db.get_reading(reading.id).await.unwrap().unwrap();
    assert_eq!(fetched.content, "This is the content of the article.");

    let readings = db.list_readings(100, 0).await.unwrap();
    assert_eq!(readings.len(), 1);

    let deleted = db.delete_reading(reading.id).await.unwrap();
    assert!(deleted);

    let gone = db.get_reading(reading.id).await.unwrap();
    assert!(gone.is_none());
}
