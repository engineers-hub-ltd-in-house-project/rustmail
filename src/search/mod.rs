// 検索機能の実装

use crate::mail::Message;

pub struct SearchEngine {
    // 将来的にインデックスなどを保持
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn search<'a>(&self, query: &str, messages: &'a [Message]) -> Vec<&'a Message> {
        // 簡単なテキスト検索の実装
        messages
            .iter()
            .filter(|message| {
                message
                    .subject
                    .to_lowercase()
                    .contains(&query.to_lowercase())
                    || message
                        .body
                        .get_display_content()
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                    || message
                        .get_sender_display()
                        .to_lowercase()
                        .contains(&query.to_lowercase())
            })
            .collect()
    }

    pub fn search_by_sender<'a>(&self, sender: &str, messages: &'a [Message]) -> Vec<&'a Message> {
        messages
            .iter()
            .filter(|message| {
                message
                    .get_sender_display()
                    .to_lowercase()
                    .contains(&sender.to_lowercase())
            })
            .collect()
    }

    pub fn search_by_subject<'a>(
        &self,
        subject: &str,
        messages: &'a [Message],
    ) -> Vec<&'a Message> {
        messages
            .iter()
            .filter(|message| {
                message
                    .subject
                    .to_lowercase()
                    .contains(&subject.to_lowercase())
            })
            .collect()
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
