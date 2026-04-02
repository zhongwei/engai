use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, FocusZone, Panel, VocabDetail, VocabTab};

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

    let main = Layout::horizontal([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)])
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
        Span::styled(
            " Engai ",
            Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
        ),
        Span::raw(" - AI English Learning System"),
    ]);
    f.render_widget(
        Paragraph::new(title).style(Style::default().bg(Color::DarkGray)),
        area,
    );
}

fn render_sidebar(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == FocusZone::Sidebar;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let items: Vec<ListItem> = Panel::ALL
        .iter()
        .map(|&panel| {
            let label = panel.label();
            let selected = app.panel == panel;
            let style = if selected && focused {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let prefix = if selected && focused { ">" } else { " " };
            ListItem::new(Span::styled(format!(" {}{}", prefix, label), style))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(border_style),
    );
    f.render_widget(list, area);
}

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let msg = if let Some(ref status) = app.status_message {
        status.clone()
    } else if app.focus == FocusZone::Sidebar {
        "[Up/Down/j/k] Nav [Tab/Enter] Focus content [q] Quit".to_string()
    } else {
        match app.panel {
            Panel::Vocab => {
                "[Up/Down] Scroll [Enter] Detail [Tab] Switch [Esc] Sidebar [q] Quit".to_string()
            }
            Panel::Review => "[Space] Flip [0-5] Rate [n] Skip [Esc] Sidebar [q] Quit".to_string(),
            Panel::Read => "[Up/Down] Scroll [Enter] Detail [Esc] Sidebar [q] Quit".to_string(),
            Panel::Chat => "[Enter] Send [Esc] Sidebar [q] Quit".to_string(),
            Panel::Stats => "[r] Refresh [Esc] Sidebar [q] Quit".to_string(),
        }
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
                    let meaning = w
                        .meaning
                        .as_deref()
                        .unwrap_or("—")
                        .chars()
                        .take(30)
                        .collect::<String>();
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
                    let meaning = p
                        .meaning
                        .as_deref()
                        .unwrap_or("—")
                        .chars()
                        .take(25)
                        .collect::<String>();
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
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
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
                    format!(
                        "  Familiarity: {} / Interval: {} days",
                        w.familiarity, w.interval
                    ),
                    Style::default().fg(Color::Yellow),
                ),
                Line::from(format!("  Ease Factor: {:.2}", w.ease_factor)),
            ]
        }
        VocabDetail::Phrase(p) => {
            vec![
                Line::styled(
                    format!("  {}", p.phrase),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Line::from(""),
                Line::styled("  Meaning", Style::default().add_modifier(Modifier::BOLD)),
                Line::from(format!("  {}", p.meaning.as_deref().unwrap_or("(not set)"))),
                Line::from(""),
                Line::styled(
                    format!(
                        "  Familiarity: {} / Interval: {} days",
                        p.familiarity, p.interval
                    ),
                    Style::default().fg(Color::Yellow),
                ),
                Line::from(format!("  Ease Factor: {:.2}", p.ease_factor)),
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
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .centered()
            .block(Block::default().borders(Borders::ALL).title(" Review "));
        f.render_widget(para, area);
        return;
    }

    if app.review_index >= app.review_items.len() {
        let total = app.review_items.len();
        let para = Paragraph::new(format!("Review complete! {} items reviewed.", total))
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .centered()
            .block(Block::default().borders(Borders::ALL).title(" Review "));
        f.render_widget(para, area);
        return;
    }

    let item = &app.review_items[app.review_index];
    let progress = format!("{}/{}", app.review_index + 1, app.review_items.len());

    let inner = area.inner(Margin::new(2, 1));

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(4),
        Constraint::Length(3),
    ])
    .split(inner);

    let progress_line = Paragraph::new(format!(
        "Progress: {}  |  Familiarity: {}  |  [Space] Flip  [1-5] Rate  [n] Skip",
        progress, item.familiarity
    ))
    .style(Style::default().fg(Color::Yellow));
    f.render_widget(progress_line, chunks[0]);

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
            Line::styled(format!("  {}", meaning), Style::default().fg(Color::White)),
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

    let legend = Paragraph::new("0=Again  1=Hard  2=Difficult  3=OK  4=Easy  5=Perfect")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(legend, chunks[2]);

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
            let title = r
                .title
                .as_deref()
                .unwrap_or("(untitled)")
                .chars()
                .take(40)
                .collect::<String>();
            let preview = r.content.chars().take(50).collect::<String>();
            let line = format!(
                " {}  {}  {}",
                title,
                preview,
                r.source.as_deref().unwrap_or("")
            );
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
    let title = detail.reading.title.as_deref().unwrap_or("(untitled)");

    let mut lines = vec![
        Line::styled(
            format!("  {}", title),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
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
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
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
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(area);

    let messages: Vec<Line> = app
        .chat_messages
        .iter()
        .flat_map(|msg| {
            let role_style = match msg.role.as_str() {
                "user" => Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                "assistant" => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
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
                        Line::from(vec![Span::styled(prefix, role_style), Span::raw(l)])
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
    let input =
        Paragraph::new(input_text).block(Block::default().borders(Borders::ALL).title(" Input "));
    f.render_widget(input, chunks[1]);

    if app.chat_loading {
        f.set_cursor_position((chunks[1].x + 2, chunks[1].y + 1));
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
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Stats [r] Refresh "),
                );
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
        Line::from(format!("  Pending Reviews:    {}", data.pending_reviews)),
        Line::from(format!("  Reviewed Today:     {}", data.reviewed_today)),
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
