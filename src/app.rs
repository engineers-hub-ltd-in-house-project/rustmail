use crossterm::event::{KeyCode, KeyEvent};
use std::error::Error;

use crate::mail::{Account, Message};
use crate::storage::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    MailList,
    MailView,
    Compose,
    Help,
    #[allow(dead_code)]
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    #[allow(dead_code)]
    Command,
}

pub struct App {
    pub should_quit: bool,
    pub mode: AppMode,
    pub previous_mode: Option<AppMode>,
    pub input_mode: InputMode,
    pub mail_list_state: ratatui::widgets::ListState,
    pub folder_list_state: ratatui::widgets::ListState,
    pub messages: Vec<Message>,
    pub current_message: Option<Message>,
    pub accounts: Vec<Account>,
    pub current_account_index: usize,
    #[allow(dead_code)]
    pub current_folder: String,
    pub search_query: String,
    pub status_message: String,
    pub config: Config,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            should_quit: false,
            mode: AppMode::MailList,
            previous_mode: None,
            input_mode: InputMode::Normal,
            mail_list_state: ratatui::widgets::ListState::default(),
            folder_list_state: ratatui::widgets::ListState::default(),
            messages: Vec::new(),
            current_message: None,
            accounts: Vec::new(),
            current_account_index: 0,
            current_folder: "INBOX".to_string(),
            search_query: String::new(),
            status_message: "Ready".to_string(),
            config: Config::default(),
        };

        // デフォルトで最初のアイテムを選択
        app.mail_list_state.select(Some(0));
        app.folder_list_state.select(Some(0));

        app
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn show_help(&mut self) {
        if self.mode != AppMode::Help {
            self.previous_mode = Some(self.mode.clone());
            self.mode = AppMode::Help;
        }
    }

    pub fn hide_help(&mut self) {
        if self.mode == AppMode::Help {
            self.mode = self.previous_mode.take().unwrap_or(AppMode::MailList);
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key_event(key_event),
            InputMode::Search => self.handle_search_key_event(key_event),
            InputMode::Command => self.handle_command_key_event(key_event),
        }
    }

    fn handle_normal_key_event(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
        match self.mode {
            AppMode::MailList => match key_event.code {
                KeyCode::Char('q') => self.quit(),
                KeyCode::Char('h') => self.show_help(),
                KeyCode::Char('j') | KeyCode::Down => self.select_next_mail(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous_mail(),
                KeyCode::Enter => self.open_selected_mail(),
                KeyCode::Char('c') => self.mode = AppMode::Compose,
                KeyCode::Char('r') => self.reply_to_selected_mail(),
                KeyCode::Char('R') => self.reply_all_to_selected_mail(),
                KeyCode::Char('f') => self.forward_selected_mail(),
                KeyCode::Char('d') => self.delete_selected_mail(),
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Search;
                    self.search_query.clear();
                }
                _ => {}
            },
            AppMode::MailView => match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => self.mode = AppMode::MailList,
                KeyCode::Char('h') => self.show_help(),
                KeyCode::Char('r') => self.reply_to_current_mail(),
                KeyCode::Char('R') => self.reply_all_to_current_mail(),
                KeyCode::Char('f') => self.forward_current_mail(),
                KeyCode::Char('d') => self.delete_current_mail(),
                _ => {}
            },
            AppMode::Compose => match key_event.code {
                KeyCode::Esc => self.mode = AppMode::MailList,
                KeyCode::Char('h') => self.show_help(),
                KeyCode::F(10) => self.send_composed_mail(),
                _ => {}
            },
            AppMode::Help => match key_event.code {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') => {
                    self.hide_help();
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_search_key_event(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
        match key_event.code {
            KeyCode::Enter => {
                self.perform_search();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_command_key_event(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn Error>> {
        // コマンドモードの実装は後で追加
        if key_event.code == KeyCode::Esc {
            self.input_mode = InputMode::Normal;
        }
        Ok(())
    }

    // メール操作メソッド
    fn select_next_mail(&mut self) {
        let i = match self.mail_list_state.selected() {
            Some(i) => {
                if i >= self.messages.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.mail_list_state.select(Some(i));
    }

    fn select_previous_mail(&mut self) {
        let i = match self.mail_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.messages.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.mail_list_state.select(Some(i));
    }

    fn open_selected_mail(&mut self) {
        if let Some(i) = self.mail_list_state.selected() {
            if let Some(message) = self.messages.get(i) {
                self.current_message = Some(message.clone());
                self.mode = AppMode::MailView;
            }
        }
    }

    fn reply_to_selected_mail(&mut self) {
        // TODO: 返信機能の実装
        self.status_message = "Reply功能は未実装です".to_string();
    }

    fn reply_all_to_selected_mail(&mut self) {
        // TODO: 全員への返信機能の実装
        self.status_message = "Reply All功能は未実装です".to_string();
    }

    fn forward_selected_mail(&mut self) {
        // TODO: 転送機能の実装
        self.status_message = "Forward功能は未実装です".to_string();
    }

    fn delete_selected_mail(&mut self) {
        // TODO: 削除機能の実装
        self.status_message = "Delete功能は未実装です".to_string();
    }

    fn reply_to_current_mail(&mut self) {
        // TODO: 現在のメールへの返信
        self.status_message = "Reply功能は未実装です".to_string();
    }

    fn reply_all_to_current_mail(&mut self) {
        // TODO: 現在のメールへの全員返信
        self.status_message = "Reply All功能は未実装です".to_string();
    }

    fn forward_current_mail(&mut self) {
        // TODO: 現在のメールの転送
        self.status_message = "Forward功能は未実装です".to_string();
    }

    fn delete_current_mail(&mut self) {
        // TODO: 現在のメールの削除
        self.status_message = "Delete功能は未実装です".to_string();
    }

    fn send_composed_mail(&mut self) {
        // TODO: 作成したメールの送信
        self.status_message = "Send功能は未実装です".to_string();
    }

    fn perform_search(&mut self) {
        // TODO: 検索機能の実装
        self.status_message = format!("Searching for: {}", self.search_query);
    }

    pub fn get_current_account(&self) -> Option<&Account> {
        self.accounts.get(self.current_account_index)
    }
}
