# Rustmail - Terminal Email Client

A fast, secure, and efficient terminal-based email client written in Rust, inspired by the simplicity and effectiveness of mew.

## ✨ Features

- 🚀 **High Performance** - Leveraging Rust's performance with efficient TUI rendering via ratatui
- 🔒 **Memory Safe** - Built with Rust's memory safety and type safety guarantees
- ⌨️ **Keyboard-First** - Vim-like key bindings for efficient navigation
- 📧 **Multi-Account Support** - IMAP/SMTP protocol support for multiple email accounts
- 🔐 **OAuth2 Authentication** - Google OAuth2 support for secure authentication
- 🔍 **Fast Search** - Full-text search powered by SQLite FTS
- 💾 **Offline Support** - Local caching for fast access and offline reading
- 🎨 **Terminal UI** - Clean, responsive interface that works in any terminal
- 📤 **Email Sending** - Full SMTP support with HTML/plain text and attachments
- 📥 **Email Receiving** - Complete IMAP implementation with folder management

## 🚀 Quick Start

### Prerequisites

- Rust 1.75 or higher
- SQLite3
- A terminal emulator (works on Linux/macOS/Windows)

### Installation

```bash
git clone https://github.com/your-username/rustmail.git
cd rustmail
cargo build --release
```

### Running

```bash
# Development mode
cargo run

# Or run the compiled binary
./target/release/rustmail
```

## 📋 Detailed Setup Guide

### 1. First Setup

1. **Clone the project and build**
   ```bash
   git clone https://github.com/your-username/rustmail.git
   cd rustmail
   cargo build --release
   ```

2. **Create the configuration directory**
   ```bash
   mkdir -p ~/.config/rustmail
   ```

3. **Copy the configuration file**
   Save the following template as `~/.config/rustmail/config.json`.

### 2. Configuration File Template

#### Basic Template (Gmail + General SMTP)
```json
{
  "app": {
    "check_interval": 5,
    "max_messages_per_folder": 1000,
    "auto_mark_read": false,
    "download_attachments": false,
    "data_dir": "~/.config/rustmail",
    "log_level": "info"
  },
  "ui": {
    "theme": "default",
    "show_thread_tree": true,
    "show_preview_pane": true,
    "date_format": "%Y-%m-%d",
    "time_format": "%H:%M",
    "folder_pane_width": 20,
    "message_list_height": 50
  },
  "accounts": [
    {
      "id": "main",
      "name": "Main Account",
      "email": "your-email@gmail.com",
      "imap": {
        "server": "imap.gmail.com",
        "port": 993,
        "username": "your-email@gmail.com",
        "use_tls": true,
        "use_starttls": false,
        "auth_method": "OAuth2"
      },
      "smtp": {
        "server": "smtp.gmail.com",
        "port": 587,
        "username": "your-email@gmail.com",
        "use_tls": false,
        "use_starttls": true,
        "auth_method": "OAuth2"
      },
      "oauth_config": {
        "client_id": "YOUR_GOOGLE_CLIENT_ID",
        "client_secret": "YOUR_GOOGLE_CLIENT_SECRET",
        "redirect_uri": "http://localhost:8080/oauth/callback"
      }
    }
  ]
}
```

### 3. Email Provider Specific Configuration Examples

#### Gmail (OAuth2 Recommended)
```json
{
  "id": "gmail",
  "name": "Gmail Account",
  "email": "user@gmail.com",
  "imap": {
    "server": "imap.gmail.com",
    "port": 993,
    "username": "user@gmail.com",
    "use_tls": true,
    "use_starttls": false,
    "auth_method": "OAuth2"
  },
  "smtp": {
    "server": "smtp.gmail.com",
    "port": 587,
    "username": "user@gmail.com",
    "use_tls": false,
    "use_starttls": true,
    "auth_method": "OAuth2"
  },
  "oauth_config": {
    "client_id": "YOUR_GOOGLE_CLIENT_ID",
    "client_secret": "YOUR_GOOGLE_CLIENT_SECRET",
    "redirect_uri": "http://localhost:8080/oauth/callback"
  }
}
```

#### Yahoo Mail
```json
{
  "id": "yahoo",
  "name": "Yahoo Mail",
  "email": "user@yahoo.com",
  "imap": {
    "server": "imap.mail.yahoo.com",
    "port": 993,
    "username": "user@yahoo.com",
    "password": "YOUR_APP_PASSWORD",
    "use_tls": true,
    "use_starttls": false,
    "auth_method": "Plain"
  },
  "smtp": {
    "server": "smtp.mail.yahoo.com",
    "port": 587,
    "username": "user@yahoo.com",
    "password": "YOUR_APP_PASSWORD",
    "use_tls": false,
    "use_starttls": true,
    "auth_method": "Plain"
  }
}
```

