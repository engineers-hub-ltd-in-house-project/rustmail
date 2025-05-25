use async_imap::types::{Fetch, Flag as ImapFlag, Mailbox};
use async_imap::{Authenticator, Client, Session};
use async_native_tls::{TlsConnector, TlsStream};
use base64::{engine::general_purpose, Engine as _};
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

use super::{Account, Address, AuthMethod, Flag, MailError, MailResult, Message, MessageBody};

pub struct ImapClient {
    session: Option<Session<TlsStream<tokio_util::compat::Compat<TcpStream>>>>,
    account: Account,
}

impl ImapClient {
    pub fn new(account: Account) -> Self {
        Self {
            session: None,
            account,
        }
    }

    /// IMAPサーバーに接続
    pub async fn connect(&mut self) -> MailResult<()> {
        let imap_config = &self.account.imap;

        println!("デバッグ: IMAP接続開始");
        println!("  サーバー: {}:{}", imap_config.server, imap_config.port);
        println!(
            "  TLS: {}, STARTTLS: {}",
            imap_config.use_tls, imap_config.use_starttls
        );
        println!("  認証方式: {:?}", imap_config.auth_method);

        // TCP接続
        println!("デバッグ: TCP接続を開始中...");
        let tcp_stream = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            TcpStream::connect(&format!("{}:{}", imap_config.server, imap_config.port)),
        )
        .await
        .map_err(|_| MailError::Connection("TCP connection timeout (30 seconds)".to_string()))?
        .map_err(|e| MailError::Connection(format!("TCP connection failed: {}", e)))?;

        println!("デバッグ: TCP接続が成功しました");

        // 互換性のためのcompat変換
        let compat_stream = tcp_stream.compat();

