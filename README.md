# Rustmail - Terminal Email Client

A fast, secure, and efficient terminal-based email client written in Rust, inspired by the simplicity and effectiveness of mew.

## ✨ Features

- 🚀 **High Performance** - Leveraging Rust's performance with efficient TUI rendering via ratatui
- 🔒 **Memory Safe** - Built with Rust's memory safety and type safety guarantees
- ⌨️ **Keyboard-First** - Vim-like key bindings for efficient navigation
- 📧 **Multi-Account Support** - IMAP/SMTP protocol support for multiple email accounts
- 🔍 **Fast Search** - Full-text search powered by SQLite FTS
- 💾 **Offline Support** - Local caching for fast access and offline reading
- 🎨 **Terminal UI** - Clean, responsive interface that works in any terminal

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
│   ├── client.rs        # Email client
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

## 📊 Development Status

### ✅ Completed

- Basic TUI interface with ratatui
- Project structure and module design
- Configuration file management (JSON)
- SQLite database design
- Message data structures and models
- Basic key bindings and navigation
- Demo data display
- Search engine foundation

### 🚧 In Progress

- Real IMAP/SMTP connections
- Email send/receive functionality
- Attachment handling
- Full search implementation

### 📋 Planned

- Email composition interface
- Address book management
- Email signatures
- Message templates
- Color themes and customization
- Plugin system
- GPG encryption support
- Calendar integration

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
- The Rust community for their amazing ecosystem

## 📚 References

- [IMAP Protocol RFC 3501](https://tools.ietf.org/html/rfc3501)
- [SMTP Protocol RFC 5321](https://tools.ietf.org/html/rfc5321)
- [Ratatui Documentation](https://ratatui.rs/)
- [Tokio Async Runtime](https://tokio.rs/)

---

**Note**: This is an early-stage project under active development. Features and APIs may change. Contributions and feedback are greatly appreciated! 