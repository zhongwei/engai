# Engai Phase 3: TUI + Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a ratatui-based terminal UI with sidebar navigation (Vocab, Review, Read, Chat, Stats), enable dual-mode launch (Web + TUI concurrently), and polish error handling, logging, and edge cases across the entire system.

**Architecture:** The TUI runs in an alternate screen buffer using crossterm as the ratatui backend. It shares `Arc<Db>` and other state with the web server. When no CLI subcommand is given, the app spawns both the web server and TUI concurrently via `tokio::select!`. Each TUI panel (vocab list, word detail, review session, readings, chat, stats) is a separate component rendered into the main content area based on sidebar selection. User input is handled via crossterm events polled in the TUI's async event loop.

**Tech Stack:** Rust (ratatui 0.30.0, crossterm 0.29, tokio 1.50.0), existing engai-core (db, ai, review, sync, prompt)

---

## File Structure

```
engai/
├── crates/
│   ├── engai-core/
│   │   └── src/
│   │       ├── db.rs              # UNCHANGED
│   │       ├── ai.rs              # UNCHANGED
│   │       ├── review.rs          # UNCHANGED
│   │       ├── sync.rs            # UNCHANGED
│   │       ├── prompt.rs          # UNCHANGED
│   │       ├── config.rs          # UNCHANGED
│   │       ├── models.rs          # UNCHANGED
│   │       ├── markdown.rs        # UNCHANGED
│   │       └── lib.rs             # UNCHANGED
│   └── engai/
│       ├── Cargo.toml             # ADD: ratatui, crossterm, textwrap
│       ├── build.rs               # UNCHANGED
│       └── src/
│           ├── main.rs            # MODIFY: default → Web + TUI concurrently
│           ├── server.rs          # UNCHANGED
│           ├── state.rs           # UNCHANGED
│           ├── error.rs           # UNCHANGED
│           ├── cmd_*.rs           # UNCHANGED (CLI commands)
│           ├── routes/            # UNCHANGED (Web API)
│           └── tui/
│               ├── mod.rs          # CREATE: TUI app entry, event loop, terminal setup
│               ├── app.rs          # CREATE: App state machine (current panel, data, mode)
│               ├── ui.rs           # CREATE: main layout (sidebar + content + status bar)
│               ├── event.rs        # CREATE: crossterm event handling
│               ├── panel_vocab.rs  # CREATE: word/phrase list + word detail view
│               ├── panel_review.rs # CREATE: SM-2 review session (flash card style)
│               ├── panel_read.rs   # CREATE: reading list + content view
│               ├── panel_chat.rs   # CREATE: terminal AI chat (non-streaming)
│               └── panel_stats.rs  # CREATE: dashboard stats view
```

---

### Task 1: Add TUI Dependencies

**Files:**
- Modify: `crates/engai/Cargo.toml`

- [ ] **Step 1: Add ratatui and crossterm to engai Cargo.toml**

Add these lines to the `[dependencies]` section in `crates/engai/Cargo.toml`:
```toml
ratatui = "0.30.0"
crossterm = "0.29"
textwrap = "0.16"
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p engai`
Expected: Compiles (new deps unused but added)

- [ ] **Step 3: Commit**

```bash
git add crates/engai/Cargo.toml crates/engai/Cargo.lock
git commit -m "chore: add ratatui, crossterm, textwrap dependencies for TUI"
```

---

### Task 2: TUI App State Machine

**Files:**
- Create: `crates/engai/src/tui/mod.rs`
- Create: `crates/engai/src/tui/app.rs`
- Create: `crates/engai/src/tui/event.rs`
- Create: `crates/engai/src/tui/ui.rs`

- [ ] **Step 1: Create TUI module with app state**

