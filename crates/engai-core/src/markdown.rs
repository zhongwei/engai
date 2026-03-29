use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use gray_matter::{engine::YAML, Matter};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct WordFrontmatter {
    word: String,
    phonetic: Option<String>,
    familiarity: i32,
    interval: i32,
    next_review: Option<String>,
    synced_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PhraseFrontmatter {
    phrase: String,
    familiarity: i32,
    interval: i32,
    next_review: Option<String>,
    synced_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReadingFrontmatter {
    title: Option<String>,
    source: Option<String>,
    imported_at: Option<String>,
    synced_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MarkdownWord {
    pub word: String,
    pub phonetic: Option<String>,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<NaiveDateTime>,
    pub meaning: Option<String>,
    pub examples: Vec<String>,
    pub synonyms: Vec<String>,
    pub ai_explanation: Option<String>,
    pub my_notes: Vec<String>,
    pub reviews: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MarkdownPhrase {
    pub phrase: String,
    pub familiarity: i32,
    pub interval: i32,
    pub next_review: Option<NaiveDateTime>,
    pub meaning: Option<String>,
    pub examples: Vec<String>,
    pub ai_explanation: Option<String>,
    pub my_notes: Vec<String>,
    pub reviews: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MarkdownReading {
    pub title: String,
    pub source: Option<String>,
    pub content: String,
    pub vocabulary: Vec<String>,
    pub summary: Option<String>,
    pub my_notes: Vec<String>,
}

fn parse_frontmatter<T: serde::de::DeserializeOwned>(content: &str) -> Result<(T, String)> {
    let matter = Matter::<YAML>::new();
    let result = matter.parse(content);
    let fm: T = result
        .data
        .context("Missing frontmatter")?
        .deserialize::<T>()
        .context("Failed to deserialize frontmatter")?;
    Ok((fm, result.content))
}

fn parse_date(s: &str) -> Result<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .context(format!("Failed to parse date: {s}"))
}

fn extract_section<'a>(body: &'a str, heading: &str) -> Option<&'a str> {
    let marker = format!("## {heading}");
    let start = body.find(&marker)?;
    let rest = &body[start + marker.len()..];
    let end = rest.find("\n## ").map(|i| start + marker.len() + i);
    let section = match end {
        Some(e) => &body[start + marker.len()..e],
        None => rest,
    };
    let section = section.trim_start_matches('\n').trim_end();
    if section.is_empty() {
        None
    } else {
        Some(section)
    }
}

fn extract_list(body: &str, heading: &str) -> Vec<String> {
    let Some(section) = extract_section(body, heading) else {
        return Vec::new();
    };
    section
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with("- ") {
                Some(line[2..].to_string())
            } else {
                None
            }
        })
        .collect()
}

fn format_date(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

impl MarkdownWord {
    pub fn parse(content: &str) -> Result<Self> {
        let (fm, body): (WordFrontmatter, String) = parse_frontmatter(content)?;
        let next_review = match fm.next_review {
            Some(ref s) if !s.is_empty() => Some(parse_date(s)?),
            _ => None,
        };
        Ok(Self {
            word: fm.word,
            phonetic: fm.phonetic,
            familiarity: fm.familiarity,
            interval: fm.interval,
            next_review,
            meaning: extract_section(&body, "Meaning").map(|s| s.to_string()),
            examples: {
                let mut ex = extract_list(&body, "Examples");
                if ex.is_empty() {
                    ex = extract_list(&body, "Example");
                }
                ex
            },
            synonyms: extract_list(&body, "Synonyms"),
            ai_explanation: extract_section(&body, "AI Explanation").map(|s| s.to_string()),
            my_notes: extract_list(&body, "My Notes"),
            reviews: extract_list(&body, "Review"),
        })
    }

    pub fn parse_file(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
        Self::parse(&content)
    }

    pub fn to_markdown_string(&self) -> String {
        let mut md = String::new();
        md.push_str("---\n");
        md.push_str(&format!("word: {}\n", self.word));
        if let Some(ref ph) = self.phonetic {
            md.push_str(&format!("phonetic: {}\n", ph));
        }
        md.push_str(&format!("familiarity: {}\n", self.familiarity));
        md.push_str(&format!("interval: {}\n", self.interval));
        if let Some(dt) = self.next_review {
            md.push_str(&format!("next_review: {}\n", format_date(dt)));
        }
        md.push_str("---\n\n");
        md.push_str(&format!("# {}\n", self.word));
        if let Some(ref m) = self.meaning {
            md.push_str("\n## Meaning\n");
            md.push_str(m);
            md.push_str("\n");
        }
        if !self.examples.is_empty() {
            md.push_str("\n## Examples\n");
            for ex in &self.examples {
                md.push_str(&format!("- {}\n", ex));
            }
        }
        if !self.synonyms.is_empty() {
            md.push_str("\n## Synonyms\n");
            for syn in &self.synonyms {
                md.push_str(&format!("- {}\n", syn));
            }
        }
        if let Some(ref ai) = self.ai_explanation {
            md.push_str("\n## AI Explanation\n");
            md.push_str(ai);
            md.push_str("\n");
        }
        if !self.my_notes.is_empty() {
            md.push_str("\n## My Notes\n");
            for note in &self.my_notes {
                md.push_str(&format!("- {}\n", note));
            }
        }
        if !self.reviews.is_empty() {
            md.push_str("\n## Review\n");
            for rev in &self.reviews {
                md.push_str(&format!("- {}\n", rev));
            }
        }
        md
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_markdown_string())
            .with_context(|| format!("Writing {}", path.display()))?;
        Ok(())
    }
}