#### Outlook/Hotmail
```json
{
  "id": "outlook",
  "name": "Outlook Account",
  "email": "user@outlook.com",
  "imap": {
    "server": "outlook.office365.com",
    "port": 993,
    "username": "user@outlook.com",
    "password": "YOUR_APP_PASSWORD",
    "use_tls": true,
    "use_starttls": false,
    "auth_method": "Plain"
  },
  "smtp": {
    "server": "smtp-mail.outlook.com",
    "port": 587,
    "username": "user@outlook.com",
    "password": "YOUR_APP_PASSWORD",
    "use_tls": false,
    "use_starttls": true,
    "auth_method": "Plain"
  }
}
```

#### iCloud Mail
```json
{
  "id": "icloud",
  "name": "iCloud Mail",
  "email": "user@icloud.com",
  "imap": {
    "server": "imap.mail.me.com",
    "port": 993,
    "username": "user@icloud.com",
    "password": "YOUR_APP_PASSWORD",
    "use_tls": true,
    "use_starttls": false,
    "auth_method": "Plain"
  },
  "smtp": {
    "server": "smtp.mail.me.com",
    "port": 587,
    "username": "user@icloud.com",
    "password": "YOUR_APP_PASSWORD",
    "use_tls": false,
    "use_starttls": true,
    "auth_method": "Plain"
  }
}
```

#### General Enterprise Email (Example)
```json
{
  "id": "work",
  "name": "Work Account",
  "email": "work@company.com",
  "imap": {
    "server": "mail.company.com",
    "port": 993,
    "username": "work@company.com",
    "password": "YOUR_PASSWORD",
    "use_tls": true,
    "use_starttls": false,
    "auth_method": "Plain"
  },
  "smtp": {
    "server": "mail.company.com",
    "port": 587,
    "username": "work@company.com",
    "password": "YOUR_PASSWORD",
    "use_tls": false,
    "use_starttls": true,
    "auth_method": "Plain"
  }
}
```

### 4. OAuth2 Setup (For Gmail)

#### Google Developer Console Configuration
1. **Access Google Cloud Console**
   - Access https://console.cloud.google.com/
   - Create a new project or select an existing one

2. **Enable Gmail API**
   ```
   Navigation → API & Services → Libraries
   → Search for "Gmail API" → Enable
   ```

3. **Create OAuth2 Credentials**
   ```
   API & Services → Credentials → Create Credentials
   → OAuth 2.0 Client ID → Desktop Application
   ```

4. **Set Redirect URI**
   ```
   Authorized Redirect URIs: http://localhost:8080/oauth/callback
   ```

5. **Get Client ID and Client Secret**
   - Copy the Client ID and Client Secret from the created credentials
   - Paste them into the corresponding parts of `config.json`

#### Adding Test User (For Development)
```
OAuth Consent Screen → Test Users → Add Your Gmail Address
```

### 5. Application Password Configuration

#### Gmail
1. **Enable Two-Factor Authentication**
   - Google Account Settings → Security → Two-Factor Authentication Process

2. **Generate Application Password**
   ```
   Google Account Settings → Security → App Passwords
   → Select Email → Generate Password
   ```

#### Yahoo Mail
1. **Access Account Security**
   - Yahoo Mail Settings → Account Information → Account Security

2. **Generate Application Password**
   ```
   App Passwords → New App Password
   → "Other Applications" → "Rustmail"
   ```

#### Outlook
1. **Access Microsoft Account**
   - account.microsoft.com → Security

2. **Create App Password**
   ```
   Other Security Options → App Password
   → New App Password
   ```

### 6. First Launch and Authentication

1. **Launch the Application**
   ```bash
   ./target/release/rustmail
   ```

2. **OAuth2 Authentication (For Gmail)**
   - Browser will automatically open
   - Login with Google Account
   - Grant Access
   - Browser will automatically close and return to the application

3. **Configuration Verification**
   - Verify that the account is recognized correctly in the application
   - Test if emails are received

### 7. Troubleshooting

#### Common Issues and Solutions

**Issue: "Authentication Error" Occurs**
```
Solution:
1. Verify that the application password is correctly set
2. Verify if Two-Factor Authentication is enabled
3. Verify Server Configuration (Host Name, Port)
```