`crates/engai/src/tui/app.rs`:
```rust
use engai_core::models::{ChatEntry, Phrase, Reading, Word};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Vocab,
    Review,
    Read,
    Chat,
    Stats,
}

impl Panel {
    pub const ALL: [Panel; 5] = [Panel::Vocab, Panel::Review, Panel::Read, Panel::Chat, Panel::Stats];

    pub fn label(self) -> &'static str {
        match self {
            Panel::Vocab => "Vocabulary",
            Panel::Review => "Review",
            Panel::Read => "Reading",
            Panel::Chat => "Chat",
            Panel::Stats => "Stats",
        }
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|&p| p == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

pub struct App {
    pub panel: Panel,
    pub should_quit: bool,

    // Vocab panel
    pub words: Vec<Word>,
    pub phrases: Vec<Phrase>,
    pub vocab_tab: VocabTab,
    pub vocab_list_index: usize,
    pub vocab_detail: Option<VocabDetail>,

    // Review panel
    pub review_items: Vec<ReviewItem>,
    pub review_index: usize,
            pub review_show_answer: bool,
            pub review_loading: bool,

    // Read panel
    pub readings: Vec<Reading>,
    pub reading_list_index: usize,
    pub reading_detail: Option<ReadingDetail>,

    // Chat panel
    pub chat_messages: Vec<ChatEntry>,
    pub chat_input: String,
    pub chat_loading: bool,
    pub chat_error: Option<String>,

    // Stats panel
    pub stats: Option<StatsData>,

    // Status
    pub status_message: Option<String>,
    pub status_message_time: Option<std::time::Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VocabTab {
    Words,
    Phrases,
}

#[derive(Debug, Clone)]
pub enum VocabDetail {
    Word(Word),
    Phrase(Phrase),
}

#[derive(Debug, Clone)]
pub struct ReviewItem {
    pub target_type: String,
    pub id: i64,
    pub display: String,
    pub meaning: Option<String>,
    pub familiarity: i32,
    pub interval: i32,
    pub ease_factor: f64,
}

#[derive(Debug, Clone)]
pub struct ReadingDetail {
    pub reading: Reading,
    pub analysis: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StatsData {
    pub word_count: i64,
    pub phrase_count: i64,
    pub pending_reviews: i64,
    pub reviewed_today: i64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            panel: Panel::Vocab,
            should_quit: false,
            words: Vec::new(),
            phrases: Vec::new(),
            vocab_tab: VocabTab::Words,
            vocab_list_index: 0,
            vocab_detail: None,
            review_items: Vec::new(),
            review_index: 0,
            review_show_answer: false,
            review_loading: false,
            readings: Vec::new(),
            reading_list_index: 0,
            reading_detail: None,
            chat_messages: Vec::new(),
            chat_input: String::new(),
            chat_loading: false,
            chat_error: None,
            stats: None,
            status_message: None,
            status_message_time: None,
        }
    }
}

impl App {
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.status_message_time = Some(std::time::Instant::now());
    }

    pub fn clear_status_if_expired(&mut self) {
        if let Some(t) = self.status_message_time {
            if t.elapsed().as_secs() > 3 {
                self.status_message = None;
                self.status_message_time = None;
            }
        }
    }
}
```

- [ ] **Step 2: Create event module**

`crates/engai/src/tui/event.rs`:
```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyCode, KeyModifiers),
    Tick,
}

pub fn poll_event(timeout: Duration) -> Option<AppEvent> {
    if event::poll(timeout).ok()? {
        match event::read().ok()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                Some(AppEvent::Key(key.code, key.modifiers))
            }
            Event::Resize(_, _) => Some(AppEvent::Tick),
            _ => None,
        }
    } else {
        Some(AppEvent::Tick)
    }
}
```

- [ ] **Step 3: Create UI layout module**

