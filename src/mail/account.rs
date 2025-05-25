use super::oauth::{GoogleOAuthConfig, GoogleTokens};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub imap: ImapConfig,
    pub smtp: SmtpConfig,
    pub signature: Option<String>,
    pub default_folder: String,
    pub enabled: bool,
    pub oauth_config: Option<GoogleOAuthConfig>,
    pub tokens: Option<GoogleTokens>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String, // 実際の実装では暗号化して保存
    pub use_tls: bool,
    pub use_starttls: bool,
    pub auth_method: AuthMethod,
    pub folders: Vec<FolderMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String, // 実際の実装では暗号化して保存
    pub use_tls: bool,
    pub use_starttls: bool,
    pub auth_method: AuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthMethod {
    Plain,
    Login,
    #[allow(dead_code)]
    CramMd5,
    OAuth2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderMapping {
    pub folder_type: FolderType,
    pub server_name: String,
    pub local_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FolderType {
    Inbox,
    Sent,
    Drafts,
    Trash,
    #[allow(dead_code)]
    Spam,
    #[allow(dead_code)]
    Archive,
    #[allow(dead_code)]
    Custom(String),
}

impl Default for Account {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Account".to_string(),
            email: "user@example.com".to_string(),
            imap: ImapConfig::default(),
            smtp: SmtpConfig::default(),
            signature: None,
            default_folder: "INBOX".to_string(),
            enabled: true,
            oauth_config: None,
            tokens: None,
        }
    }
}

impl Default for ImapConfig {
    fn default() -> Self {
        Self {
            server: "imap.example.com".to_string(),
            port: 993,
            username: "user@example.com".to_string(),
            password: "password".to_string(),
            use_tls: true,
            use_starttls: false,
            auth_method: AuthMethod::Plain,
            folders: vec![
                FolderMapping {
                    folder_type: FolderType::Inbox,
                    server_name: "INBOX".to_string(),
                    local_name: "受信箱".to_string(),
                },
                FolderMapping {
                    folder_type: FolderType::Sent,
                    server_name: "Sent".to_string(),
                    local_name: "送信済み".to_string(),
                },
                FolderMapping {
                    folder_type: FolderType::Drafts,
                    server_name: "Drafts".to_string(),
                    local_name: "下書き".to_string(),
                },
                FolderMapping {
                    folder_type: FolderType::Trash,
                    server_name: "Trash".to_string(),
                    local_name: "ゴミ箱".to_string(),
                },
            ],
        }
    }
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            server: "smtp.example.com".to_string(),
            port: 587,
            username: "user@example.com".to_string(),
            password: "password".to_string(),
            use_tls: true,
            use_starttls: true,
            auth_method: AuthMethod::Plain,
        }
    }
}

impl Account {
    pub fn new(
        id: String,
        name: String,
        email: String,
        imap: ImapConfig,
        smtp: SmtpConfig,
    ) -> Self {
        Self {
            id,
            name,
            email,
            imap,
            smtp,
            signature: None,
            default_folder: "INBOX".to_string(),
            enabled: true,
            oauth_config: None,
            tokens: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Account name cannot be empty".to_string());
        }

        if self.email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }

        if self.imap.server.is_empty() {
            return Err("IMAP server cannot be empty".to_string());
        }

        if self.smtp.server.is_empty() {
            return Err("SMTP server cannot be empty".to_string());
        }

        if self.imap.username.is_empty() {
            return Err("IMAP username cannot be empty".to_string());
        }

        if self.smtp.username.is_empty() {
            return Err("SMTP username cannot be empty".to_string());
        }

        Ok(())
    }

    pub fn get_folder_mapping(&self, folder_type: &FolderType) -> Option<&FolderMapping> {
        self.imap
            .folders
            .iter()
            .find(|f| f.folder_type == *folder_type)
    }

    pub fn get_inbox_folder(&self) -> String {
        self.get_folder_mapping(&FolderType::Inbox)
            .map(|f| f.server_name.clone())
            .unwrap_or_else(|| "INBOX".to_string())
    }

    pub fn get_sent_folder(&self) -> String {
        self.get_folder_mapping(&FolderType::Sent)
            .map(|f| f.server_name.clone())
            .unwrap_or_else(|| "Sent".to_string())
    }

    pub fn get_drafts_folder(&self) -> String {
        self.get_folder_mapping(&FolderType::Drafts)
            .map(|f| f.server_name.clone())
            .unwrap_or_else(|| "Drafts".to_string())
    }

    pub fn get_trash_folder(&self) -> String {
        self.get_folder_mapping(&FolderType::Trash)
            .map(|f| f.server_name.clone())
            .unwrap_or_else(|| "Trash".to_string())
    }
}

impl ImapConfig {
    pub fn get_connection_url(&self) -> String {
        let scheme = if self.use_tls { "imaps" } else { "imap" };
        format!("{}://{}:{}", scheme, self.server, self.port)
    }
}

impl SmtpConfig {
    pub fn get_connection_url(&self) -> String {
        let scheme = if self.use_tls { "smtps" } else { "smtp" };
        format!("{}://{}:{}", scheme, self.server, self.port)
    }
}

impl FolderMapping {
    pub fn new(folder_type: FolderType, server_name: String, local_name: String) -> Self {
        Self {
            folder_type,
            server_name,
            local_name,
        }
    }
}