**Issue: OAuth2 Authentication Fails**
```
Solution:
1. Verify if the project is correctly set up in Google Cloud Console
2. Verify if the Redirect URI is correctly set
3. Verify if the Test User is added (For Development)
```

**Issue: Emails Cannot Be Received**
```
Solution:
1. Verify if IMAP is enabled (For Gmail, check settings)
2. Verify if the port is not blocked by a firewall
3. Verify if TLS/STARTTLS Configuration is Correct
```

**Issue: Emails Cannot Be Sent**
```
Solution:
1. Verify SMTP Configuration
2. Verify if Sent Limit is Not Hit
3. Verify Sent Address is Correctly Set
```

#### Log File Verification
```bash
# Set log level to debug for detailed log
# Change "log_level": "debug" in config.json

# Log File Location
tail -f ~/.config/rustmail/rustmail.log
```

#### Configuration File Verification
```bash
# JSON File Syntax Check
python3 -m json.tool ~/.config/rustmail/config.json
```

### 8. Security Best Practices

1. **Use Application Password**
   - Use Dedicated App Password Instead of Regular Password

2. **Configuration File Permission Setting**
   ```bash
   chmod 600 ~/.config/rustmail/config.json
   ```

3. **Regular Password Update**
   - Update Application Password Regularly

4. **TLS/STARTTLS Enablement**
   - Use Encrypted Connection Always

5. **OAuth2 Utilization**
   - Use OAuth2 Authentication Whenever Possible (Especially For Gmail)

## ⌨️ Key Bindings

### Mail List View

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Open email |
| `c` | Compose new email |
| `r` | Reply |
| `R` | Reply all |
| `f` | Forward |
| `d` | Delete |
| `/` | Search |
| `q` | Quit |

### Mail View

| Key | Action |
|-----|--------|
| `q` / `Esc` | Back to list |
| `r` | Reply |
| `R` | Reply all |
| `f` | Forward |
| `d` | Delete |

### Search Mode

| Key | Action |
|-----|--------|
| `Enter` | Execute search |
| `Esc` | Cancel search |
| `Backspace` | Delete character |

## ⚙️ Configuration

Rustmail creates a configuration file at `~/.config/rustmail/config.json` on first run.

### Example Configuration

```json
{
  "app": {
    "check_interval": 5,
    "max_messages_per_folder": 1000,
    "auto_mark_read": false,
    "download_attachments": false,
    "data_dir": "~/.config/rustmail",
    "log_level": "info"
  },
  "ui": {
    "theme": "default",
    "show_thread_tree": true,
    "show_preview_pane": true,
    "date_format": "%Y-%m-%d",
    "time_format": "%H:%M",
    "folder_pane_width": 20,
    "message_list_height": 50
  },
  "accounts": [
    {
      "id": "work",
      "name": "Work Account",
      "email": "work@company.com",
      "imap": {
        "server": "imap.company.com",
        "port": 993,
        "username": "work@company.com",
        "password": "encrypted_password",
        "use_tls": true,
        "use_starttls": false,
        "auth_method": "Plain"
      },
      "smtp": {
        "server": "smtp.company.com",
        "port": 587,
        "username": "work@company.com",
        "password": "encrypted_password",
        "use_tls": false,
        "use_starttls": true,
        "auth_method": "Plain"
      }
    },
    {
      "id": "gmail",
      "name": "Gmail Account",
      "email": "user@gmail.com",
      "imap": {
        "server": "imap.gmail.com",
        "port": 993,
        "username": "user@gmail.com",
        "use_tls": true,
        "use_starttls": false,
        "auth_method": "OAuth2"
      },
      "smtp": {
        "server": "smtp.gmail.com",
        "port": 587,
        "username": "user@gmail.com",
        "use_tls": false,
        "use_starttls": true,
        "auth_method": "OAuth2"
      },
      "oauth_config": {
        "client_id": "your_google_client_id",
        "client_secret": "your_google_client_secret",
        "redirect_uri": "http://localhost:8080/oauth/callback"
      }
    }
  ]
}
```

## 🏗️ Architecture

