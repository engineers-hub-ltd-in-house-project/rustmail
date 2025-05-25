use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::{StorageError, StorageResult};
use crate::mail::Account;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub ui: UiConfig,
    pub keybindings: KeyBindings,
    pub accounts: Vec<Account>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub check_interval: u64, // 分単位
    pub max_messages_per_folder: usize,
    pub auto_mark_read: bool,
    pub download_attachments: bool,
    pub data_dir: PathBuf,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_thread_tree: bool,
    pub show_preview_pane: bool,
    pub date_format: String,
    pub time_format: String,
    pub folder_pane_width: u16,
    pub message_list_height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub quit: String,
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub enter: String,
    pub compose: String,
    pub reply: String,
    pub reply_all: String,
    pub forward: String,
    pub delete: String,
    pub search: String,
    pub mark_read: String,
    pub mark_unread: String,
    pub flag: String,
    pub archive: String,
    pub next_account: String,
    pub prev_account: String,
    pub refresh: String,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustmail");

        Self {
            app: AppConfig {
                check_interval: 5,
                max_messages_per_folder: 1000,
                auto_mark_read: false,
                download_attachments: false,
                data_dir: data_dir.clone(),
                log_level: "info".to_string(),
            },
            ui: UiConfig {
                theme: "default".to_string(),
                show_thread_tree: true,
                show_preview_pane: true,
                date_format: "%Y-%m-%d".to_string(),
                time_format: "%H:%M".to_string(),
                folder_pane_width: 20,
                message_list_height: 50,
            },
            keybindings: KeyBindings {
                quit: "q".to_string(),
                up: "k".to_string(),
                down: "j".to_string(),
                left: "h".to_string(),
                right: "l".to_string(),
                enter: "Enter".to_string(),
                compose: "c".to_string(),
                reply: "r".to_string(),
                reply_all: "R".to_string(),
                forward: "f".to_string(),
                delete: "d".to_string(),
                search: "/".to_string(),
                mark_read: "m".to_string(),
                mark_unread: "u".to_string(),
                flag: "F".to_string(),
                archive: "a".to_string(),
                next_account: "n".to_string(),
                prev_account: "p".to_string(),
                refresh: "g".to_string(),
            },
            accounts: Vec::new(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| StorageError::Io(format!("Failed to read config file: {}", e)))?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| StorageError::Parse(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        // ディレクトリが存在しない場合は作成
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).map_err(|e| {
                StorageError::Io(format!("Failed to create config directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| StorageError::Parse(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content)
            .map_err(|e| StorageError::Io(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    pub fn get_config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustmail")
    }

    pub fn get_config_file() -> PathBuf {
        Self::get_config_dir().join("config.json")
    }

    pub fn get_data_dir(&self) -> &PathBuf {
        &self.app.data_dir
    }

    pub fn get_database_file(&self) -> PathBuf {
        self.get_data_dir().join("rustmail.db")
    }

    pub fn load() -> StorageResult<Self> {
        let config_file = Self::get_config_file();

        if config_file.exists() {
            Self::load_from_file(config_file)
        } else {
            // 設定ファイルが存在しない場合はデフォルト設定を作成
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> StorageResult<()> {
        let config_file = Self::get_config_file();
        self.save_to_file(config_file)
    }

    pub fn add_account(&mut self, account: Account) -> StorageResult<()> {
        account.validate().map_err(StorageError::Config)?;

        if !self.accounts.iter().any(|a| a.id == account.id) {
            self.accounts.push(account);
        }

        Ok(())
    }

    pub fn remove_account(&mut self, account_id: &str) -> StorageResult<()> {
        let index = self
            .accounts
            .iter()
            .position(|a| a.id == account_id)
            .ok_or_else(|| StorageError::Config("Account not found".to_string()))?;

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

    pub fn validate(&self) -> StorageResult<()> {
        // データディレクトリの作成可能性をチェック
        if let Err(e) = fs::create_dir_all(&self.app.data_dir) {
            return Err(StorageError::Config(format!(
                "Cannot create data directory: {}",
                e
            )));
        }

        // 各アカウントの設定をチェック
        for account in &self.accounts {
            account.validate().map_err(|e| {
                StorageError::Config(format!("Invalid account {}: {}", account.name, e))
            })?;
        }

        Ok(())
    }

    pub fn ensure_directories(&self) -> StorageResult<()> {
        fs::create_dir_all(&self.app.data_dir)
            .map_err(|e| StorageError::Io(format!("Failed to create data directory: {}", e)))?;

        fs::create_dir_all(Self::get_config_dir())
            .map_err(|e| StorageError::Io(format!("Failed to create config directory: {}", e)))?;

        Ok(())
    }
}
