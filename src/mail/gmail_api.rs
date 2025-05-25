use crate::mail::{Account, Address, Flag, MailError, MailResult, Message, MessageBody};
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use reqwest;
use serde::{Deserialize, Serialize};

const GMAIL_API_BASE_URL: &str = "https://www.googleapis.com/gmail/v1";

#[derive(Debug, Deserialize)]
struct GmailProfile {
    #[serde(rename = "emailAddress")]
    email_address: String,
    #[serde(rename = "messagesTotal")]
    messages_total: u64,
    #[serde(rename = "threadsTotal")]
    threads_total: u64,
    #[serde(rename = "historyId")]
    history_id: String,
}

#[derive(Debug, Deserialize)]
struct GmailMessageList {
    messages: Option<Vec<GmailMessageRef>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    #[serde(rename = "resultSizeEstimate")]
    result_size_estimate: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GmailMessageRef {
    id: String,
    #[serde(rename = "threadId")]
    thread_id: String,
}

#[derive(Debug, Deserialize)]
struct GmailMessage {
    id: String,
    #[serde(rename = "threadId")]
    thread_id: String,
    #[serde(rename = "labelIds")]
    label_ids: Option<Vec<String>>,
    snippet: Option<String>,
    payload: Option<GmailPayload>,
    #[serde(rename = "internalDate")]
    internal_date: Option<String>,
    #[serde(rename = "historyId")]
    history_id: Option<String>,
    #[serde(rename = "sizeEstimate")]
    size_estimate: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct GmailPayload {
    #[serde(rename = "partId")]
    part_id: Option<String>,
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    filename: Option<String>,
    headers: Option<Vec<GmailHeader>>,
    body: Option<GmailBody>,
    parts: Option<Vec<GmailPayload>>,
}

#[derive(Debug, Deserialize)]
struct GmailHeader {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct GmailBody {
    #[serde(rename = "attachmentId")]
    attachment_id: Option<String>,
    size: Option<u32>,
    data: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GmailLabels {
    labels: Option<Vec<GmailLabel>>,
}

#[derive(Debug, Deserialize)]
struct GmailLabel {
    id: String,
    name: String,
    #[serde(rename = "messageListVisibility")]
    message_list_visibility: Option<String>,
    #[serde(rename = "labelListVisibility")]
    label_list_visibility: Option<String>,
    #[serde(rename = "type")]
    label_type: Option<String>,
}

pub struct GmailApiClient {
    account: Account,
    http_client: reqwest::Client,
}

impl GmailApiClient {
    pub fn new(account: Account) -> Self {
        Self {
            account,
            http_client: reqwest::Client::new(),
        }
    }

    /// 接続テスト（プロフィール取得）
    pub async fn connect(&self) -> MailResult<()> {
        let access_token = self.get_access_token()?;

        println!("デバッグ: Gmail API接続テスト中...");

        let url = format!("{}/users/me/profile", GMAIL_API_BASE_URL);
        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| MailError::Connection(format!("Gmail API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(MailError::Connection(format!(
                "Gmail API connection failed: {} - {}",
                status, error_text
            )));
        }

        let profile: GmailProfile = response
            .json()
            .await
            .map_err(|e| MailError::Connection(format!("Failed to parse Gmail profile: {}", e)))?;

        println!("デバッグ: Gmail API接続成功");
        println!("  メールアドレス: {}", profile.email_address);
        println!("  メッセージ総数: {}", profile.messages_total);
        println!("  スレッド総数: {}", profile.threads_total);

        Ok(())
    }

    /// フォルダー一覧を取得（ラベル一覧）
    pub async fn list_folders(&self) -> MailResult<Vec<String>> {
        let access_token = self.get_access_token()?;

        let url = format!("{}/users/me/labels", GMAIL_API_BASE_URL);
        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| MailError::Protocol(format!("Gmail labels request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(MailError::Protocol(format!(
                "Gmail labels request failed: {} - {}",
                status, error_text
            )));
        }

        let labels: GmailLabels = response
            .json()
            .await
            .map_err(|e| MailError::Protocol(format!("Failed to parse Gmail labels: {}", e)))?;

        let folder_names = labels
            .labels
            .unwrap_or_default()
            .into_iter()
            .map(|label| label.name)
            .collect();

        Ok(folder_names)
    }

    /// メッセージ一覧を取得
    pub async fn fetch_messages(
        &self,
        folder_name: &str,
        limit: Option<usize>,
    ) -> MailResult<Vec<Message>> {
        let access_token = self.get_access_token()?;

        // Gmail APIではラベルIDでフィルタリング
        let label_id = self.convert_folder_to_label_id(folder_name);

        let mut url = format!("{}/users/me/messages", GMAIL_API_BASE_URL);
        let mut params = vec![];

        if let Some(label) = label_id {
            params.push(format!("labelIds={}", label));
        }

        if let Some(limit) = limit {
            params.push(format!("maxResults={}", limit));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        println!("デバッグ: Gmail API メッセージ一覧取得中... URL: {}", url);

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| MailError::Protocol(format!("Gmail messages request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(MailError::Protocol(format!(
                "Gmail messages request failed: {} - {}",
                status, error_text
            )));
        }

