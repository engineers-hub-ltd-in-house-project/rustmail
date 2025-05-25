pub mod compose;
pub mod mail_list;
pub mod mail_view;
pub mod widgets;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, AppMode, InputMode};

pub fn render_ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    match app.mode {
        AppMode::MailList => render_mail_list(f, app, size),
        AppMode::MailView => render_mail_view(f, app, size),
        AppMode::Compose => render_compose(f, app, size),
        AppMode::Settings => render_settings(f, app, size),
    }
}

fn render_mail_list(f: &mut Frame, app: &mut App, area: Rect) {
    // メイン画面のレイアウト
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Min(0),    // メイン部分
            Constraint::Length(1), // ステータスバー
        ])
        .split(area);

    // タブバー
    render_tab_bar(f, app, chunks[0]);

    // メイン部分を左右に分割
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // フォルダ一覧
            Constraint::Percentage(75), // メール一覧
        ])
        .split(chunks[1]);

    // フォルダ一覧
    render_folder_list(f, app, main_chunks[0]);

    // メール一覧
    render_message_list(f, app, main_chunks[1]);

    // ステータスバー
    render_status_bar(f, app, chunks[2]);

    // 検索モードの場合は検索バーを表示
    if app.input_mode == InputMode::Search {
        render_search_bar(f, app, area);
    }
}

fn render_mail_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Min(0),    // メール表示
            Constraint::Length(1), // ステータスバー
        ])
        .split(area);

    render_tab_bar(f, app, chunks[0]);

    if let Some(message) = &app.current_message {
        render_message_detail(f, message, chunks[1]);
    } else {
        let block = Block::default().title("メール表示").borders(Borders::ALL);
        let paragraph = Paragraph::new("メッセージが選択されていません").block(block);
        f.render_widget(paragraph, chunks[1]);
    }

    render_status_bar(f, app, chunks[2]);
}

fn render_compose(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // タブバー
            Constraint::Min(0),    // 作成画面
            Constraint::Length(1), // ステータスバー
        ])
        .split(area);

    render_tab_bar(f, app, chunks[0]);

    let block = Block::default().title("メール作成").borders(Borders::ALL);
    let paragraph = Paragraph::new("メール作成機能は未実装です\nEscキーで戻ります").block(block);
    f.render_widget(paragraph, chunks[1]);

    render_status_bar(f, app, chunks[2]);
}

fn render_settings(f: &mut Frame, _app: &mut App, area: Rect) {
    let block = Block::default().title("設定").borders(Borders::ALL);
    let paragraph = Paragraph::new("設定機能は未実装です").block(block);
    f.render_widget(paragraph, area);
}

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let current_account = app
        .get_current_account()
        .map(|a| a.name.as_str())
        .unwrap_or("アカウントなし");

    let tabs = [
        "[1]受信箱".to_string(),
        "[2]送信済み".to_string(),
        "[3]下書き".to_string(),
        "[4]ゴミ箱".to_string(),
        format!("| アカウント: {}", current_account),
    ];

    let tab_text = tabs.join(" ");
    let paragraph =
        Paragraph::new(tab_text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(paragraph, area);
}

fn render_folder_list(f: &mut Frame, app: &mut App, area: Rect) {
    let folders = ["受信箱 (5)", "送信済み", "下書き", "ゴミ箱", "アーカイブ"];

    let items: Vec<ListItem> = folders
        .iter()
        .map(|folder| ListItem::new(Line::from(Span::raw(*folder))))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("フォルダ").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.folder_list_state);
}

fn render_message_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|message| {
            let style = if message.is_unread() {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let attachment_indicator = if message.has_attachments() {
                "📎 "
            } else {
                ""
            };
            let flag_indicator = if message.is_flagged() { "🏴 " } else { "" };

            let line = Line::from(vec![
                Span::styled(
                    format!("{}{}● ", attachment_indicator, flag_indicator),
                    style,
                ),
                Span::styled(format!("From: {}", message.get_sender_display()), style),
                Span::raw("\n   "),
                Span::styled(format!("Sub: {}", message.subject), style),
                Span::raw("\n   "),
                Span::styled(message.format_date(), Style::default().fg(Color::Gray)),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("メール一覧").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.mail_list_state);
}

fn render_message_detail(f: &mut Frame, message: &crate::mail::Message, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // ヘッダー
            Constraint::Min(0),    // 本文
        ])
        .split(area);

    // ヘッダー部分
    let header_text = format!(
        "From: {}\nTo: {}\nSubject: {}\nDate: {}\n{}",
        message.get_sender_display(),
        message.get_recipients_display(),
        message.subject,
        message.format_date(),
        if message.has_attachments() {
            format!("Attachments: {} file(s)", message.attachments.len())
        } else {
            String::new()
        }
    );

    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .title("メールヘッダー")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(header, chunks[0]);

    // 本文部分
    let body = Paragraph::new(message.body.get_display_content())
        .block(Block::default().title("メール本文").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(body, chunks[1]);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match app.input_mode {
        InputMode::Search => format!("検索: {} | {}", app.search_query, app.status_message),
        _ => app.status_message.clone(),
    };

    let paragraph =
        Paragraph::new(status_text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(paragraph, area);
}

fn render_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 20, area);
    f.render_widget(Clear, popup_area);

    let search_text = format!("検索: {}", app.search_query);
    let paragraph = Paragraph::new(search_text)
        .block(Block::default().title("検索").borders(Borders::ALL))
        .style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(paragraph, popup_area);
}

// ポップアップ用のヘルパー関数
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
