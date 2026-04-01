use anyhow::Result;
use reqwest::Client;

use super::models::{ChatEntry, Phrase, Reading, ReviewEntry, ReviewResult, StatsData, Word};

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let mut url = base_url;
        if url.ends_with('/') {
            url.pop();
        }
        Self {
            base_url: url,
            client: Client::new(),
        }
    }

    pub async fn list_words(&self, limit: i64, offset: i64) -> Result<Vec<Word>> {
        let url = format!("{}/api/words?limit={}&offset={}", self.base_url, limit, offset);
        let words = self.client.get(&url).send().await?.json().await?;
        Ok(words)
    }

    pub async fn get_word(&self, word: &str) -> Result<Word> {
        let url = format!("{}/api/words/{}", self.base_url, word);
        let word = self.client.get(&url).send().await?.json().await?;
        Ok(word)
    }

    pub async fn explain_word(&self, word: &str) -> Result<String> {
        let url = format!("{}/api/words/{}/explain", self.base_url, word);
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        Ok(text)
    }

    pub async fn list_phrases(&self, limit: i64, offset: i64) -> Result<Vec<Phrase>> {
        let url = format!("{}/api/phrases?limit={}&offset={}", self.base_url, limit, offset);
        let phrases = self.client.get(&url).send().await?.json().await?;
        Ok(phrases)
    }

    pub async fn get_phrase(&self, id: i64) -> Result<Phrase> {
        let url = format!("{}/api/phrases/{}", self.base_url, id);
        let phrase = self.client.get(&url).send().await?.json().await?;
        Ok(phrase)
    }

    pub async fn explain_phrase(&self, id: i64) -> Result<String> {
        let url = format!("{}/api/phrases/{}/explain", self.base_url, id);
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        Ok(text)
    }

    pub async fn today_reviews(&self) -> Result<Vec<ReviewEntry>> {
        let url = format!("{}/api/reviews/today", self.base_url);
        let entries = self.client.get(&url).send().await?.json().await?;
        Ok(entries)
    }

    pub async fn submit_review(&self, target_type: &str, id: i64, quality: i32) -> Result<ReviewResult> {
        let url = format!("{}/api/reviews/{}/{}", self.base_url, target_type, id);
        let result = self.client
            .post(&url)
            .json(&serde_json::json!({ "quality": quality }))
            .send()
            .await?
            .json()
            .await?;
        Ok(result)
    }

    pub async fn list_readings(&self, limit: i64, offset: i64) -> Result<Vec<Reading>> {
        let url = format!("{}/api/readings?limit={}&offset={}", self.base_url, limit, offset);
        let readings = self.client.get(&url).send().await?.json().await?;
        Ok(readings)
    }

    pub async fn get_reading(&self, id: i64) -> Result<Reading> {
        let url = format!("{}/api/readings/{}", self.base_url, id);
        let reading = self.client.get(&url).send().await?.json().await?;
        Ok(reading)
    }

    pub async fn analyze_reading(&self, id: i64) -> Result<String> {
        let url = format!("{}/api/readings/{}/analyze", self.base_url, id);
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        Ok(text)
    }

    pub async fn get_stats(&self) -> Result<StatsData> {
        let url = format!("{}/api/stats", self.base_url);
        let stats = self.client.get(&url).send().await?.json().await?;
        Ok(stats)
    }

    pub async fn get_chat_history(&self, limit: i64) -> Result<Vec<ChatEntry>> {
        let url = format!("{}/api/chat/history?limit={}", self.base_url, limit);
        let entries = self.client.get(&url).send().await?.json().await?;
        Ok(entries)
    }

    pub async fn chat(&self, content: &str) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);
        let resp = self.client
            .post(&url)
            .json(&serde_json::json!({ "content": content }))
            .send()
            .await?;
        let text = resp.text().await?;
        Ok(text)
    }
}
