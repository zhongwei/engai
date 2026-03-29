use engai_core::markdown::{MarkdownPhrase, MarkdownReading, MarkdownWord};

const WORD_MD: &str = r#"---
word: abandon
phonetic: /əˈbændən/
familiarity: 3
interval: 7
next_review: 2026-04-05 00:00:00
synced_at: 2026-03-29T10:00:00
---

# abandon

## Meaning
to leave completely and forever

## Examples
- He abandoned his car in the snow.
- She abandoned hope of finding her lost ring.

## Synonyms
- forsake
- desert

## AI Explanation
Abandon means to give up completely. It can be used for physical objects, people, or abstract concepts like hope.

## My Notes
- Used a lot in weather reports

## Review
- 2026-03-20: quality 4
- 2026-03-25: quality 5
"#;

#[test]
fn test_parse_word_markdown() {
    let word = MarkdownWord::parse(WORD_MD).unwrap();
    assert_eq!(word.word, "abandon");
    assert_eq!(word.phonetic.as_deref(), Some("/əˈbændən/"));
    assert_eq!(word.familiarity, 3);
    assert_eq!(word.interval, 7);
    assert!(word.next_review.is_some());
    assert_eq!(
        word.next_review.unwrap().format("%Y-%m-%d").to_string(),
        "2026-04-05"
    );
    assert_eq!(word.meaning.as_deref(), Some("to leave completely and forever"));
    assert_eq!(word.examples.len(), 2);
    assert_eq!(word.examples[0], "He abandoned his car in the snow.");
    assert_eq!(word.synonyms.len(), 2);
    assert_eq!(word.synonyms[0], "forsake");
    assert!(word.ai_explanation.is_some());
    assert!(word.ai_explanation.as_ref().unwrap().contains("give up completely"));
    assert_eq!(word.my_notes.len(), 1);
    assert_eq!(word.my_notes[0], "Used a lot in weather reports");
    assert_eq!(word.reviews.len(), 2);
    assert!(word.reviews[0].contains("quality 4"));
}

#[test]
fn test_generate_word_markdown() {
    let word = MarkdownWord {
        word: "test".to_string(),
        phonetic: Some("/tɛst/".to_string()),
        familiarity: 1,
        interval: 2,
        next_review: None,
        meaning: Some("a procedure intended to establish quality".to_string()),
        examples: vec!["This is a test.".to_string()],
        synonyms: vec![],
        ai_explanation: None,
        my_notes: vec![],
        reviews: vec![],
    };
    let md = word.to_markdown_string();
    assert!(md.contains("# test"));
    assert!(md.contains("a procedure intended to establish quality"));
    assert!(md.contains("phonetic: /tɛst/"));
    assert!(md.contains("familiarity: 1"));
}

const PHRASE_MD: &str = r#"---
phrase: break the ice
familiarity: 2
interval: 3
next_review: 2026-04-01 12:00:00
synced_at: 2026-03-28T08:00:00
---

# break the ice

## Meaning
To initiate conversation in a social setting.

## Examples
- She told a joke to break the ice.

## AI Explanation
An idiom used when you start a conversation to make people feel more relaxed.

## My Notes
- Common in business meetings

## Review
- 2026-03-20: quality 3
"#;

#[test]
fn test_parse_phrase_markdown() {
    let phrase = MarkdownPhrase::parse(PHRASE_MD).unwrap();
    assert_eq!(phrase.phrase, "break the ice");
    assert_eq!(phrase.familiarity, 2);
    assert_eq!(phrase.interval, 3);
    assert!(phrase.next_review.is_some());
    assert_eq!(
        phrase.meaning.as_deref(),
        Some("To initiate conversation in a social setting.")
    );
    assert_eq!(phrase.examples.len(), 1);
    assert_eq!(phrase.examples[0], "She told a joke to break the ice.");
    assert!(phrase.ai_explanation.is_some());
    assert_eq!(phrase.my_notes.len(), 1);
    assert_eq!(phrase.reviews.len(), 1);
}

const READING_MD: &str = r#"---
title: The Art of Focus
source: https://example.com/focus
imported_at: 2026-03-25T09:00:00
synced_at: 2026-03-29T10:00:00
---

# The Art of Focus

## Content
Focus is the ability to direct one's attention.
It requires practice and discipline.

## Vocabulary
- concentration
- mindfulness
- discipline

## Summary (AI)
This article discusses the importance of focus and provides tips for improving concentration.

## My Notes
- Need to practice this daily
"#;

#[test]
fn test_parse_reading_markdown() {
    let reading = MarkdownReading::parse(READING_MD).unwrap();
    assert_eq!(reading.title, "The Art of Focus");
    assert_eq!(reading.source.as_deref(), Some("https://example.com/focus"));
    assert!(reading.content.contains("Focus is the ability"));
    assert_eq!(reading.vocabulary.len(), 3);
    assert_eq!(reading.vocabulary[0], "concentration");
    assert!(reading.summary.is_some());
    assert!(reading.summary.as_ref().unwrap().contains("importance of focus"));
    assert_eq!(reading.my_notes.len(), 1);
    assert_eq!(reading.my_notes[0], "Need to practice this daily");
}

#[tokio::test]
async fn test_roundtrip_word_file() {
    let word = MarkdownWord {
        word: "ephemeral".to_string(),
        phonetic: Some("/ɪˈfɛmərəl/".to_string()),
        familiarity: 2,
        interval: 5,
        next_review: None,
        meaning: Some("lasting for a very short time".to_string()),
        examples: vec!["The ephemeral beauty of cherry blossoms.".to_string()],
        synonyms: vec!["fleeting".to_string(), "transient".to_string()],
        ai_explanation: Some("Ephemeral describes something that is temporary.".to_string()),
        my_notes: vec!["Good word for essays".to_string()],
        reviews: vec![],
    };

    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();

    word.save_to_file(&path).unwrap();
    let parsed = MarkdownWord::parse_file(&path).unwrap();
    assert_eq!(parsed.word, "ephemeral");
    assert_eq!(parsed.phonetic.as_deref(), Some("/ɪˈfɛmərəl/"));
    assert_eq!(parsed.familiarity, 2);
    assert_eq!(parsed.interval, 5);
    assert_eq!(parsed.meaning.as_deref(), Some("lasting for a very short time"));
    assert_eq!(parsed.examples.len(), 1);
    assert_eq!(parsed.synonyms.len(), 2);
    assert!(parsed.ai_explanation.is_some());
    assert_eq!(parsed.my_notes.len(), 1);

    let regenerated = parsed.to_markdown_string();
    let reparsed = MarkdownWord::parse(&regenerated).unwrap();
    assert_eq!(reparsed.word, parsed.word);
    assert_eq!(reparsed.phonetic, parsed.phonetic);
    assert_eq!(reparsed.familiarity, parsed.familiarity);
    assert_eq!(reparsed.interval, parsed.interval);
    assert_eq!(reparsed.meaning, parsed.meaning);
    assert_eq!(reparsed.examples, parsed.examples);
    assert_eq!(reparsed.synonyms, parsed.synonyms);
    assert_eq!(reparsed.ai_explanation, parsed.ai_explanation);
    assert_eq!(reparsed.my_notes, parsed.my_notes);
}