`crates/engai/src/tui/ui.rs`:
```rust
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, Panel, VocabDetail, VocabTab};

const SIDEBAR_WIDTH: u16 = 18;
const TITLE_HEIGHT: u16 = 1;
const STATUS_HEIGHT: u16 = 1;

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();
    if size.width < 40 || size.height < 10 {
        render_too_small(f, size);
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(TITLE_HEIGHT),
        Constraint::Min(0),
        Constraint::Length(STATUS_HEIGHT),
    ])
    .split(size);

    render_title(f, chunks[0]);

    let main = Layout::horizontal([
        Constraint::Length(SIDEBAR_WIDTH),
        Constraint::Min(0),
    ])
    .split(chunks[1]);

    render_sidebar(f, main[0], app);
    render_content(f, main[1], app);
    render_status(f, chunks[2], app);
}

fn render_too_small(f: &mut Frame, area: Rect) {
    let msg = Paragraph::new("Terminal too small (min 40x10)")
        .style(Style::default().fg(Color::Red))
        .centered();
    f.render_widget(msg, area);
}

fn render_title(f: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(" Engai ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::raw(" - AI English Learning System"),
    ]);
    f.render_widget(
        Paragraph::new(title).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

fn render_sidebar(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = Panel::ALL
        .iter()
        .map(|&panel| {
            let label = panel.label();
            let selected = app.panel == panel;
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Span::styled(format!(" {} ", label), style))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).title(" Menu "));
    f.render_widget(list, area);
}

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let msg = if let Some(ref status) = app.status_message {
        status.clone()
    } else {
        let hints = match app.panel {
            Panel::Vocab => "[Enter] Detail [Tab] Switch [q] Quit".to_string(),
            Panel::Review => "[Space] Flip [1-5] Rate [n] Skip [q] Quit".to_string(),
            Panel::Read => "[Enter] Detail [q] Quit".to_string(),
            Panel::Chat => "[Enter] Send [q] Quit".to_string(),
            Panel::Stats => "[r] Refresh [q] Quit".to_string(),
        };
        hints
    };
    let status = Paragraph::new(Span::styled(
        format!(" {}", msg),
        Style::default().fg(Color::DarkGray).bg(Color::Black),
    ));
    f.render_widget(status, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    match app.panel {
        Panel::Vocab => render_vocab(f, area, app),
        Panel::Review => render_review(f, area, app),
        Panel::Read => render_read(f, area, app),
        Panel::Chat => render_chat(f, area, app),
        Panel::Stats => render_stats(f, area, app),
    }
}

fn render_vocab(f: &mut Frame, area: Rect, app: &App) {
    if let Some(ref detail) = app.vocab_detail {
        render_vocab_detail(f, area, detail);
        return;
    }

    let header = match app.vocab_tab {
        VocabTab::Words => "Words",
        VocabTab::Phrases => "Phrases",
    };
    let tab_label = format!(
        "{} [Tab] Switch  {} items",
        header,
        app.words.len() + app.phrases.len()
    );

    let (items, count) = match app.vocab_tab {
        VocabTab::Words => {
            let items: Vec<ListItem> = app
                .words
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    let fam = familiarity_bar(w.familiarity);
                    let meaning = w.meaning.as_deref().unwrap_or("—").chars().take(30).collect::<String>();
                    let line = format!(" {}  {:20}  {}  {}", fam, w.word, meaning, w.familiarity);
                    let style = if i == app.vocab_list_index {
                        Style::default().bg(Color::DarkGray).bold()
                    } else {
                        Style::default()
                    };
                    ListItem::new(line).style(style)
                })
                .collect();
            (items, app.words.len())
        }
        VocabTab::Phrases => {
            let items: Vec<ListItem> = app
                .phrases
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let fam = familiarity_bar(p.familiarity);
                    let meaning = p.meaning.as_deref().unwrap_or("—").chars().take(25).collect::<String>();
                    let line = format!(" {}  {:22}  {}  {}", fam, p.phrase, meaning, p.familiarity);
                    let style = if i == app.vocab_list_index {
                        Style::default().bg(Color::DarkGray).bold()
                    } else {
                        Style::default()
                    };
                    ListItem::new(line).style(style)
                })
                .collect();
            (items, app.phrases.len())
        }
    };

    let header = format!("{} (showing {} items)", tab_label, count);
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(header))
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(list, area);
}

fn familiarity_bar(level: i32) -> &'static str {
    match level {
        0 => "[·    ]",
        1 => "[▏    ]",
        2 => "[▎    ]",
        3 => "[▍▌   ]",
        4 => "[▍▌▋  ]",
        5 => "[█████]",
        _ => "[?????]",
    }
}

fn render_vocab_detail(f: &mut Frame, area: Rect, detail: &VocabDetail) {
    let lines = match detail {
        VocabDetail::Word(w) => {
            vec![
                Line::styled(
                    format!("  {}", w.word),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Line::from(""),
                Line::styled("  Meaning", Style::default().add_modifier(Modifier::BOLD)),
                Line::from(format!("  {}", w.meaning.as_deref().unwrap_or("(not set)"))),
                Line::from(""),
                Line::styled("  Phonetic", Style::default().add_modifier(Modifier::BOLD)),
                Line::from(format!(
                    "  {}",
                    w.phonetic.as_deref().unwrap_or("(not set)")
                )),
                Line::from(""),
                Line::styled(
                    format!("  Familiarity: {} / Interval: {} days", w.familiarity, w.interval),
                    Style::default().fg(Color::Yellow),
                ),
                Line::from(format!(
                    "  Ease Factor: {:.2}",
                    w.ease_factor
                )),
            ]
        }
        VocabDetail::Phrase(p) => {
            vec![
                Line::styled(
                    format!("  {}", p.phrase),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Line::from(""),
                Line::styled("  Meaning", Style::default().add_modifier(Modifier::BOLD)),
                Line::from(format!("  {}", p.meaning.as_deref().unwrap_or("(not set)"))),
                Line::from(""),
                Line::styled(
                    format!("  Familiarity: {} / Interval: {} days", p.familiarity, p.interval),
                    Style::default().fg(Color::Yellow),
                ),
                Line::from(format!(
                    "  Ease Factor: {:.2}",
                    p.ease_factor
                )),
            ]
        }
    };

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Detail [Esc] Back "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn render_review(f: &mut Frame, area: Rect, app: &App) {
    if app.review_loading {
        let para = Paragraph::new("Loading review queue...")
            .style(Style::default().fg(Color::Yellow))
            .centered()
            .block(Block::default().borders(Borders::ALL).title(" Review "));
        f.render_widget(para, area);
        return;
    }

    if app.review_items.is_empty() {
        let para = Paragraph::new("No items to review!")
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .centered()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Review "),
            );
        f.render_widget(para, area);
        return;
    }

    if app.review_index >= app.review_items.len() {
        let total = app.review_items.len();
        let para = Paragraph::new(format!("Review complete! {} items reviewed.", total))
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .centered()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Review "),
            );
        f.render_widget(para, area);
        return;
    }

    let item = &app.review_items[app.review_index];
    let progress = format!(
        "{}/{}",
        app.review_index + 1,
        app.review_items.len()
    );

    let inner = area.inner(Margin::new(2, 1));

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(4),
        Constraint::Length(3),
    ])
    .split(inner);

    // Progress
    let progress_line = Paragraph::new(format!(
        "Progress: {}  |  Familiarity: {}  |  [Space] Flip  [1-5] Rate  [n] Skip",
        progress, item.familiarity
    ))
    .style(Style::default().fg(Color::Yellow));
    f.render_widget(progress_line, chunks[0]);

    // Card
    if app.review_show_answer {
        let meaning = item.meaning.as_deref().unwrap_or("(no meaning set)");
        let card_text = vec![
            Line::from(""),
            Line::styled(
                format!("  {}", item.display),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::styled(
                format!("  {}", meaning),
                Style::default().fg(Color::White),
            ),
        ];
        let card = Paragraph::new(card_text)
            .block(Block::default().borders(Borders::ALL).title(" Card "))
            .wrap(Wrap { trim: true });
        f.render_widget(card, chunks[1]);
    } else {
        let card_text = vec![
            Line::from(""),
            Line::styled(
                format!("  {}", item.display),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::styled(
                "  [Press Space to reveal answer]",
                Style::default().fg(Color::DarkGray),
            ),
        ];
        let card = Paragraph::new(card_text)
            .block(Block::default().borders(Borders::ALL).title(" Card "))
            .wrap(Wrap { trim: true });
        f.render_widget(card, chunks[1]);
    }

    // Rating legend
    let legend = Paragraph::new(
        "0=Again  1=Hard  2=Difficult  3=OK  4=Easy  5=Perfect",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(legend, chunks[2]);

    // Outer border
    let border_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Review - {} ", progress));
    f.render_widget(border_block, area);
}

fn render_read(f: &mut Frame, area: Rect, app: &App) {
    if let Some(ref detail) = app.reading_detail {
        render_reading_detail(f, area, detail);
        return;
    }

    let items: Vec<ListItem> = app
        .readings
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let title = r.title.as_deref().unwrap_or("(untitled)").chars().take(40).collect::<String>();
            let preview = r.content.chars().take(50).collect::<String>();
            let line = format!(" {}  {}  {}", title, preview, r.source.as_deref().unwrap_or(""));
            let style = if i == app.reading_list_index {
                Style::default().bg(Color::DarkGray).bold()
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let header = format!("Readings ({} items) [Enter] Detail", app.readings.len());
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(header));
    f.render_widget(list, area);
}

fn render_reading_detail(f: &mut Frame, area: Rect, detail: &super::app::ReadingDetail) {
    let title = detail
        .reading
        .title
        .as_deref()
        .unwrap_or("(untitled)");

    let mut lines = vec![
        Line::styled(
            format!("  {}", title),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::styled("  Content", Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
    ];

    for paragraph in detail.reading.content.split('\n').take(40) {
        lines.push(Line::from(format!("  {}", paragraph)));
    }

    if let Some(ref analysis) = detail.analysis {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "  AI Analysis",
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow),
        ));
        lines.push(Line::from(""));
        for line in analysis.split('\n').take(30) {
            lines.push(Line::from(format!("  {}", line)));
        }
    }

    let para = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Reading Detail [Esc] Back "),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn render_chat(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(area);

    let messages: Vec<Line> = app
        .chat_messages
        .iter()
        .flat_map(|msg| {
            let role_style = match msg.role.as_str() {
                "user" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                "assistant" => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                _ => Style::default(),
            };
            let prefix = match msg.role.as_str() {
                "user" => "You: ",
                "assistant" => "AI:  ",
                _ => "",
            };
            let content_lines = msg
                .content
                .split('\n')
                .map(|l| {
                    if l.is_empty() {
                        Line::from("")
                    } else {
                        Line::from(vec![
                            Span::styled(prefix, role_style),
                            Span::raw(l),
                        ])
                    }
                })
                .collect::<Vec<_>>();
            content_lines
        })
        .collect();

    let messages_para = Paragraph::new(messages)
        .block(Block::default().borders(Borders::ALL).title(" Chat "))
        .wrap(Wrap { trim: false });
    f.render_widget(messages_para, chunks[0]);

    let input_text = if app.chat_loading {
        "(sending...)".to_string()
    } else {
        format!("> {}", app.chat_input)
    };
    let input = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title(" Input "));
    f.render_widget(input, chunks[1]);

    if app.chat_loading {
        f.set_cursor_position((
            chunks[1].x + 2,
            chunks[1].y + 1,
        ));
    } else {
        f.set_cursor_position((
            chunks[1].x + 2 + app.chat_input.len() as u16,
            chunks[1].y + 1,
        ));
    }
}

fn render_stats(f: &mut Frame, area: Rect, app: &App) {
    let data = match &app.stats {
        Some(s) => s,
        None => {
            let para = Paragraph::new("Loading stats...")
                .style(Style::default().fg(Color::Yellow))
                .centered()
                .block(Block::default().borders(Borders::ALL).title(" Stats [r] Refresh "));
            f.render_widget(para, area);
            return;
        }
    };

    let lines = vec![
        Line::from(""),
        Line::styled(
            "  Learning Statistics",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from(format!("  Total Words:        {}", data.word_count)),
        Line::from(format!("  Total Phrases:      {}", data.phrase_count)),
        Line::from(format!(
            "  Pending Reviews:    {}",
            data.pending_reviews
        )),
        Line::from(format!(
            "  Reviewed Today:     {}",
            data.reviewed_today
        )),
        Line::from(""),
        Line::styled(
            "  Spaced Repetition:  SM-2 Algorithm",
            Style::default().fg(Color::DarkGray),
        ),
        Line::from(""),
        Line::styled(
            "  [r] Refresh  [q] Quit",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Dashboard "))
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}
```

