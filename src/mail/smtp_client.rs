use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Message as LettreMessage, SmtpTransport, Transport};
use std::time::{Duration, SystemTime};

use super::oauth::GoogleOAuthClient;
use super::{Account, AuthMethod, MailError, MailResult, Message};

pub struct SmtpClient {
    account: Account,
    transport: Option<SmtpTransport>,
}

impl SmtpClient {
    pub fn new(account: Account) -> Self {
        Self {
            account,
            transport: None,
        }
    }

    /// SMTPサーバーに接続
    pub async fn connect(&mut self) -> MailResult<()> {
        let smtp_config = &self.account.smtp;

        let mut transport_builder = if smtp_config.use_tls {
            // 直接TLS接続（通常はポート465）
            SmtpTransport::relay(&smtp_config.server)
                .map_err(|e| MailError::Connection(format!("SMTP relay error: {}", e)))?
                .port(smtp_config.port)
        } else {
            // 平文またはSTARTTLS接続（通常はポート587）
            SmtpTransport::builder_dangerous(&smtp_config.server).port(smtp_config.port)
        };

        // STARTTLS設定
        if smtp_config.use_starttls {
            let tls_parameters = TlsParameters::new(smtp_config.server.clone())
                .map_err(|e| MailError::Connection(format!("TLS parameters error: {}", e)))?;
            transport_builder = transport_builder.tls(Tls::Required(tls_parameters));
        }

        // 認証設定
        match smtp_config.auth_method {
            AuthMethod::OAuth2 => {
                transport_builder = self.setup_oauth2_auth(transport_builder).await?;
            }
            AuthMethod::Plain => {
                let creds =
                    Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());
                transport_builder = transport_builder
                    .credentials(creds)
                    .authentication(vec![Mechanism::Plain]);
            }
            AuthMethod::Login => {
                let creds =
                    Credentials::new(smtp_config.username.clone(), smtp_config.password.clone());
                transport_builder = transport_builder
                    .credentials(creds)
                    .authentication(vec![Mechanism::Login]);
            }
            AuthMethod::CramMd5 => {
                // CRAM-MD5は現在のlettreでサポートされていない
                return Err(MailError::Authentication(
                    "CRAM-MD5 not supported".to_string(),
                ));
            }
        }

        // タイムアウト設定
        transport_builder = transport_builder.timeout(Some(Duration::from_secs(30)));

        let transport = transport_builder.build();

        // 接続テスト
        transport
            .test_connection()
            .map_err(|e| MailError::Connection(format!("Connection test failed: {}", e)))?;

