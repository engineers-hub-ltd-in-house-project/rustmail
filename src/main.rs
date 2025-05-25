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
    let accounts_to_add = app.config.accounts.clone();
    for account in &accounts_to_add {
        if let Err(e) = mail_client.add_account(account.clone()) {
            eprintln!("アカウント {} の追加に失敗しました: {}", account.name, e);
        }
    }

    // OAuth2認証が必要なアカウントを特定
    let mut oauth_accounts_to_process = Vec::new();
    for (index, account) in app.config.accounts.iter().enumerate() {
        if account.imap.auth_method == AuthMethod::OAuth2 && account.tokens.is_none() {
            oauth_accounts_to_process.push((index, account.id.clone(), account.email.clone()));
        }
    }

    // OAuth2認証が必要なアカウントの処理
    let mut accounts_updated = false;
    for (account_index, account_id, account_email) in oauth_accounts_to_process {
        println!("OAuth2認証が必要です: {}", account_email);

        match mail_client.start_oauth_flow(&account_id).await {
            Ok(auth_url) => {
                println!("ブラウザで以下のURLにアクセスして認証を完了してください:");
                println!("{}", auth_url);

                // 認証コードの手動入力を受け付ける
                println!("\n認証完了後、以下の情報を入力してください:");
                println!("1. 認証コード（code=の部分）");
                println!("2. state値（上記URLに含まれている &state= の部分）");
                println!("例: 認証コード: 4/0AUJR-x4M5gPe77uV8eRUzOJ-B7xYqdVAMlowxFOoo_KhTIgI_NYbIdxc34lS80Et7_cHFQ");
                println!("例: state値: jItq_R8pT63cRebOPtmzNQ");

                use std::io::Write;
                print!("\n認証コード: ");
                std::io::stdout().flush().unwrap();

                let mut auth_code = String::new();
                std::io::stdin().read_line(&mut auth_code).unwrap();
                let auth_code = auth_code.trim().to_string();

                print!("state値: ");
                std::io::stdout().flush().unwrap();

                let mut state_value = String::new();
                std::io::stdin().read_line(&mut state_value).unwrap();
                let state_value = state_value.trim().to_string();

                if !auth_code.is_empty() && !state_value.is_empty() {
                    // OAuth2コールバックを処理
                    match mail_client
                        .handle_oauth_callback(&account_id, auth_code, state_value)
                        .await
                    {
                        Ok(_) => {
                            println!("OAuth2認証が完了しました！");

                            // 設定ファイルに更新されたアカウント情報を保存
                            for updated_account in mail_client.get_accounts() {
                                if updated_account.id == account_id {
                                    app.config.accounts[account_index] = updated_account.clone();
                                    println!("デバッグ: アカウント情報を更新しました");
                                    if let Some(tokens) = &updated_account.tokens {
                                        println!("デバッグ: OAuth2トークンが見つかりました - アクセストークン: {}...", 
                                                &tokens.access_token[..std::cmp::min(20, tokens.access_token.len())]);
                                    } else {
                                        println!("デバッグ: OAuth2トークンが見つかりません");
                                    }
                                    break;
                                }
                            }
                            accounts_updated = true;
                        }
                        Err(e) => {
                            eprintln!("OAuth2認証の処理に失敗しました: {}", e);
                        }
                    }
                } else {
                    eprintln!("認証コードまたはstate値が入力されませんでした。");
                }
            }
            Err(e) => {
                eprintln!("OAuth2フローの開始に失敗しました: {}", e);
            }
        }
    }

    // 既存のトークンが期限切れでないかチェックして、必要に応じてリフレッシュ
    for (_account_index, account) in app.config.accounts.iter_mut().enumerate() {
        if account.imap.auth_method == AuthMethod::OAuth2 && account.tokens.is_some() {
            println!("既存のOAuth2トークンをチェック中: {}", account.email);

            // トークンのリフレッシュを試行
            match mail_client.refresh_oauth_token(&account.id).await {
                Ok(_) => {
                    println!("OAuth2トークンをリフレッシュしました: {}", account.email);

                    // 更新されたトークンを設定に反映
                    for updated_account in mail_client.get_accounts() {
                        if updated_account.id == account.id {
                            *account = updated_account.clone();
                            accounts_updated = true;
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("トークンリフレッシュ警告 ({}): {}", account.email, e);
                    println!("既存のトークンをそのまま使用します");
                }
            }
        }
    }

    // OAuth2処理完了後に設定を保存
    if accounts_updated {
        if let Err(e) = app.config.save() {
            eprintln!("設定ファイルの保存に失敗しました: {}", e);
        } else {
            println!("OAuth2トークンが設定ファイルに保存されました。");
        }

        // OAuth2認証が完了したアカウントのIMAPに接続
        for account in &app.config.accounts {
            if account.imap.auth_method == AuthMethod::OAuth2 && account.tokens.is_some() {
                println!("IMAP接続を試行中: {}", account.email);
                match mail_client.connect_imap(&account.id).await {
                    Ok(_) => {
                        println!("IMAP接続が成功しました: {}", account.email);
                    }
                    Err(e) => {
                        eprintln!("IMAP接続に失敗しました ({}): {}", account.email, e);
                    }
                }
            }
        }
    }

    // デモ用のメッセージを読み込み
    let first_account = app.config.accounts.first().cloned();
    if let Some(account) = first_account {
        // OAuth2アカウントでトークンがない場合はスキップ
        if account.imap.auth_method == AuthMethod::OAuth2 && account.tokens.is_none() {
            eprintln!("OAuth2認証が完了していません。認証後に再起動してください。");
        } else {
            // IMAP接続を行う（OAuth2トークンが存在する場合も含む）
            println!("IMAP接続を試行中: {}", account.email);
            match mail_client.connect_imap(&account.id).await {
                Ok(_) => {
                    println!("IMAP接続が成功しました: {}", account.email);

                    // メッセージ取得を試行
                    match mail_client
                        .fetch_messages(&account.id, "INBOX", Some(10))
                        .await
                    {
                        Ok(messages) => {
                            println!("メッセージを {} 件取得しました", messages.len());
                            app.messages = messages;
                        }
                        Err(e) => {
                            eprintln!("メッセージの読み込みに失敗しました: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("IMAP接続に失敗しました ({}): {}", account.email, e);
                }
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
    let result = run_app(&mut terminal, &mut app, &mut mail_client).await;

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
    _mail_client: &mut MailClient,
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