- [ ] **Step 4: Create TUI mod.rs with terminal setup and event loop**

`crates/engai/src/tui/mod.rs`:
```rust
pub mod app;
pub mod event;
pub mod panel_chat;
pub mod panel_read;
pub mod panel_review;
pub mod panel_stats;
pub mod panel_vocab;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;
use std::time::Duration;

use crate::state::AppState;
use app::{App, Panel};

pub async fn run_tui(state: AppState) -> Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, state).await;

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: AppState,
) -> Result<()> {
    let mut app = App::default();

    // Load initial data
    panel_vocab::load_vocab(&state, &mut app).await;
    panel_stats::load_stats(&state, &mut app).await;

    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        match event::poll_event(tick_rate) {
            Some(event::AppEvent::Key(code, modifiers)) => {
                handle_key(&mut app, &state, code, modifiers).await;
            }
            Some(event::AppEvent::Tick) => {
                app.clear_status_if_expired();
            }
            None => continue,
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn on_panel_enter(app: &mut App, state: &AppState) {
    match app.panel {
        Panel::Vocab => panel_vocab::load_vocab(state, app).await,
        Panel::Review => {
            if app.review_items.is_empty() {
                panel_review::load_review(state, app).await;
            }
        }
        Panel::Read => {
            if app.readings.is_empty() {
                panel_read::load_readings(state, app).await;
            }
        }
        Panel::Chat => {
            if app.chat_messages.is_empty() {
                match state.db.get_recent_chat(50).await {
                    Ok(msgs) => app.chat_messages = msgs.into_iter().rev().collect(),
                    Err(_) => {}
                }
            }
        }
        Panel::Stats => panel_stats::load_stats(state, app).await,
    }
}

async fn handle_key(app: &mut App, state: &AppState, code: KeyCode, modifiers: KeyModifiers) {
    // Global quit
    if code == KeyCode::Char('q') && app.chat_input.is_empty() {
        app.should_quit = true;
        return;
    }
    if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    // Esc: go back
    if code == KeyCode::Esc {
        if app.vocab_detail.is_some() {
            app.vocab_detail = None;
            return;
        }
        if app.reading_detail.is_some() {
            app.reading_detail = None;
            return;
        }
    }

    // Panel-specific keys take priority
    match app.panel {
        Panel::Vocab => panel_vocab::handle_key(app, state, code).await,
        Panel::Review => panel_review::handle_key(app, state, code).await,
        Panel::Read => panel_read::handle_key(app, state, code).await,
        Panel::Chat => panel_chat::handle_key(app, state, code).await,
        Panel::Stats => panel_stats::handle_key(app, state, code).await,
    }

    // Global navigation (only if not in chat input mode)
    if !(app.panel == Panel::Chat && !app.chat_input.is_empty())
    {
        match code {
            KeyCode::Char('[') | KeyCode::Left => {
                app.panel = app.panel.prev();
                on_panel_enter(app, state).await;
            }
            KeyCode::Char(']') | KeyCode::Right => {
                app.panel = app.panel.next();
                on_panel_enter(app, state).await;
            }
            KeyCode::Tab => {
                if app.panel == Panel::Vocab && app.vocab_detail.is_none() {
                    panel_vocab::handle_tab(app);
                }
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 5: Create stub panel files**

`crates/engai/src/tui/panel_vocab.rs`:
```rust
use crossterm::event::KeyCode;
use engai_core::db::Db;
use std::sync::Arc;

