use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message as LettreMessage, SmtpTransport, Transport};
use std::collections::HashMap;
use tokio::sync::Mutex;

use super::{Account, MailError, MailResult, Message};

pub struct MailClient {
    accounts: Vec<Account>,
    connections: Mutex<HashMap<String, Connection>>,
}

#[derive(Debug)]
struct Connection {
    account_id: String,
    // 実際の接続情報は後で追加
    // imap_session: Option<ImapSession>,
    // smtp_session: Option<SmtpSession>,
}

impl MailClient {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_account(&mut self, account: Account) -> MailResult<()> {
        account.validate().map_err(MailError::Parse)?;

        self.accounts.push(account);
        Ok(())
    }

    pub fn remove_account(&mut self, account_id: &str) -> MailResult<()> {
        let index = self
            .accounts
            .iter()
            .position(|a| a.id == account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        self.accounts.remove(index);
        Ok(())
    }

    pub fn get_account(&self, account_id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == account_id)
    }

    pub fn get_accounts(&self) -> &[Account] {
        &self.accounts
    }

    pub async fn connect(&self, account_id: &str) -> MailResult<()> {
        let _account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        // TODO: 実際のIMAP/SMTP接続を実装
        let connection = Connection {
            account_id: account_id.to_string(),
        };

        let mut connections = self.connections.lock().await;
        connections.insert(account_id.to_string(), connection);

        Ok(())
    }

    pub async fn disconnect(&self, account_id: &str) -> MailResult<()> {
        let mut connections = self.connections.lock().await;
        connections.remove(account_id);
        Ok(())
    }

    pub async fn fetch_messages(
        &self,
        account_id: &str,
        folder: &str,
        limit: Option<usize>,
    ) -> MailResult<Vec<Message>> {
        let _account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        // TODO: 実際のIMAP接続を使ってメッセージを取得
        // 現在はダミーデータを返す
        let dummy_messages = self.create_dummy_messages(account_id, folder, limit.unwrap_or(50));
        Ok(dummy_messages)
    }

    pub async fn fetch_message_body(
        &self,
        account_id: &str,
        message_id: &str,
    ) -> MailResult<String> {
        let _account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        // TODO: 実際のメッセージ本文取得を実装
        Ok(format!(
            "これは {} のメッセージ本文です。\n\n実際のメール内容がここに表示されます。",
            message_id
        ))
    }

    pub async fn send_message(&self, account_id: &str, message: &Message) -> MailResult<()> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        let smtp_config = &account.smtp;

        // Lettreメッセージを構築
        let mut builder = LettreMessage::builder()
            .from(
                format!("{} <{}>", account.name, account.email)
                    .parse()
                    .map_err(|e| MailError::Parse(format!("Invalid from address: {}", e)))?,
            )
            .subject(&message.subject);

        // To recipients
        for to in &message.to {
            builder = builder.to(to
                .email
                .parse()
                .map_err(|e| MailError::Parse(format!("Invalid to address: {}", e)))?);
        }

        // CC recipients
        for cc in &message.cc {
            builder = builder.cc(cc
                .email
                .parse()
                .map_err(|e| MailError::Parse(format!("Invalid cc address: {}", e)))?);
        }

        // BCC recipients
        for bcc in &message.bcc {
            builder = builder.bcc(
                bcc.email
                    .parse()
                    .map_err(|e| MailError::Parse(format!("Invalid bcc address: {}", e)))?,
            );
        }

        let email = builder
            .body(message.body.get_display_content())
            .map_err(|e| MailError::Parse(format!("Failed to build message: {}", e)))?;

        // SMTP認証情報
        let creds = Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());

        // SMTP接続
        let mailer = if smtp_config.use_tls {
            SmtpTransport::relay(&smtp_config.server)
                .map_err(|e| MailError::Connection(format!("SMTP relay error: {}", e)))?
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&smtp_config.server)
                .port(smtp_config.port)
                .credentials(creds)
                .build()
        };

        // メール送信
        mailer
            .send(&email)
            .map_err(|e| MailError::Protocol(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    pub async fn move_message(
        &self,
        account_id: &str,
        message_id: &str,
        from_folder: &str,
        to_folder: &str,
    ) -> MailResult<()> {
        let _account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        // TODO: 実際のメッセージ移動を実装
        println!(
            "Moving message {} from {} to {}",
            message_id, from_folder, to_folder
        );
        Ok(())
    }

    pub async fn delete_message(&self, account_id: &str, message_id: &str) -> MailResult<()> {
        let _account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        // TODO: 実際のメッセージ削除を実装
        println!("Deleting message {}", message_id);
        Ok(())
    }

    pub async fn mark_as_read(&self, account_id: &str, message_id: &str) -> MailResult<()> {
        let _account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        // TODO: 実際のフラグ設定を実装
        println!("Marking message {} as read", message_id);
        Ok(())
    }

    pub async fn get_folders(&self, account_id: &str) -> MailResult<Vec<String>> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        // アカウント設定からフォルダ一覧を取得
        let folders = account
            .imap
            .folders
            .iter()
            .map(|f| f.local_name.clone())
            .collect();

        Ok(folders)
    }

    // デバッグ用のダミーメッセージ作成
    fn create_dummy_messages(&self, account_id: &str, folder: &str, limit: usize) -> Vec<Message> {
        use super::{Address, MessageBody};
        use chrono::{Duration, Utc};

        let mut messages = Vec::new();
        for i in 0..limit.min(10) {
            let date = Utc::now() - Duration::days(i as i64);
            let from = vec![Address::new(
                format!("sender{}@example.com", i),
                Some(format!("Sender {}", i)),
            )];
            let to = vec![Address::new(
                "user@example.com".to_string(),
                Some("User".to_string()),
            )];

            let subject = match i % 5 {
                0 => "重要：会議の件について",
                1 => "プロジェクトの進捗報告",
                2 => "新着情報のお知らせ",
                3 => "システムメンテナンスのご案内",
                _ => "その他のお知らせ",
            }
            .to_string();

            let body = MessageBody::new_plain(format!(
                "これは {} のメッセージ本文です。\n\n件名: {}\n\n詳細な内容がここに表示されます。",
                i + 1,
                subject
            ));

            let mut message = Message::new(
                format!("msg-{}", i),
                from,
                to,
                subject,
                body,
                account_id.to_string(),
                folder.to_string(),
            );
            message.date = date;

            // 一部のメッセージを未読にする
            if i % 3 == 0 {
                message.mark_as_read();
            }

            messages.push(message);
        }

        messages
    }
}

impl Default for MailClient {
    fn default() -> Self {
        Self::new()
    }
}
