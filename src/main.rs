mod app;
mod mail;
mod search;
mod storage;
mod ui;
mod utils;

use std::error::Error;
use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use mail::{Account, AuthMethod, FolderMapping, FolderType, ImapConfig, MailClient, SmtpConfig};
use storage::Config;
use ui::render_ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 設定の読み込み
    let mut config = Config::load().unwrap_or_else(|_| {
        eprintln!("設定ファイルの読み込みに失敗しました。デフォルト設定を使用します。");
        Config::default()
    });

    // 必要なディレクトリを作成
    if let Err(e) = config.ensure_directories() {
        eprintln!("ディレクトリ作成に失敗しました: {}", e);
    }

    // デモ用のアカウントを追加（実際のアプリケーションでは設定から読み込み）
    if config.accounts.is_empty() {
        let demo_account = create_demo_account();
        if let Err(e) = config.add_account(demo_account) {
            eprintln!("デモアカウントの追加に失敗しました: {}", e);
        } else {
            let _ = config.save();
        }
    }

    // アプリケーション状態を初期化
    let mut app = App::new();
    app.config = config;

    // メールクライアントを初期化
    let mut mail_client = MailClient::new();

    // 設定からアカウントを追加
    for account in &app.config.accounts {
        if let Err(e) = mail_client.add_account(account.clone()) {
            eprintln!("アカウント {} の追加に失敗しました: {}", account.name, e);
        }
    }

    // デモ用のメッセージを読み込み
    if let Some(account) = app.config.accounts.first() {
        match mail_client
            .fetch_messages(&account.id, "INBOX", Some(10))
            .await
        {
            Ok(messages) => {
                app.messages = messages;
            }
            Err(e) => {
                eprintln!("メッセージの読み込みに失敗しました: {}", e);
            }
        }
    }

    // ターミナルのセットアップ
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // アプリケーションのメインループ
    let result = run_app(&mut terminal, &mut app, &mail_client).await;

    // ターミナルのクリーンアップ
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("エラーが発生しました: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    _mail_client: &MailClient,
) -> Result<(), Box<dyn Error>> {
    loop {
        // UIを描画
        terminal.draw(|f| render_ui(f, app))?;

        // イベントを同期的にポーリング（短いタイムアウト付き）
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // キーイベントを処理
                    app.handle_key_event(key)?;

                    // 終了フラグをチェック
                    if app.should_quit {
                        break;
                    }
                }
                _ => {
                    // その他のイベント（マウスなど）は無視
                }
            }
        }

        // 他の非同期タスクに時間を譲る
        tokio::task::yield_now().await;
    }

    Ok(())
}

fn create_demo_account() -> Account {
    // デモ用のアカウントを作成
    let mut account = Account::default();
    account.name = "デモアカウント".to_string();
    account.email = "demo@example.com".to_string();

    // デモ用のIMAP設定
    account.imap = ImapConfig {
        server: "imap.example.com".to_string(),
        port: 993,
        username: "demo@example.com".to_string(),
        password: "password".to_string(), // 実際の実装では暗号化
        use_tls: true,
        use_starttls: false,
        auth_method: AuthMethod::Plain,
        folders: vec![
            FolderMapping {
                server_name: "INBOX".to_string(),
                local_name: "受信箱".to_string(),
                folder_type: FolderType::Inbox,
            },
            FolderMapping {
                server_name: "Sent".to_string(),
                local_name: "送信済み".to_string(),
                folder_type: FolderType::Sent,
            },
            FolderMapping {
                server_name: "Drafts".to_string(),
                local_name: "下書き".to_string(),
                folder_type: FolderType::Drafts,
            },
            FolderMapping {
                server_name: "Trash".to_string(),
                local_name: "ゴミ箱".to_string(),
                folder_type: FolderType::Trash,
            },
        ],
    };

    // デモ用のSMTP設定
    account.smtp = SmtpConfig {
        server: "smtp.example.com".to_string(),
        port: 587,
        username: "demo@example.com".to_string(),
        password: "password".to_string(), // 実際の実装では暗号化
        use_tls: false,
        use_starttls: true,
        auth_method: AuthMethod::Plain,
    };

    account.signature = Some("--\nRustmail で送信".to_string());

    account
}