use super::app::{App, VocabDetail, VocabTab};
use crate::state::AppState;

pub async fn load_vocab(state: &AppState, app: &mut App) {
    let db = &state.db;
    match db.list_words(None, None, 200, 0).await {
        Ok(words) => app.words = words,
        Err(e) => app.set_status(format!("Failed to load words: {}", e)),
    }
    match db.list_phrases(None, None, 200, 0).await {
        Ok(phrases) => app.phrases = phrases,
        Err(e) => app.set_status(format!("Failed to load phrases: {}", e)),
    }
}

pub fn handle_tab(app: &mut App) {
    match app.vocab_tab {
        VocabTab::Words => {
            app.vocab_tab = VocabTab::Phrases;
            app.vocab_list_index = 0;
        }
        VocabTab::Phrases => {
            app.vocab_tab = VocabTab::Words;
            app.vocab_list_index = 0;
        }
    }
}

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    if app.vocab_detail.is_some() {
        return; // Esc handled in mod.rs
    }

    match code {
        KeyCode::Enter => {
            match app.vocab_tab {
                VocabTab::Words => {
                    if let Some(w) = app.words.get(app.vocab_list_index) {
                        app.vocab_detail = Some(VocabDetail::Word(w.clone()));
                    }
                }
                VocabTab::Phrases => {
                    if let Some(p) = app.phrases.get(app.vocab_list_index) {
                        app.vocab_detail = Some(VocabDetail::Phrase(p.clone()));
                    }
                }
            }
        }
        KeyCode::Up => {
            if app.vocab_list_index > 0 {
                app.vocab_list_index -= 1;
            }
        }
        KeyCode::Down => {
            let max = match app.vocab_tab {
                VocabTab::Words => app.words.len(),
                VocabTab::Phrases => app.phrases.len(),
            };
            if max > 0 && app.vocab_list_index < max - 1 {
                app.vocab_list_index += 1;
            }
        }
        _ => {}
    }
}
```

`crates/engai/src/tui/panel_review.rs`:
```rust
use crossterm::event::KeyCode;

