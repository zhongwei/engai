use engai_core::prompt::PromptEngine;
use std::io::Write;
use tempfile::TempDir;

#[tokio::test]
async fn test_load_and_render_template() {
    let tmp = TempDir::new().unwrap();
    let template_path = tmp.path().join("test.md");
    let mut f = std::fs::File::create(&template_path).unwrap();
    write!(f, "Explain the word: {{{{word}}}}\nLevel: {{{{level}}}}").unwrap();

    let engine = PromptEngine::new(tmp.path().to_path_buf());
    let rendered = engine
        .render("test.md", &[("word", "abandon"), ("level", "B2")])
        .await
        .unwrap();

    assert!(rendered.contains("abandon"), "output should contain 'abandon'");
    assert!(rendered.contains("B2"), "output should contain 'B2'");
    assert!(!rendered.contains("{{"), "output should not contain unresolved placeholders");
}

#[tokio::test]
async fn test_template_not_found() {
    let tmp = TempDir::new().unwrap();
    let engine = PromptEngine::new(tmp.path().to_path_buf());

    let result = engine.render("nonexistent.md", &[]).await;
    assert!(result.is_err(), "should fail for missing template");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("not found"), "error should mention 'not found'");
}
