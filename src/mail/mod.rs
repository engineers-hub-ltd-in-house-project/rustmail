pub mod account;
pub mod client;
pub mod imap_client;
pub mod message;
pub mod oauth;
pub mod smtp_client;

pub use account::{Account, AuthMethod, FolderMapping, FolderType, ImapConfig, SmtpConfig};
pub use client::MailClient;
pub use imap_client::ImapClient;
pub use message::{Address, Flag, Message, MessageBody};
pub use oauth::{
    GoogleOAuthClient, GoogleOAuthConfig, GoogleTokens, GoogleUserInfo, OAuthFlowManager,
};
pub use smtp_client::SmtpClient;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MailError {
    #[allow(dead_code)]
    Connection(String),
    #[allow(dead_code)]
    Authentication(String),
    #[allow(dead_code)]
    Protocol(String),
    #[allow(dead_code)]
    Io(String),
    Parse(String),
}

impl fmt::Display for MailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MailError::Connection(msg) => write!(f, "Connection error: {}", msg),
            MailError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            MailError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            MailError::Io(msg) => write!(f, "I/O error: {}", msg),
            MailError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl Error for MailError {}

pub type MailResult<T> = Result<T, MailError>;