use super::app::{App, ReviewItem};
use crate::state::AppState;
use engai_core::review::calculate_next_review;

pub async fn load_review(state: &AppState, app: &mut App) {
    app.review_loading = true;
    let db = &state.db;

    let words = db.get_today_review_words().await.unwrap_or_default();
    let phrases = db.get_today_review_phrases().await.unwrap_or_default();

    let mut items: Vec<ReviewItem> = words
        .into_iter()
        .map(|w| ReviewItem {
            target_type: "word".to_string(),
            id: w.id,
            display: w.word,
            meaning: w.meaning,
            familiarity: w.familiarity,
            interval: w.interval,
            ease_factor: w.ease_factor,
        })
        .collect();
    items.extend(phrases.into_iter().map(|p| ReviewItem {
        target_type: "phrase".to_string(),
        id: p.id,
        display: p.phrase,
        meaning: p.meaning,
        familiarity: p.familiarity,
        interval: p.interval,
        ease_factor: p.ease_factor,
    }));

    app.review_items = items;
    app.review_index = 0;
    app.review_show_answer = false;
    app.review_loading = false;
}

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    if app.review_loading || app.review_items.is_empty() {
        return;
    }

    if app.review_index >= app.review_items.len() {
        return;
    }

    match code {
        KeyCode::Char(' ') => {
            app.review_show_answer = !app.review_show_answer;
        }
        KeyCode::Char('0') => submit_review(app, state, 0).await,
        KeyCode::Char('1') => submit_review(app, state, 1).await,
        KeyCode::Char('2') => submit_review(app, state, 2).await,
        KeyCode::Char('3') => submit_review(app, state, 3).await,
        KeyCode::Char('4') => submit_review(app, state, 4).await,
        KeyCode::Char('5') => submit_review(app, state, 5).await,
        _ => {}
    }
}

async fn submit_review(app: &mut App, state: &AppState, quality: i32) {
    if app.review_index >= app.review_items.len() {
        return;
    }

    // quality is 0-5, passed directly from key press
    let quality = quality.clamp(0, 5);

    let item = app.review_items[app.review_index].clone();
    let result =
        calculate_next_review(quality, item.interval, item.ease_factor);

    let db = &state.db;
    let _ = db
        .add_review(&item.target_type, item.id, quality)
        .await;

    match item.target_type.as_str() {
        "word" => {
            let _ = db
                .update_word(
                    item.id,
                    None,
                    None,
                    None,
                    Some(result.familiarity),
                    Some(result.next_review),
                    Some(result.interval),
                    Some(result.ease_factor),
                )
                .await;
        }
        "phrase" => {
            let _ = db
                .update_phrase(
                    item.id,
                    None,
                    None,
                    Some(result.familiarity),
                    Some(result.next_review),
                    Some(result.interval),
                    Some(result.ease_factor),
                )
                .await;
        }
        _ => {}
    }

    app.review_index += 1;
    app.review_show_answer = false;

    if app.review_index >= app.review_items.len() {
        app.set_status(format!(
            "Review complete! {} items reviewed.",
            app.review_items.len()
        ));
    }
}
```

`crates/engai/src/tui/panel_read.rs`:
```rust
use crossterm::event::KeyCode;

use super::app::{App, ReadingDetail};
use crate::state::AppState;

pub async fn load_readings(state: &AppState, app: &mut App) {
    match state.db.list_readings(100, 0).await {
        Ok(readings) => app.readings = readings,
        Err(e) => app.set_status(format!("Failed to load readings: {}", e)),
    }
}

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    if app.reading_detail.is_some() {
        return; // Esc handled in mod.rs
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
```

`crates/engai/src/tui/panel_chat.rs`:
```rust
use crossterm::event::KeyCode;

use super::app::App;
use crate::state::AppState;
use engai_core::ai::ChatMessage;

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if app.chat_loading || app.chat_input.trim().is_empty() {
                return;
            }

            let input = app.chat_input.trim().to_string();
            app.chat_input.clear();

            if let Err(e) = state.db.add_chat_message("user", &input).await {
                app.chat_error = Some(format!("DB error: {}", e));
                return;
            }

            // Reload chat messages to show user message
            match state.db.get_recent_chat(50).await {
                Ok(msgs) => {
                    app.chat_messages = msgs.into_iter().rev().collect();
                }
                Err(_) => {}
            }

            app.chat_loading = true;

            let recent = state.db.get_recent_chat(20).await.unwrap_or_default();
            let messages: Vec<ChatMessage> = recent
                .iter()
                .map(|r| ChatMessage {
                    role: r.role.clone(),
                    content: r.content.clone(),
                })
                .collect();

            let ai = state.ai_client.clone();
            match ai.chat_completion(messages).await {
                Ok(response) => {
                    let _ = state.db.add_chat_message("assistant", &response).await;
                    app.chat_error = None;
                }
                Err(e) => {
                    app.chat_error = Some(format!("AI error: {}", e));
                }
            }

            // Reload to show assistant message
            match state.db.get_recent_chat(50).await {
                Ok(msgs) => {
                    app.chat_messages = msgs.into_iter().rev().collect();
                }
                Err(_) => {}
            }

            app.chat_loading = false;
        }
        KeyCode::Char(c) => {
            app.chat_input.push(c);
        }
        KeyCode::Backspace => {
            app.chat_input.pop();
        }
        _ => {}
    }
}
```

`crates/engai/src/tui/panel_stats.rs`:
```rust
use crossterm::event::KeyCode;

