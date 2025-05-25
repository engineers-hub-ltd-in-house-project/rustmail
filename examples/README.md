# RustMail Configuration Examples

This directory contains sample configuration files for various email providers and use cases.

## 📋 Configuration File Structure

RustMail configuration files consist of four main sections:

```json
{
  "app": { /* Application settings */ },
  "ui": { /* User interface settings */ },
  "keybindings": { /* Keyboard shortcuts */ },
  "accounts": [ /* Array of email account configurations */ ]
}
```

## 🗂️ Sample Files

### 📧 `config.sample.json` - Basic Gmail OAuth2 Setup
Basic OAuth2 configuration sample for Gmail accounts.

**Usage:**
1. Create OAuth2 client ID in Google Cloud Console
2. Replace `YOUR_GOOGLE_CLIENT_ID` and `YOUR_GOOGLE_CLIENT_SECRET` with actual values
3. Replace `your.email@gmail.com` with your actual Gmail address
4. Copy to `~/.config/rustmail/config.json`

### 👥 `config.multi-account.json` - Multi-Account Setup
Configuration with Gmail (OAuth2), enterprise email (Login), and Yahoo Mail (Login).

**Features:**
- Gmail: OAuth2 authentication (automatically uses Gmail API)
- Enterprise email: Login authentication (app password)
- Yahoo Mail: Login authentication (disabled by default)

### 🏢 `config.enterprise.json` - Enterprise Email Setup
Configuration samples for Office 365 and on-premises Exchange Server.

**Targets:**
- Microsoft Office 365
- On-premises Exchange Server (domain authentication)

### 🍎 `config.icloud.json` - iCloud Mail Setup
Configuration samples for Apple iCloud Mail (@icloud.com, @me.com).

**Note:** iCloud requires app-specific passwords when 2FA is enabled.

## ⚙️ Configuration Sections

### 📱 App Section
```json
{
  "check_interval": 5,                    // Email check interval (minutes)
  "max_messages_per_folder": 1000,        // Max messages per folder
  "auto_mark_read": false,                // Auto-mark messages as read
  "download_attachments": false,          // Auto-download attachments
  "data_dir": "~/.config/rustmail",       // Data directory
  "log_level": "info"                     // Log level (debug/info/warn/error)
}
```

### 🎨 UI Section
```json
{
  "theme": "default",                     // Theme name
  "show_thread_tree": true,               // Show thread view
  "show_preview_pane": true,              // Show preview pane
  "date_format": "%Y-%m-%d",              // Date format
  "time_format": "%H:%M",                 // Time format
  "folder_pane_width": 20,                // Folder pane width (%)
  "message_list_height": 50               // Message list height (%)
}
```

### ⌨️ Keybindings Section
```json
{
  "quit": "q",                           // Quit application
  "up": "k", "down": "j",                // Navigate up/down (vim-style)
  "left": "h", "right": "l",             // Navigate left/right
  "compose": "c",                        // Compose email
  "reply": "r", "reply_all": "R",        // Reply/Reply all
  "delete": "d",                         // Delete
  "search": "/",                         // Search
  "refresh": "g"                         // Refresh
}
```

### 📬 Accounts Section
Each account has the following structure:

```json
{
  "id": "unique_account_id",             // Unique account identifier
  "name": "Display Name",                // Display name
  "email": "user@example.com",           // Email address
  "imap": { /* IMAP settings */ },
  "smtp": { /* SMTP settings */ },
  "signature": "Signature text",          // Email signature
  "default_folder": "INBOX",             // Default folder
  "enabled": true,                       // Account enabled/disabled
  "oauth_config": { /* OAuth2 config */ }, // Only for OAuth2
  "tokens": null                         // OAuth2 tokens (auto-generated)
}
```

## 🔐 Authentication Methods

### OAuth2 (Recommended - Gmail)
```json
{
  "auth_method": "OAuth2",
  "oauth_config": {
    "client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
    "client_secret": "YOUR_CLIENT_SECRET",
    "redirect_uri": "http://localhost:8080/oauth/callback"
  }
}
```

### Login/Plain (App Passwords)
```json
{
  "auth_method": "Login",  // or "Plain"
  "username": "user@example.com",
  "password": "app_specific_password"
}
```

## 📂 Folder Configuration

Configure folder mappings in each account's IMAP section:

```json
{
  "folders": [
    {
      "folder_type": "Inbox",            // Internal system type
      "server_name": "INBOX",            // Server folder name
      "local_name": "Inbox"              // Local display name
    }
  ]
}
```

### Main folder_type Values
- `Inbox`: Inbox
- `Sent`: Sent items
- `Drafts`: Draft messages
- `Trash`: Deleted messages

### Provider-Specific Folder Names
| Provider | Sent | Drafts | Trash |
|----------|------|--------|-------|
| Gmail | `[Gmail]/Sent Mail` | `[Gmail]/Drafts` | `[Gmail]/Trash` |
| Yahoo | `Sent` | `Draft` | `Trash` |
| Outlook | `Sent Items` | `Drafts` | `Deleted Items` |
| iCloud | `Sent Messages` | `Drafts` | `Deleted Messages` |

## 🚀 Getting Started

1. **Choose Sample**: Select appropriate sample file for your email provider
2. **Edit Configuration**: Replace placeholder values with actual account information
3. **Copy File**: Place as `~/.config/rustmail/config.json`
4. **OAuth2 Setup**: For Gmail, configure client in Google Cloud Console
5. **First Run**: Execute `cargo run` for initial OAuth2 authentication

## 💡 Tips

- **Gmail Priority**: Gmail accounts automatically use Gmail API (faster & more reliable)
- **App Passwords**: Use app-specific passwords when 2FA is enabled
- **Multiple Accounts**: Add multiple configurations to the `accounts` array
- **Disable Accounts**: Set `"enabled": false` to temporarily disable accounts

## 🔒 Security Considerations

- Passwords and tokens are stored in plaintext (encryption planned for future)
- Keep OAuth2 client_secret secure and private
- Set appropriate file permissions: `chmod 600 ~/.config/rustmail/config.json`
- Use app-specific passwords instead of main account passwords
- Regularly rotate app passwords for better security

## 🛠️ Troubleshooting

If you encounter issues:

1. **Verify Credentials**: Ensure usernames, passwords, and app passwords are correct
2. **Check Server Settings**: Verify hostnames, ports, and TLS/STARTTLS settings
3. **App Passwords**: For Yahoo, Outlook, and iCloud, ensure app passwords are enabled
4. **Gmail OAuth2**: Verify Google Cloud Console configuration and redirect URI
5. **Check Logs**: Set `"log_level": "debug"` for detailed error messages
6. **Network Issues**: Ensure firewall allows connections to email servers
7. **2FA Settings**: Disable 2FA temporarily if using regular passwords (not recommended)

### Common Issues

- **Gmail IMAP timeout**: Gmail accounts automatically fallback to Gmail API
- **Wrong folder names**: Check provider-specific folder names in the table above
- **Authentication errors**: Verify app passwords are generated correctly
- **Connection refused**: Check server hostnames and ports

For detailed setup instructions, see the main README.md file in the project root. 