        // TLS設定
        println!("デバッグ: TLS接続を開始中...");
        let connector = TlsConnector::new();
        let tls_stream = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            connector.connect(&imap_config.server, compat_stream),
        )
        .await
        .map_err(|_| MailError::Connection("TLS connection timeout (30 seconds)".to_string()))?
        .map_err(|e| MailError::Connection(format!("TLS connection failed: {}", e)))?;

        println!("デバッグ: TLS接続が成功しました");

        // IMAPクライアント作成
        println!("デバッグ: IMAPクライアントを作成中...");
        let client = Client::new(tls_stream);

        // 認証
        println!("デバッグ: 認証処理を開始中...");
        let session = match imap_config.auth_method {
            AuthMethod::OAuth2 => {
                // OAuth2認証の実装
                let tokens = self.account.tokens.as_ref().ok_or_else(|| {
                    MailError::Authentication(
                        "No OAuth2 tokens available. Please run OAuth2 flow first.".to_string(),
                    )
                })?;

                let _oauth_config = self.account.oauth_config.as_ref().ok_or_else(|| {
                    MailError::Authentication("No OAuth2 config available".to_string())
                })?;

                println!(
                    "デバッグ: OAuth2認証を試行中 - ユーザー: {}",
                    self.account.email
                );
                println!(
                    "デバッグ: アクセストークン長: {} 文字",
                    tokens.access_token.len()
                );
                println!(
                    "デバッグ: アクセストークンの最初の50文字: {}",
                    &tokens.access_token[..std::cmp::min(50, tokens.access_token.len())]
                );

                // XOAUTH2文字列を手動で生成
                let auth_string = format!(
                    "user={}\x01auth=Bearer {}\x01\x01",
                    self.account.email, tokens.access_token
                );
                let auth_string_b64 = general_purpose::STANDARD.encode(auth_string.as_bytes());

                println!(
                    "デバッグ: XOAUTH2文字列を生成しました（長さ: {}文字）",
                    auth_string_b64.len()
                );
                println!(
                    "デバッグ: 認証文字列の最初の100文字: {}",
                    &auth_string_b64[..std::cmp::min(100, auth_string_b64.len())]
                );

                // 改良されたXOAUTH2 Authenticatorを作成
                let authenticator = XOAuth2Authenticator::new_with_string(auth_string_b64);

                println!("デバッグ: XOAUTH2 Authenticatorを作成しました");
                println!("デバッグ: IMAP AUTHENTICATE XOAUTH2 コマンドを実行中...");

                // async-imapのauthenticateメソッドを使用してOAuth2認証を実行（タイムアウト付き）
                match tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    client.authenticate("XOAUTH2", authenticator),
                )
                .await
                {
                    Ok(Ok(session)) => {
                        println!("デバッグ: OAuth2 IMAP認証が成功しました");
                        session
                    }
                    Ok(Err(e)) => {
                        println!("デバッグ: OAuth2 IMAP認証エラー: {:?}", e);
                        return Err(MailError::Authentication(format!(
                            "OAuth2 IMAP authentication failed: {:?}. トークンが期限切れの可能性があります。再認証を試してください。",
                            e
                        )));
                    }
                    Err(_) => {
                        println!("デバッグ: OAuth2 IMAP認証がタイムアウトしました (10秒)");
                        return Err(MailError::Authentication(
                            "OAuth2 IMAP authentication timeout (10 seconds). ネットワーク接続またはサーバーの問題の可能性があります。".to_string()
                        ));
                    }
                }
            }
            AuthMethod::Plain | AuthMethod::Login => {
                println!("デバッグ: 基本認証を試行中...");
                tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    client.login(&imap_config.username, &imap_config.password),
                )
                .await
                .map_err(|_| MailError::Authentication("Login timeout (30 seconds)".to_string()))?
                .map_err(|e| MailError::Authentication(format!("Login failed: {:?}", e)))?
            }
            AuthMethod::CramMd5 => {
                return Err(MailError::Authentication(
                    "CRAM-MD5 not implemented".to_string(),
                ));
            }
        };

        println!("デバッグ: 認証が完了しました");
        self.session = Some(session);
        Ok(())
    }

    /// 接続を切断
    pub async fn disconnect(&mut self) -> MailResult<()> {
        if let Some(mut session) = self.session.take() {
            session
                .logout()
                .await
                .map_err(|e| MailError::Protocol(format!("Logout failed: {:?}", e)))?;
        }
        Ok(())
    }

    /// フォルダーを選択
    pub async fn select_folder(&mut self, folder_name: &str) -> MailResult<Mailbox> {
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        let mailbox = session
            .select(folder_name)
            .await
            .map_err(|e| MailError::Protocol(format!("Folder selection failed: {:?}", e)))?;

        Ok(mailbox)
    }

    /// フォルダー一覧を取得
    pub async fn list_folders(&mut self) -> MailResult<Vec<String>> {
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        let mut folders = session
            .list(Some(""), Some("*"))
            .await
            .map_err(|e| MailError::Protocol(format!("Folder list failed: {:?}", e)))?;

        let mut folder_names = Vec::new();
        while let Some(folder_result) = folders.next().await {
            match folder_result {
                Ok(folder) => folder_names.push(folder.name().to_string()),
                Err(e) => {
                    return Err(MailError::Protocol(format!(
                        "Folder parsing failed: {:?}",
                        e
                    )))
                }
            }
        }

        Ok(folder_names)
    }

    /// メッセージ一覧を取得
    pub async fn fetch_messages(
        &mut self,
        folder_name: &str,
        limit: Option<usize>,
    ) -> MailResult<Vec<Message>> {
        self.select_folder(folder_name).await?;

        let session = self
            .session
            .as_mut()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        // 最新のメッセージから指定数取得
        let sequence_set = if let Some(limit) = limit {
            format!("1:{}", limit)
        } else {
            "1:*".to_string()
        };

        let mut messages = session
            .fetch(&sequence_set, "ENVELOPE FLAGS INTERNALDATE RFC822.SIZE")
            .await
            .map_err(|e| MailError::Protocol(format!("Message fetch failed: {:?}", e)))?;

        let mut result = Vec::new();
        let account_id = self.account.id.clone(); // 先にクローンしておく

        while let Some(message_result) = messages.next().await {
            match message_result {
                Ok(message) => {
                    if let Some(parsed_message) =
                        Self::parse_message(&message, folder_name, &account_id)
                    {
                        result.push(parsed_message);
                    }
                }
                Err(e) => {
                    return Err(MailError::Protocol(format!(
                        "Message parsing failed: {:?}",
                        e
                    )))
                }
            }
        }

        // 最新順にソート
        result.sort_by(|a, b| b.date.cmp(&a.date));

        if let Some(limit) = limit {
            result.truncate(limit);
        }

        Ok(result)
    }

    /// メッセージ本文を取得
    pub async fn fetch_message_body(&mut self, folder_name: &str, uid: u32) -> MailResult<String> {
        self.select_folder(folder_name).await?;

        let session = self
            .session
            .as_mut()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        let mut messages = session
            .uid_fetch(&uid.to_string(), "BODY[TEXT]")
            .await
            .map_err(|e| MailError::Protocol(format!("Message body fetch failed: {:?}", e)))?;

        if let Some(message_result) = messages.next().await {
            match message_result {
                Ok(message) => {
                    if let Some(body) = message.body() {
                        return Ok(String::from_utf8_lossy(body).to_string());
                    }
                }
                Err(e) => {
                    return Err(MailError::Protocol(format!(
                        "Message body parsing failed: {:?}",
                        e
                    )))
                }
            }
        }

        Err(MailError::Protocol("Message body not found".to_string()))
    }

    /// メッセージをフラグ設定
    pub async fn set_message_flags(
        &mut self,
        folder_name: &str,
        uid: u32,
        flags: &[Flag],
    ) -> MailResult<()> {
        self.select_folder(folder_name).await?;

        let session = self
            .session
            .as_mut()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        // フラグを文字列に変換
        let flag_strings: Vec<String> = flags
            .iter()
            .filter_map(|flag| match flag {
                Flag::Seen => Some("\\Seen".to_string()),
                Flag::Answered => Some("\\Answered".to_string()),
                Flag::Flagged => Some("\\Flagged".to_string()),
                Flag::Deleted => Some("\\Deleted".to_string()),
                Flag::Draft => Some("\\Draft".to_string()),
                _ => None,
            })
            .collect();

        let flags_str = flag_strings.join(" ");

        session
            .uid_store(&uid.to_string(), &format!("+FLAGS ({})", flags_str))
            .await
            .map_err(|e| MailError::Protocol(format!("Flag setting failed: {:?}", e)))?;

        Ok(())
    }

    /// メッセージを移動
    pub async fn move_message(
        &mut self,
        from_folder: &str,
        to_folder: &str,
        uid: u32,
    ) -> MailResult<()> {
        self.select_folder(from_folder).await?;

        let session = self
            .session
            .as_mut()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        // メッセージをコピー
        session
            .uid_copy(&uid.to_string(), to_folder)
            .await
            .map_err(|e| MailError::Protocol(format!("Message copy failed: {:?}", e)))?;

        // 元のメッセージに削除フラグを設定
        session
            .uid_store(&uid.to_string(), "+FLAGS (\\Deleted)")
            .await
            .map_err(|e| MailError::Protocol(format!("Delete flag setting failed: {:?}", e)))?;

        // Expunge（実際に削除）
        session
            .expunge()
            .await
            .map_err(|e| MailError::Protocol(format!("Expunge failed: {:?}", e)))?;

        Ok(())
    }

    /// メッセージを削除
    pub async fn delete_message(&mut self, folder_name: &str, uid: u32) -> MailResult<()> {
        self.select_folder(folder_name).await?;

        let session = self
            .session
            .as_mut()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        // 削除フラグを設定
        session
            .uid_store(&uid.to_string(), "+FLAGS (\\Deleted)")
            .await
            .map_err(|e| MailError::Protocol(format!("Delete flag setting failed: {:?}", e)))?;

        // Expunge（実際に削除）
        session
            .expunge()
            .await
            .map_err(|e| MailError::Protocol(format!("Expunge failed: {:?}", e)))?;

        Ok(())
    }

    /// IMAPメッセージをパース
    fn parse_message(fetch: &Fetch, folder_name: &str, account_id: &str) -> Option<Message> {
        let envelope = fetch.envelope()?;

        // 送信者
        let from: Vec<Address> = envelope
            .from
            .as_ref()?
            .iter()
            .filter_map(|addr| {
                let mailbox = addr.mailbox.as_ref()?;
                let host = addr.host.as_ref()?;
                let email = format!(
                    "{}@{}",
                    String::from_utf8_lossy(mailbox),
                    String::from_utf8_lossy(host)
                );
                let name = addr
                    .name
                    .as_ref()
                    .map(|n| String::from_utf8_lossy(n).to_string());
                Some(Address::new(email, name))
            })
            .collect();

        // 受信者
        let to: Vec<Address> = envelope
            .to
            .as_ref()
            .map(|to_list| {
                to_list
                    .iter()
                    .filter_map(|addr| {
                        let mailbox = addr.mailbox.as_ref()?;
                        let host = addr.host.as_ref()?;
                        let email = format!(
                            "{}@{}",
                            String::from_utf8_lossy(mailbox),
                            String::from_utf8_lossy(host)
                        );
                        let name = addr
                            .name
                            .as_ref()
                            .map(|n| String::from_utf8_lossy(n).to_string());
                        Some(Address::new(email, name))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // 件名
        let subject = envelope
            .subject
            .as_ref()
            .map(|s| String::from_utf8_lossy(s).to_string())
            .unwrap_or_default();

        // メッセージID（UIDを使用）
        let message_id = fetch.uid.map(|uid| uid.to_string()).unwrap_or_default();

        // フラグ
        let mut flags = Vec::new();
        for flag in fetch.flags() {
            match flag {
                ImapFlag::Seen => flags.push(Flag::Seen),
                ImapFlag::Answered => flags.push(Flag::Answered),
                ImapFlag::Flagged => flags.push(Flag::Flagged),
                ImapFlag::Deleted => flags.push(Flag::Deleted),
                ImapFlag::Draft => flags.push(Flag::Draft),
                _ => {}
            }
        }

        // 日付
        let date = envelope
            .date
            .as_ref()
            .and_then(|d| chrono::DateTime::parse_from_rfc2822(&String::from_utf8_lossy(d)).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let mut message = Message::new(
            message_id,
            from,
            to,
            subject,
            MessageBody::new_plain("本文を読み込み中...".to_string()),
            account_id.to_string(),
            folder_name.to_string(),
        );

        message.date = date;
        message.flags = flags;

        Some(message)
    }
}

impl Drop for ImapClient {
    fn drop(&mut self) {
        // 接続があれば切断する（非同期なので完全ではない）
        if self.session.is_some() {
            // 実際の実装では適切な切断処理が必要
        }
    }
}

// OAuth2 Authenticator の実装
struct XOAuth2Authenticator {
    auth_string: String,
}

impl XOAuth2Authenticator {
    fn new(email: &str, access_token: &str) -> Self {
        let auth_string = format!("user={}\x01auth=Bearer {}\x01\x01", email, access_token);
        let auth_string_b64 = general_purpose::STANDARD.encode(auth_string.as_bytes());

        Self {
            auth_string: auth_string_b64,
        }
    }

    fn new_with_string(auth_string: String) -> Self {
        Self { auth_string }
    }
}

impl Authenticator for XOAuth2Authenticator {
    type Response = String;

    #[allow(unused_variables)]
    fn process(&mut self, challenge: &[u8]) -> Self::Response {
        // XOAUTH2では、最初のレスポンスでbase64エンコードされた認証情報を送信
        self.auth_string.clone()
    }
}