use super::app::{App, StatsData};
use crate::state::AppState;

pub async fn load_stats(state: &AppState, app: &mut App) {
    let db = &state.db;
    let word_count = db.word_count().await.unwrap_or(0);
    let phrase_count = db.phrase_count().await.unwrap_or(0);
    let pending = db.pending_review_count().await.unwrap_or(0);
    let today = db.review_count_today().await.unwrap_or(0);

    app.stats = Some(StatsData {
        word_count,
        phrase_count,
        pending_reviews: pending,
        reviewed_today: today,
    });
}

pub async fn handle_key(app: &mut App, state: &AppState, code: KeyCode) {
    if code == KeyCode::Char('r') {
        load_stats(state, app).await;
        app.set_status("Stats refreshed");
    }
}
```

- [ ] **Step 6: Add `mod tui;` to main.rs and verify compilation**

Add `mod tui;` to `crates/engai/src/main.rs` module declarations.

Run: `cargo check -p engai`
Expected: Compiles

- [ ] **Step 7: Commit**

```bash
git add crates/engai/src/tui/ crates/engai/Cargo.toml crates/engai/Cargo.lock crates/engai/src/main.rs
git commit -m "feat(tui): add ratatui TUI with sidebar navigation, vocab, review, read, chat, stats panels"
```

---

### Task 3: Dual-Mode Launch (Web + TUI Concurrently)

**Files:**
- Modify: `crates/engai/src/main.rs`

- [ ] **Step 1: Modify main.rs to run Web + TUI concurrently when no subcommand is given**

Update the `None =>` match arm in `crates/engai/src/main.rs` to spawn both the web server and TUI concurrently:

```rust
None => {
    let config = engai_core::config::Config::load_global()?;
    let db_path = config.db_path();
    let db = engai_core::db::Db::new(&db_path).await?;
    let state = crate::state::AppState::new(std::sync::Arc::new(db), config.clone())?;
    let tui_state = state.clone();

    let port = config.server.port;
    let server_handle = tokio::spawn(async move {
        if let Err(e) = crate::server::run_server(state, port).await {
            tracing::error!("Server error: {}", e);
        }
    });

    let tui_handle = tokio::spawn(async move {
        if let Err(e) = crate::tui::run_tui(tui_state).await {
            tracing::error!("TUI error: {}", e);
        }
    });

    tokio::select! {
        _ = tui_handle => {
            tracing::info!("TUI exited, shutting down server");
        }
        _ = server_handle => {
            tracing::info!("Server exited, shutting down TUI");
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p engai`
Expected: Compiles

- [ ] **Step 3: Run all existing tests**

Run: `cargo test -p engai-core`
Expected: All 27 tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/engai/src/main.rs
git commit -m "feat: dual-mode launch - Web + TUI run concurrently when no subcommand"
```

---

### Task 4: Panel-Specific Navigation and Tab Refinements

**Files:**
- Modify: `crates/engai/src/tui/ui.rs` (update status bar hints)

- [ ] **Step 1: Update status bar hints to reflect `[`/`]` navigation**

In `crates/engai/src/tui/ui.rs`, update the `render_status` function hints:

```rust
fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let msg = if let Some(ref status) = app.status_message {
        status.clone()
    } else {
        let hints = match app.panel {
            Panel::Vocab => "[Up/Down] Scroll [Enter] Detail [Tab] Switch [[/]] Nav [q] Quit".to_string(),
            Panel::Review => "[Space] Flip [0-5] Rate [n] Skip [q] Quit".to_string(),
            Panel::Read => "[Up/Down] Scroll [Enter] Detail [q] Quit".to_string(),
            Panel::Chat => "[Enter] Send [q] Quit".to_string(),
            Panel::Stats => "[r] Refresh [q] Quit".to_string(),
        };
        hints
    };
    let status = Paragraph::new(Span::styled(
        format!(" {}", msg),
        Style::default().fg(Color::DarkGray).bg(Color::Black),
    ));
    f.render_widget(status, area);
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p engai`

- [ ] **Step 3: Run all tests**

Run: `cargo test -p engai-core`
Expected: All 27 tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/engai/src/tui/
git commit -m "feat(tui): update navigation hints and use [/] for panel switching"
```

---

### Task 5: TUI Chat AI Explain (Inline) + Review Polish

**Files:**
- Modify: `crates/engai/src/tui/panel_vocab.rs`
- Modify: `crates/engai/src/tui/panel_review.rs`

- [ ] **Step 1: Add AI explain in vocab detail view**

Add an 'e' key handler to `panel_vocab.rs` that triggers AI explanation in the vocab detail view. In `handle_key`, when `app.vocab_detail.is_some()`:

```rust
if app.vocab_detail.is_some() {
    match code {
        KeyCode::Char('e') => {
            let detail = app.vocab_detail.clone().unwrap();
            app.set_status("Requesting AI explanation...".to_string());
            let ai = state.ai_client.clone();
            let prompt = state.prompt_engine.clone();
            let result = match &detail {
                VocabDetail::Word(w) => ai.explain_word(&w.word, &prompt).await,
                VocabDetail::Phrase(p) => ai.explain_phrase(&p.phrase, &prompt).await,
            };
            match result {
                Ok(explanation) => {
                    app.set_status("AI explanation received".to_string());
                    tracing::info!("AI explanation: {}", explanation.chars().take(100).collect::<String>());
                }
                Err(e) => app.set_status(format!("AI error: {}", e)),
            }
        }
        _ => return,
    }
    return;
}
```

Note: Since ratatui is not designed for long-running async operations blocking the UI, we keep the AI call simple (non-streaming). The status bar shows feedback. A more advanced approach would use channels, but that's beyond Phase 3 scope.

- [ ] **Step 2: Add 'n' key for skip in review panel**

In `panel_review.rs`, add to `handle_key`:
```rust
KeyCode::Char('n') => {
    if app.review_index < app.review_items.len() {
        app.review_index += 1;
        app.review_show_answer = false;
        app.set_status("Skipped");
    }
}
```

- [ ] **Step 3: Verify compilation and tests**

Run: `cargo check -p engai`
Run: `cargo test -p engai-core`

- [ ] **Step 4: Commit**

```bash
git add crates/engai/src/tui/
git commit -m "feat(tui): AI explain in vocab detail and skip in review"
```

---

### Task 6: Polish - Logging, Error Handling, Edge Cases

**Files:**
- Modify: `crates/engai/src/main.rs`

- [ ] **Step 1: Improve main.rs error handling with user-friendly messages**

Update the `main` function in `crates/engai/src/main.rs` to catch and display errors gracefully:

```rust
#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "engai=info");
    }
    tracing_subscriber::fmt::init();

    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { target }) => cmd_add::run(target).await?,
        Some(Commands::Explain { target }) => cmd_explain::run(target).await?,
        Some(Commands::Review { all }) => cmd_review::run(all).await?,
        Some(Commands::Sync) => cmd_sync::run().await?,
        Some(Commands::Read { file }) => cmd_read::run(&file).await?,
        Some(Commands::Import { path }) => cmd_import::run(&path).await?,
        Some(Commands::Export { word, phrase, all }) => {
            cmd_export::run(word, phrase, all).await?
        }
        Some(Commands::Stats) => cmd_stats::run().await?,
        Some(Commands::Config { action }) => cmd_config::run(action).await?,
        Some(Commands::Note { action }) => cmd_note::run(action).await?,
        Some(Commands::Server { port }) => {
            let config = engai_core::config::Config::load_global()?;
            let db_path = config.db_path();
            let db = engai_core::db::Db::new(&db_path).await?;
            let state = crate::state::AppState::new(std::sync::Arc::new(db), config)?;
            crate::server::run_server(state, port).await?;
        }
        None => {
            let config = engai_core::config::Config::load_global()?;
            let db_path = config.db_path();
            let db = engai_core::db::Db::new(&db_path).await?;
            let state = crate::state::AppState::new(std::sync::Arc::new(db), config.clone())?;
            let tui_state = state.clone();

            let port = config.server.port;
            let server_handle = tokio::spawn(async move {
                if let Err(e) = crate::server::run_server(state, port).await {
                    tracing::error!("Server error: {}", e);
                }
            });

            let tui_handle = tokio::spawn(async move {
                if let Err(e) = crate::tui::run_tui(tui_state).await {
                    tracing::error!("TUI error: {}", e);
                }
            });

            tokio::select! {
                _ = tui_handle => {
                    tracing::info!("TUI exited, shutting down server");
                }
                _ = server_handle => {
                    tracing::info!("Server exited, shutting down TUI");
                }
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -p engai -- -D warnings`
Expected: No warnings

- [ ] **Step 3: Run all tests**

Run: `cargo test -p engai-core`

- [ ] **Step 4: Build release binary**

Run: `cargo build -p engai`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add crates/engai/src/main.rs
git commit -m "polish: improve error handling, default log level, and graceful shutdown"
```

---

### Task 7: Final Verification

**Files:** (no file changes)

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: All 27+ tests pass

- [ ] **Step 2: Run clippy on full workspace**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

- [ ] **Step 3: Verify binary builds**

Run: `cargo build`
Expected: `engai` binary produced in `target/debug/engai.exe`

- [ ] **Step 4: Quick functional smoke test**

Run: `engai --help`
Expected: Shows CLI help with subcommands

Run: `engai stats`
Expected: Shows stats (word count, etc.)

- [ ] **Step 5: Final commit (if any fixes were needed)**

```bash
git add -A
git commit -m "fix: address clippy warnings and test fixes"
```