        self.transport = Some(transport);
        Ok(())
    }

    /// OAuth2認証を設定
    async fn setup_oauth2_auth(
        &self,
        transport_builder: lettre::transport::smtp::SmtpTransportBuilder,
    ) -> MailResult<lettre::transport::smtp::SmtpTransportBuilder> {
        let tokens =
            self.account.tokens.as_ref().ok_or_else(|| {
                MailError::Authentication("No OAuth2 tokens available".to_string())
            })?;

        let oauth_config =
            self.account.oauth_config.as_ref().ok_or_else(|| {
                MailError::Authentication("No OAuth2 config available".to_string())
            })?;

        let oauth_client = GoogleOAuthClient::new(oauth_config.clone()).map_err(|e| {
            MailError::Authentication(format!("OAuth2 client creation failed: {}", e))
        })?;

        let xoauth2_string =
            oauth_client.generate_xoauth2_string(&self.account.email, &tokens.access_token);

        // OAuth2用のカスタム認証
        let creds = Credentials::new(self.account.email.clone(), xoauth2_string);

        Ok(transport_builder
            .credentials(creds)
            .authentication(vec![Mechanism::Xoauth2]))
    }

    /// メールを送信
    pub async fn send_message(&mut self, message: &Message) -> MailResult<()> {
        let transport = self
            .transport
            .as_ref()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        // Lettreメッセージを構築
        let email = self.build_lettre_message(message)?;

        // メール送信
        transport
            .send(&email)
            .map_err(|e| MailError::Protocol(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    /// Lettreメッセージを構築
    fn build_lettre_message(&self, message: &Message) -> MailResult<LettreMessage> {
        let mut builder = LettreMessage::builder();

        // From
        builder = builder.from(
            format!("{} <{}>", self.account.name, self.account.email)
                .parse()
                .map_err(|e| MailError::Parse(format!("Invalid from address: {}", e)))?,
        );

        // Subject
        builder = builder.subject(&message.subject);

        // To recipients
        for to in &message.to {
            let address = if let Some(name) = &to.name {
                format!("{} <{}>", name, to.email)
            } else {
                to.email.clone()
            };
            builder = builder.to(address
                .parse()
                .map_err(|e| MailError::Parse(format!("Invalid to address: {}", e)))?);
        }

        // CC recipients
        for cc in &message.cc {
            let address = if let Some(name) = &cc.name {
                format!("{} <{}>", name, cc.email)
            } else {
                cc.email.clone()
            };
            builder = builder.cc(address
                .parse()
                .map_err(|e| MailError::Parse(format!("Invalid cc address: {}", e)))?);
        }

        // BCC recipients
        for bcc in &message.bcc {
            let address = if let Some(name) = &bcc.name {
                format!("{} <{}>", name, bcc.email)
            } else {
                bcc.email.clone()
            };
            builder = builder.bcc(
                address
                    .parse()
                    .map_err(|e| MailError::Parse(format!("Invalid bcc address: {}", e)))?,
            );
        }

        // Message-ID
        if !message.id.is_empty() {
            builder = builder.message_id(Some(message.id.clone()));
        }

        // Date - chrono::DateTime<Utc>をSystemTimeに変換
        let system_time: SystemTime = message.date.into();
        builder = builder.date(system_time);

        // Body
        let body_content = match &message.body {
            super::MessageBody::Plain(text) => text.clone(),
            super::MessageBody::Html(html) => html.clone(),
            super::MessageBody::Multipart { parts } => {
                // テキストパートを優先的に選択
                parts
                    .iter()
                    .find(|part| part.content_type.starts_with("text/plain"))
                    .or_else(|| {
                        parts
                            .iter()
                            .find(|part| part.content_type.starts_with("text/"))
                    })
                    .map(|part| part.content.clone())
                    .unwrap_or_default()
            }
        };

        // 署名を追加
        let final_body = if let Some(signature) = &self.account.signature {
            format!("{}\n\n--\n{}", body_content, signature)
        } else {
            body_content
        };

        // メッセージタイプに応じてボディを設定
        let email = match &message.body {
            super::MessageBody::Html(_) => builder
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(final_body)
                .map_err(|e| MailError::Parse(format!("Failed to build HTML message: {}", e)))?,
            _ => builder
                .body(final_body)
                .map_err(|e| MailError::Parse(format!("Failed to build message: {}", e)))?,
        };

        Ok(email)
    }

    /// 接続をテスト
    pub async fn test_connection(&self) -> MailResult<()> {
        let transport = self
            .transport
            .as_ref()
            .ok_or_else(|| MailError::Connection("Not connected".to_string()))?;

        transport
            .test_connection()
            .map_err(|e| MailError::Connection(format!("Connection test failed: {}", e)))?;

        Ok(())
    }

    /// 接続を切断
    pub fn disconnect(&mut self) {
        self.transport = None;
    }

    /// 送信ログを取得（実装例）
    pub fn get_send_log(&self) -> Vec<String> {
        // 実際の実装では送信履歴を管理
        vec![
            "2024-01-01 10:00:00 - メール送信成功".to_string(),
            "2024-01-01 09:30:00 - メール送信成功".to_string(),
        ]
    }
}

// SMTP認証メカニズム用のヘルパー
impl From<AuthMethod> for Vec<Mechanism> {
    fn from(auth_method: AuthMethod) -> Self {
        match auth_method {
            AuthMethod::Plain => vec![Mechanism::Plain],
            AuthMethod::Login => vec![Mechanism::Login],
            AuthMethod::CramMd5 => vec![], // サポートされていない
            AuthMethod::OAuth2 => vec![Mechanism::Xoauth2],
        }
    }
}

impl Drop for SmtpClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail::{Address, MessageBody};

    #[test]
    fn test_smtp_client_creation() {
        let account = Account::default();
        let client = SmtpClient::new(account);
        assert!(client.transport.is_none());
    }

    #[test]
    fn test_build_lettre_message() {
        let account = Account::default();
        let client = SmtpClient::new(account);

        let message = Message::new(
            "test-id".to_string(),
            vec![Address::new(
                "sender@example.com".to_string(),
                Some("Sender".to_string()),
            )],
            vec![Address::new(
                "recipient@example.com".to_string(),
                Some("Recipient".to_string()),
            )],
            "テストメール".to_string(),
            MessageBody::new_plain("これはテストメールです。".to_string()),
            "test-account".to_string(),
            "Sent".to_string(),
        );

        let result = client.build_lettre_message(&message);
        assert!(result.is_ok());
    }
}
