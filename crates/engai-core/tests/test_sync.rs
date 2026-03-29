use std::sync::Arc;

use engai_core::db::Db;
use engai_core::markdown::MarkdownWord;
use engai_core::sync::SyncEngine;

#[tokio::test]
async fn test_sync_word_from_db_to_markdown() {
    let db = Arc::new(Db::new_in_memory().await.unwrap());
    let tmp = tempfile::TempDir::new().unwrap();
    let docs_path = tmp.path();
    let prompts_path = tmp.path().join("prompts");

    db.add_word("hello", Some("/həˈloʊ/"), Some("a greeting"))
        .await
        .unwrap();

    let engine = SyncEngine::new(db.clone(), docs_path, &prompts_path);
    engine.sync_words().await.unwrap();

    let md_file = docs_path.join("01_vocab").join("hello.md");
    assert!(md_file.exists());

    let parsed = MarkdownWord::parse_file(&md_file).unwrap();
    assert_eq!(parsed.word, "hello");
    assert_eq!(parsed.phonetic.as_deref(), Some("/həˈloʊ/"));
    assert_eq!(parsed.meaning.as_deref(), Some("a greeting"));
}

#[tokio::test]
async fn test_sync_word_from_markdown_to_db() {
    let db = Arc::new(Db::new_in_memory().await.unwrap());
    let tmp = tempfile::TempDir::new().unwrap();
    let docs_path = tmp.path();
    let prompts_path = tmp.path().join("prompts");

    let vocab_dir = docs_path.join("01_vocab");
    std::fs::create_dir_all(&vocab_dir).unwrap();
    std::fs::write(
        vocab_dir.join("serendipity.md"),
        r#"---
word: serendipity
phonetic: /ˌsɛrənˈdɪpɪti/
familiarity: 2
interval: 3
---
# serendipity

## Meaning
the occurrence of events by chance in a happy way

## Examples
- Finding a great book by accident.
"#,
    )
    .unwrap();

    let engine = SyncEngine::new(db.clone(), docs_path, &prompts_path);
    engine.sync_words().await.unwrap();

    let word = db.get_word("serendipity").await.unwrap().unwrap();
    assert_eq!(word.word, "serendipity");
    assert_eq!(word.meaning.as_deref(), Some("the occurrence of events by chance in a happy way"));
    assert_eq!(word.phonetic.as_deref(), Some("/ˌsɛrənˈdɪpɪti/"));
    assert_eq!(word.familiarity, 2);
    assert_eq!(word.interval, 3);

    let examples = db.get_examples("word", word.id).await.unwrap();
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].sentence, "Finding a great book by accident.");
}

#[tokio::test]
async fn test_sync_phrase() {
    let db = Arc::new(Db::new_in_memory().await.unwrap());
    let tmp = tempfile::TempDir::new().unwrap();
    let docs_path = tmp.path();
    let prompts_path = tmp.path().join("prompts");

    let phrases_dir = docs_path.join("02_phrases");
    std::fs::create_dir_all(&phrases_dir).unwrap();
    std::fs::write(
        phrases_dir.join("take_off.md"),
        r#"---
phrase: take off
familiarity: 1
interval: 1
---
# take off

## Meaning
to leave the ground (for aircraft)
"#,
    )
    .unwrap();

    let engine = SyncEngine::new(db.clone(), docs_path, &prompts_path);
    engine.sync_phrases().await.unwrap();

    let phrase = db.get_phrase("take off").await.unwrap().unwrap();
    assert_eq!(phrase.phrase, "take off");
    assert_eq!(phrase.meaning.as_deref(), Some("to leave the ground (for aircraft)"));
    assert_eq!(phrase.familiarity, 1);
}