impl MarkdownPhrase {
    pub fn parse(content: &str) -> Result<Self> {
        let (fm, body): (PhraseFrontmatter, String) = parse_frontmatter(content)?;
        let next_review = match fm.next_review {
            Some(ref s) if !s.is_empty() => Some(parse_date(s)?),
            _ => None,
        };
        Ok(Self {
            phrase: fm.phrase,
            familiarity: fm.familiarity,
            interval: fm.interval,
            next_review,
            meaning: extract_section(&body, "Meaning").map(|s| s.to_string()),
            examples: {
                let mut ex = extract_list(&body, "Examples");
                if ex.is_empty() {
                    ex = extract_list(&body, "Example");
                }
                ex
            },
            ai_explanation: extract_section(&body, "AI Explanation").map(|s| s.to_string()),
            my_notes: extract_list(&body, "My Notes"),
            reviews: extract_list(&body, "Review"),
        })
    }

    pub fn parse_file(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
        Self::parse(&content)
    }

    pub fn to_markdown_string(&self) -> String {
        let mut md = String::new();
        md.push_str("---\n");
        md.push_str(&format!("phrase: {}\n", self.phrase));
        md.push_str(&format!("familiarity: {}\n", self.familiarity));
        md.push_str(&format!("interval: {}\n", self.interval));
        if let Some(dt) = self.next_review {
            md.push_str(&format!("next_review: {}\n", format_date(dt)));
        }
        md.push_str("---\n\n");
        md.push_str(&format!("# {}\n", self.phrase));
        if let Some(ref m) = self.meaning {
            md.push_str("\n## Meaning\n");
            md.push_str(m);
            md.push_str("\n");
        }
        if !self.examples.is_empty() {
            md.push_str("\n## Examples\n");
            for ex in &self.examples {
                md.push_str(&format!("- {}\n", ex));
            }
        }
        if let Some(ref ai) = self.ai_explanation {
            md.push_str("\n## AI Explanation\n");
            md.push_str(ai);
            md.push_str("\n");
        }
        if !self.my_notes.is_empty() {
            md.push_str("\n## My Notes\n");
            for note in &self.my_notes {
                md.push_str(&format!("- {}\n", note));
            }
        }
        if !self.reviews.is_empty() {
            md.push_str("\n## Review\n");
            for rev in &self.reviews {
                md.push_str(&format!("- {}\n", rev));
            }
        }
        md
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_markdown_string())
            .with_context(|| format!("Writing {}", path.display()))?;
        Ok(())
    }
}

impl MarkdownReading {
    pub fn parse(content: &str) -> Result<Self> {
        let (fm, body): (ReadingFrontmatter, String) = parse_frontmatter(content)?;
        Ok(Self {
            title: fm.title.unwrap_or_default(),
            source: fm.source,
            content: extract_section(&body, "Content")
                .map(|s| s.to_string())
                .unwrap_or_default(),
            vocabulary: extract_list(&body, "Vocabulary"),
            summary: extract_section(&body, "Summary (AI)").map(|s| s.to_string()),
            my_notes: extract_list(&body, "My Notes"),
        })
    }

    pub fn parse_file(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
        Self::parse(&content)
    }

    pub fn to_markdown_string(&self) -> String {
        let mut md = String::new();
        md.push_str("---\n");
        if !self.title.is_empty() {
            md.push_str(&format!("title: {}\n", self.title));
        }
        if let Some(ref src) = self.source {
            md.push_str(&format!("source: {}\n", src));
        }
        md.push_str("---\n\n");
        md.push_str(&format!("# {}\n", self.title));
        md.push_str("\n## Content\n");
        md.push_str(&self.content);
        md.push_str("\n");
        if !self.vocabulary.is_empty() {
            md.push_str("\n## Vocabulary\n");
            for v in &self.vocabulary {
                md.push_str(&format!("- {}\n", v));
            }
        }
        if let Some(ref s) = self.summary {
            md.push_str("\n## Summary (AI)\n");
            md.push_str(s);
            md.push_str("\n");
        }
        if !self.my_notes.is_empty() {
            md.push_str("\n## My Notes\n");
            for note in &self.my_notes {
                md.push_str(&format!("- {}\n", note));
            }
        }
        md
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_markdown_string())
            .with_context(|| format!("Writing {}", path.display()))?;
        Ok(())
    }
}
