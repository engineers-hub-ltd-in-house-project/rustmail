use std::collections::HashMap;
use tokio::sync::Mutex;

use super::oauth::{GoogleOAuthClient, OAuthFlowManager};
use super::{Account, GmailApiClient, ImapClient, MailError, MailResult, Message, SmtpClient};

pub struct MailClient {
    accounts: Vec<Account>,
    imap_connections: Mutex<HashMap<String, ImapClient>>,
    smtp_connections: Mutex<HashMap<String, SmtpClient>>,
    oauth_flow_manager: Mutex<OAuthFlowManager>,
    gmail_api_clients: Mutex<HashMap<String, GmailApiClient>>,
}

impl MailClient {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            imap_connections: Mutex::new(HashMap::new()),
            smtp_connections: Mutex::new(HashMap::new()),
            oauth_flow_manager: Mutex::new(OAuthFlowManager::new()),
            gmail_api_clients: Mutex::new(HashMap::new()),
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

    pub fn get_account_mut(&mut self, account_id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == account_id)
    }

    pub fn get_accounts(&self) -> &[Account] {
        &self.accounts
    }

    /// OAuth2認証フローを開始
    pub async fn start_oauth_flow(&self, account_id: &str) -> MailResult<String> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        let oauth_config = account
            .oauth_config
            .as_ref()
            .ok_or_else(|| MailError::Authentication("No OAuth config".to_string()))?;

        let oauth_client = GoogleOAuthClient::new(oauth_config.clone()).map_err(|e| {
            MailError::Authentication(format!("OAuth client creation failed: {}", e))
        })?;

        let (auth_url, csrf_token) = oauth_client.get_authorization_url();

        let mut flow_manager = self.oauth_flow_manager.lock().await;
        flow_manager.start_flow(account_id.to_string(), csrf_token);

        Ok(auth_url.to_string())
    }

    /// OAuth2認証コードを処理
    pub async fn handle_oauth_callback(
        &mut self,
        account_id: &str,
        authorization_code: String,
        state: String,
    ) -> MailResult<()> {
        // 必要な情報を先に取得
        let oauth_config = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?
            .oauth_config
            .as_ref()
            .ok_or_else(|| MailError::Authentication("No OAuth config".to_string()))?
            .clone();

        // CSRF検証
        {
            let mut flow_manager = self.oauth_flow_manager.lock().await;
            flow_manager
                .validate_and_complete_flow(account_id, &state)
                .map_err(|e| MailError::Authentication(e.to_string()))?;
        }

        // トークン取得
        let oauth_client = GoogleOAuthClient::new(oauth_config).map_err(|e| {
            MailError::Authentication(format!("OAuth client creation failed: {}", e))
        })?;

        let tokens = oauth_client
            .exchange_code_for_token(authorization_code)
            .await
            .map_err(|e| MailError::Authentication(format!("Token exchange failed: {}", e)))?;

        // アカウントにトークンを保存
        let account = self
            .get_account_mut(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;
        account.tokens = Some(tokens);

        Ok(())
    }

    /// OAuth2トークンを更新
    pub async fn refresh_oauth_token(&mut self, account_id: &str) -> MailResult<()> {
        let account = self
            .get_account_mut(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        let oauth_config = account
            .oauth_config
            .as_ref()
            .ok_or_else(|| MailError::Authentication("No OAuth config".to_string()))?;

        let current_tokens = account
            .tokens
            .as_ref()
            .ok_or_else(|| MailError::Authentication("No tokens available".to_string()))?;

        let refresh_token = current_tokens
            .refresh_token
            .as_ref()
            .ok_or_else(|| MailError::Authentication("No refresh token available".to_string()))?;

        let oauth_client = GoogleOAuthClient::new(oauth_config.clone()).map_err(|e| {
            MailError::Authentication(format!("OAuth client creation failed: {}", e))
        })?;

        let new_tokens = oauth_client
            .refresh_access_token(refresh_token.clone())
            .await
            .map_err(|e| MailError::Authentication(format!("Token refresh failed: {}", e)))?;

        account.tokens = Some(new_tokens);

        Ok(())
    }

    /// IMAPサーバーに接続
    pub async fn connect_imap(&self, account_id: &str) -> MailResult<()> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        // Gmailアカウントの場合は最初からGmail APIクライアントを使用
        if self.is_gmail_account(&account.email) {
            println!(
                "デバッグ: Gmailアカウントを検出しました。Gmail APIクライアントを使用します。"
            );

            let gmail_client = GmailApiClient::new(account.clone());
            match gmail_client.connect().await {
                Ok(_) => {
                    println!("デバッグ: Gmail API接続が成功しました");
                    let mut gmail_clients = self.gmail_api_clients.lock().await;
                    gmail_clients.insert(account_id.to_string(), gmail_client);
                    return Ok(());
                }
                Err(e) => {
                    println!("デバッグ: Gmail API接続に失敗しました: {:?}", e);
                    return Err(MailError::Connection(format!(
                        "Gmail API connection failed: {}",
                        e
                    )));
                }
            }
        }

        // Gmail以外のアカウントの場合はIMAPを使用
        println!("デバッグ: 非Gmailアカウントです。IMAPクライアントを使用します。");
        let mut imap_client = ImapClient::new(account.clone());
        match imap_client.connect().await {
            Ok(_) => {
                println!("デバッグ: IMAP接続が成功しました");
                let mut connections = self.imap_connections.lock().await;
                connections.insert(account_id.to_string(), imap_client);
                Ok(())
            }
            Err(MailError::Authentication(ref msg)) if msg.contains("timeout") => {
                println!("デバッグ: IMAP OAuth2認証がタイムアウトしました。Gmail APIクライアントを試行します。");

                // Gmail APIクライアントを作成して接続テスト
                let gmail_client = GmailApiClient::new(account.clone());
                match gmail_client.connect().await {
                    Ok(_) => {
                        println!("デバッグ: Gmail API接続が成功しました");
                        let mut gmail_clients = self.gmail_api_clients.lock().await;
                        gmail_clients.insert(account_id.to_string(), gmail_client);
                        Ok(())
                    }
                    Err(e) => {
                        println!("デバッグ: Gmail API接続も失敗しました: {:?}", e);
                        Err(MailError::Connection(format!(
                            "IMAP OAuth2 timeout and Gmail API connection also failed: {}",
                            e
                        )))
                    }
                }
            }
            Err(e) => {
                println!("デバッグ: IMAP接続に失敗しました: {:?}", e);
                Err(e)
            }
        }
    }

    /// Gmailアカウントかどうかを判定
    fn is_gmail_account(&self, email: &str) -> bool {
        let email_lower = email.to_lowercase();
        email_lower.ends_with("@gmail.com") || email_lower.ends_with("@googlemail.com")
    }

    /// SMTPサーバーに接続
    pub async fn connect_smtp(&self, account_id: &str) -> MailResult<()> {
        let account = self
            .get_account(account_id)
            .ok_or_else(|| MailError::Parse("Account not found".to_string()))?;

        let mut smtp_client = SmtpClient::new(account.clone());
        smtp_client.connect().await?;

        let mut connections = self.smtp_connections.lock().await;
        connections.insert(account_id.to_string(), smtp_client);

        Ok(())
    }

    /// IMAP接続を切断
    pub async fn disconnect_imap(&self, account_id: &str) -> MailResult<()> {
        let mut connections = self.imap_connections.lock().await;
        if let Some(mut client) = connections.remove(account_id) {
            client.disconnect().await?;
        }
        Ok(())
    }

    /// SMTP接続を切断
    pub async fn disconnect_smtp(&self, account_id: &str) -> MailResult<()> {
        let mut connections = self.smtp_connections.lock().await;
        if let Some(mut client) = connections.remove(account_id) {
            client.disconnect();
        }
        Ok(())
    }

    /// メッセージ一覧を取得
    pub async fn fetch_messages(
        &self,
        account_id: &str,
        folder: &str,
        limit: Option<usize>,
    ) -> MailResult<Vec<Message>> {
        // まずGmail APIクライアントが利用可能かチェック
        {
            let gmail_clients = self.gmail_api_clients.lock().await;
            if let Some(gmail_client) = gmail_clients.get(account_id) {
                println!("デバッグ: Gmail APIクライアントを使用してメッセージを取得します");
                return gmail_client.fetch_messages(folder, limit).await;
            }
        }

        // Gmail APIクライアントがない場合はIMAPクライアントを使用
        let mut connections = self.imap_connections.lock().await;
        let client = connections.get_mut(account_id).ok_or_else(|| {
            MailError::Connection("IMAP not connected and Gmail API not available".to_string())
        })?;

        println!("デバッグ: IMAPクライアントを使用してメッセージを取得します");
        client.fetch_messages(folder, limit).await
    }

    /// メッセージ本文を取得
    pub async fn fetch_message_body(
        &self,
        account_id: &str,
        message_id: &str,
        folder: &str,
    ) -> MailResult<String> {
        let uid: u32 = message_id
            .parse()
            .map_err(|_| MailError::Parse("Invalid message ID".to_string()))?;

        let mut connections = self.imap_connections.lock().await;
        let client = connections
            .get_mut(account_id)
            .ok_or_else(|| MailError::Connection("IMAP not connected".to_string()))?;

        client.fetch_message_body(folder, uid).await
    }

    /// メールを送信
    pub async fn send_message(&self, account_id: &str, message: &Message) -> MailResult<()> {
        let mut connections = self.smtp_connections.lock().await;
        let client = connections
            .get_mut(account_id)
            .ok_or_else(|| MailError::Connection("SMTP not connected".to_string()))?;

        client.send_message(message).await
    }

    /// メッセージを移動
    pub async fn move_message(
        &self,
        account_id: &str,
        message_id: &str,
        from_folder: &str,
        to_folder: &str,
    ) -> MailResult<()> {
        let uid: u32 = message_id
            .parse()
            .map_err(|_| MailError::Parse("Invalid message ID".to_string()))?;

        let mut connections = self.imap_connections.lock().await;
        let client = connections
            .get_mut(account_id)
            .ok_or_else(|| MailError::Connection("IMAP not connected".to_string()))?;

        client.move_message(from_folder, to_folder, uid).await
    }

    /// メッセージを削除
    pub async fn delete_message(
        &self,
        account_id: &str,
        message_id: &str,
        folder: &str,
    ) -> MailResult<()> {
        let uid: u32 = message_id
            .parse()
            .map_err(|_| MailError::Parse("Invalid message ID".to_string()))?;

        let mut connections = self.imap_connections.lock().await;
        let client = connections
            .get_mut(account_id)
            .ok_or_else(|| MailError::Connection("IMAP not connected".to_string()))?;

        client.delete_message(folder, uid).await
    }

    /// メッセージを既読にする
    pub async fn mark_as_read(
        &self,
        account_id: &str,
        message_id: &str,
        folder: &str,
    ) -> MailResult<()> {
        let uid: u32 = message_id
            .parse()
            .map_err(|_| MailError::Parse("Invalid message ID".to_string()))?;

        let mut connections = self.imap_connections.lock().await;
        let client = connections
            .get_mut(account_id)
            .ok_or_else(|| MailError::Connection("IMAP not connected".to_string()))?;

        client
            .set_message_flags(folder, uid, &[super::Flag::Seen])
            .await
    }

    /// フォルダー一覧を取得
    pub async fn get_folders(&self, account_id: &str) -> MailResult<Vec<String>> {
        // まずGmail APIクライアントが利用可能かチェック
        {
            let gmail_clients = self.gmail_api_clients.lock().await;
            if let Some(gmail_client) = gmail_clients.get(account_id) {
                println!("デバッグ: Gmail APIクライアントを使用してフォルダー一覧を取得します");
                return gmail_client.list_folders().await;
            }
        }

        // Gmail APIクライアントがない場合はIMAPクライアントを使用
        let mut connections = self.imap_connections.lock().await;
        let client = connections.get_mut(account_id).ok_or_else(|| {
            MailError::Connection("IMAP not connected and Gmail API not available".to_string())
        })?;

        println!("デバッグ: IMAPクライアントを使用してフォルダー一覧を取得します");
        client.list_folders().await
    }

    /// 接続状態をテスト
    pub async fn test_connections(&self, account_id: &str) -> MailResult<(bool, bool)> {
        let mut imap_ok = false;
        let mut smtp_ok = false;

        // IMAP接続テスト
        {
            let connections = self.imap_connections.lock().await;
            imap_ok = connections.contains_key(account_id);
        }

        // SMTP接続テスト
        {
            let connections = self.smtp_connections.lock().await;
            if let Some(client) = connections.get(account_id) {
                smtp_ok = client.test_connection().await.is_ok();
            }
        }

        Ok((imap_ok, smtp_ok))
    }

    /// すべての接続を切断
    pub async fn disconnect_all(&self, account_id: &str) -> MailResult<()> {
        self.disconnect_imap(account_id).await.ok();
        self.disconnect_smtp(account_id).await.ok();
        Ok(())
    }
}

impl Default for MailClient {
    fn default() -> Self {
        Self::new()
    }
}