```
src/
├── main.rs              # Application entry point
├── app.rs               # Application state management
├── ui/                  # User interface components
│   ├── mod.rs
│   ├── mail_list.rs     # Email list view
│   ├── mail_view.rs     # Email detail view
│   ├── compose.rs       # Email composition
│   └── widgets.rs       # Custom widgets
├── mail/                # Email processing
│   ├── mod.rs
│   ├── client.rs        # Unified email client
│   ├── imap_client.rs   # IMAP implementation
│   ├── smtp_client.rs   # SMTP implementation
│   ├── oauth.rs         # OAuth2 authentication
│   ├── message.rs       # Message structures
│   └── account.rs       # Account management
├── storage/             # Data persistence
│   ├── mod.rs
│   ├── database.rs      # SQLite database
│   └── config.rs        # Configuration management
├── search/              # Search functionality
│   └── mod.rs
└── utils/               # Utility functions
    └── mod.rs
```

## 📧 Email Features

### SMTP Support
- **Multiple Authentication**: Plain, Login, OAuth2 (Google)
- **Security**: TLS/STARTTLS support
- **Message Formats**: HTML and plain text emails
- **Attachments**: Full attachment support
- **Signatures**: Automatic signature appending

### IMAP Support  
- **Folder Management**: List, select, and navigate folders
- **Message Operations**: Fetch, move, delete, flag management
- **Streaming**: Efficient message streaming with async support
- **Search**: Server-side search capabilities
- **Flags**: Read/unread, flagged, deleted status management

### OAuth2 Integration
- **Google OAuth2**: Complete Google authentication flow
- **Token Management**: Automatic token refresh
- **XOAUTH2**: SASL XOAUTH2 string generation
- **User Info**: Retrieve user profile information
- **Security**: CSRF protection and secure token storage

## 📊 Development Status

### ✅ Completed

- ✅ Basic TUI interface with ratatui
- ✅ Project structure and module design
- ✅ Configuration file management (JSON)
- ✅ SQLite database design
- ✅ Message data structures and models
- ✅ Basic key bindings and navigation
- ✅ Demo data display
- ✅ Search engine foundation
- ✅ **IMAP client implementation** (async-imap with tokio compatibility)
- ✅ **SMTP client implementation** (lettre with OAuth2 support)
- ✅ **Google OAuth2 authentication** (oauth2 crate integration)
- ✅ **Multi-account management** with OAuth support
- ✅ **Message operations** (fetch, send, move, delete, flag management)
- ✅ **TLS/STARTTLS security** for both IMAP and SMTP

### 🚧 In Progress

- Email composition interface integration with TUI
- Attachment handling in UI
- Full search implementation with backend integration
- OAuth2 token persistence and management

### 📋 Planned

- Address book management
- Email signatures UI
- Message templates
- Color themes and customization
- Plugin system
- GPG encryption support
- Calendar integration
- Threading and conversation view

## 🛠️ Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Release Build

```bash
cargo build --release
```

### Development with Hot Reload

```bash
cargo watch -x run
```

## 🔧 Dependencies

### Core Email Libraries
- **async-imap**: Async IMAP client implementation
- **lettre**: SMTP client with modern Rust async support
- **oauth2**: OAuth2 client for secure authentication
- **tokio-util**: Compatibility layer for async streams

### UI and System
- **ratatui**: Terminal user interface framework
- **crossterm**: Cross-platform terminal library
- **tokio**: Async runtime

### Data and Storage
- **serde**: Serialization framework
- **rusqlite**: SQLite database interface
- **chrono**: Date and time handling

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting (`cargo test && cargo clippy`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[Mew](https://www.mew.org/ja/)** - Our primary inspiration. Mew is an excellent email client for Emacs developed by Kazuhiko Yamamoto since 1994. Its elegant design philosophy, efficient keyboard-driven interface, and long-standing commitment to user experience have been the guiding principles for Rustmail's development. We deeply respect the decades of innovation and refinement that Mew represents in the email client ecosystem.
- [ratatui](https://ratatui.rs/) - Excellent TUI framework for Rust
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal library
- [async-imap](https://docs.rs/async-imap/) - Async IMAP client library
- [lettre](https://docs.rs/lettre/) - Email library for Rust
- [oauth2](https://docs.rs/oauth2/) - OAuth2 client library
- The Rust community for their amazing ecosystem

## 📚 References

- [IMAP Protocol RFC 3501](https://tools.ietf.org/html/rfc3501)
- [SMTP Protocol RFC 5321](https://tools.ietf.org/html/rfc5321)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [SASL XOAUTH2 Mechanism](https://developers.google.com/gmail/imap/xoauth2-protocol)
- [Ratatui Documentation](https://ratatui.rs/)
- [Tokio Async Runtime](https://tokio.rs/)

---

**Note**: This project now includes functional SMTP/IMAP communication and OAuth2 authentication! Core email functionality is implemented and ready for integration with the TUI interface. Contributions and feedback are greatly appreciated! 