        let message_list: GmailMessageList = response.json().await.map_err(|e| {
            MailError::Protocol(format!("Failed to parse Gmail message list: {}", e))
        })?;

        let message_refs = message_list.messages.unwrap_or_default();
        println!(
            "デバッグ: {} 件のメッセージ参照を取得しました",
            message_refs.len()
        );

        // 各メッセージの詳細を取得
        let mut messages = Vec::new();
        for (i, message_ref) in message_refs.iter().take(limit.unwrap_or(10)).enumerate() {
            println!(
                "デバッグ: メッセージ {}/{} を取得中...",
                i + 1,
                message_refs.len()
            );

            match self
                .fetch_message_details(&message_ref.id, folder_name)
                .await
            {
                Ok(message) => messages.push(message),
                Err(e) => {
                    println!("警告: メッセージ {} の取得に失敗: {}", message_ref.id, e);
                    continue;
                }
            }
        }

        // 最新順にソート
        messages.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(messages)
    }

    /// 個別メッセージの詳細を取得
    async fn fetch_message_details(
        &self,
        message_id: &str,
        folder_name: &str,
    ) -> MailResult<Message> {
        let access_token = self.get_access_token()?;

        let url = format!("{}/users/me/messages/{}", GMAIL_API_BASE_URL, message_id);
        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| MailError::Protocol(format!("Gmail message request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(MailError::Protocol(format!(
                "Gmail message request failed: {} - {}",
                status, error_text
            )));
        }

        let gmail_message: GmailMessage = response
            .json()
            .await
            .map_err(|e| MailError::Protocol(format!("Failed to parse Gmail message: {}", e)))?;

        self.convert_gmail_message_to_message(gmail_message, folder_name)
    }

    /// GmailMessageをMessageに変換
    fn convert_gmail_message_to_message(
        &self,
        gmail_message: GmailMessage,
        folder_name: &str,
    ) -> MailResult<Message> {
        let payload = gmail_message
            .payload
            .as_ref()
            .ok_or_else(|| MailError::Protocol("Message payload not found".to_string()))?;

        let headers = payload
            .headers
            .as_ref()
            .ok_or_else(|| MailError::Protocol("Message headers not found".to_string()))?;

        // ヘッダーから情報を抽出
        let mut subject = String::new();
        let mut from_header = String::new();
        let mut to_header = String::new();
        let mut date_header = String::new();

        for header in headers {
            match header.name.to_lowercase().as_str() {
                "subject" => subject = header.value.clone(),
                "from" => from_header = header.value.clone(),
                "to" => to_header = header.value.clone(),
                "date" => date_header = header.value.clone(),
                _ => {}
            }
        }

        // 送信者をパース
        let from = vec![Address::new(from_header.clone(), None)]; // 簡易実装

        // 受信者をパース
        let to = if to_header.is_empty() {
            vec![]
        } else {
            vec![Address::new(to_header.clone(), None)] // 簡易実装
        };

        // 日付をパース
        let date = if date_header.is_empty() {
            Utc::now()
        } else {
            DateTime::parse_from_rfc2822(&date_header)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now())
        };

        // メッセージ本文
        let body_text = gmail_message.snippet.unwrap_or_default();
        let body = MessageBody::new_plain(body_text);

        // フラグ（ラベルから推測）
        let mut flags = Vec::new();
        if let Some(label_ids) = &gmail_message.label_ids {
            for label_id in label_ids {
                match label_id.as_str() {
                    "UNREAD" => {}               // 既読フラグなし
                    _ => flags.push(Flag::Seen), // その他は既読扱い
                }
            }
        }

        let mut message = Message::new(
            gmail_message.id,
            from,
            to,
            subject,
            body,
            self.account.id.clone(),
            folder_name.to_string(),
        );

        message.date = date;
        message.flags = flags;

        Ok(message)
    }

    /// フォルダー名をGmailラベルIDに変換
    fn convert_folder_to_label_id(&self, folder_name: &str) -> Option<String> {
        match folder_name {
            "INBOX" | "受信箱" => Some("INBOX".to_string()),
            "Sent" | "送信済み" => Some("SENT".to_string()),
            "Drafts" | "下書き" => Some("DRAFT".to_string()),
            "Trash" | "ゴミ箱" => Some("TRASH".to_string()),
            _ => None,
        }
    }

    /// アクセストークンを取得
    fn get_access_token(&self) -> MailResult<String> {
        Ok(self
            .account
            .tokens
            .as_ref()
            .ok_or_else(|| MailError::Authentication("No OAuth2 tokens available".to_string()))?
            .access_token
            .clone())
    }
}
