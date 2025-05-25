use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: Vec<Address>,
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
    pub subject: String,
    pub body: MessageBody,
    pub date: DateTime<Utc>,
    pub flags: Vec<Flag>,
    pub account_id: String,
    pub folder: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageBody {
    Plain(String),
    Html(String),
    Multipart { parts: Vec<MessagePart> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    pub content_type: String,
    pub content: String,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Flag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Recent,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub data: Vec<u8>,
}

impl Address {
    pub fn new(email: String, name: Option<String>) -> Self {
        Self { email, name }
    }

    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.email)
    }
}

impl MessageBody {
    pub fn new_plain(content: String) -> Self {
        Self::Plain(content)
    }

    pub fn new_html(content: String) -> Self {
        Self::Html(content)
    }

    pub fn get_display_content(&self) -> String {
        match self {
            Self::Plain(content) => content.clone(),
            Self::Html(content) => {
                // HTMLタグを簡単に除去（実際のHTMLパーサーを使うべき）
                content
                    .replace("<br>", "\n")
                    .replace("<br/>", "\n")
                    .replace("<p>", "")
                    .replace("</p>", "\n")
                    .replace("<div>", "")
                    .replace("</div>", "\n")
            }
            Self::Multipart { parts } => parts
                .iter()
                .filter(|part| part.content_type.starts_with("text/"))
                .map(|part| part.content.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

impl Message {
    pub fn new(
        id: String,
        from: Vec<Address>,
        to: Vec<Address>,
        subject: String,
        body: MessageBody,
        account_id: String,
        folder: String,
    ) -> Self {
        Self {
            id,
            from,
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            subject,
            body,
            date: Utc::now(),
            flags: Vec::new(),
            account_id,
            folder,
            attachments: Vec::new(),
        }
    }

    pub fn is_unread(&self) -> bool {
        !self.flags.contains(&Flag::Seen)
    }

    pub fn is_flagged(&self) -> bool {
        self.flags.contains(&Flag::Flagged)
    }

    pub fn has_attachments(&self) -> bool {
        !self.attachments.is_empty()
    }

    pub fn mark_as_read(&mut self) {
        if !self.flags.contains(&Flag::Seen) {
            self.flags.push(Flag::Seen);
        }
    }

    pub fn mark_as_unread(&mut self) {
        self.flags.retain(|f| f != &Flag::Seen);
    }

    pub fn toggle_flag(&mut self) {
        if self.flags.contains(&Flag::Flagged) {
            self.flags.retain(|f| f != &Flag::Flagged);
        } else {
            self.flags.push(Flag::Flagged);
        }
    }

    pub fn get_sender_display(&self) -> String {
        if self.from.is_empty() {
            "Unknown Sender".to_string()
        } else {
            let first_sender = &self.from[0];
            first_sender
                .name
                .as_ref()
                .unwrap_or(&first_sender.email)
                .clone()
        }
    }

    pub fn get_recipients_display(&self) -> String {
        if self.to.is_empty() {
            "No Recipients".to_string()
        } else {
            self.to
                .iter()
                .map(|addr| addr.name.as_deref().unwrap_or(&addr.email))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    pub fn format_date(&self) -> String {
        self.date.format("%Y-%m-%d %H:%M").to_string()
    }

    pub fn add_attachment(&mut self, attachment: Attachment) {
        self.attachments.push(attachment);
    }

    pub fn remove_attachment(&mut self, filename: &str) {
        self.attachments.retain(|a| a.filename != filename);
    }

    pub fn get_size(&self) -> usize {
        let body_size = match &self.body {
            MessageBody::Plain(content) | MessageBody::Html(content) => content.len(),
            MessageBody::Multipart { parts } => parts.iter().map(|p| p.content.len()).sum(),
        };

        let attachments_size: usize = self.attachments.iter().map(|a| a.size).sum();

        body_size + attachments_size
    }
}

impl Attachment {
    pub fn new(filename: String, content_type: String, data: Vec<u8>) -> Self {
        let size = data.len();
        Self {
            filename,
            content_type,
            size,
            data,
        }
    }

    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    pub fn is_text(&self) -> bool {
        self.content_type.starts_with("text/")
    }

    pub fn format_size(&self) -> String {
        if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        }
    }
}
