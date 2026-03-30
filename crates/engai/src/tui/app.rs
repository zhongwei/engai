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
    pub const ALL: [Panel; 5] = [
        Panel::Vocab,
        Panel::Review,
        Panel::Read,
        Panel::Chat,
        Panel::Stats,
    ];

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
