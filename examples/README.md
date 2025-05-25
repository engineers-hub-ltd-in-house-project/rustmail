# Configuration Examples

This directory contains sample configuration files for various email providers and use cases.

## Quick Setup

1. Choose the appropriate configuration file for your email provider
2. Copy it to `~/.config/rustmail/config.json`
3. Replace placeholder values with your actual credentials
4. Restart Rustmail

## Available Configuration Files

### `config.sample.json`
Basic configuration template with Gmail OAuth2 setup. Good starting point for most users.

**Use case:** Single Gmail account with OAuth2 authentication

### `config.multi-account.json`
Multi-account configuration supporting Gmail, Yahoo, Outlook, and enterprise email.

**Use case:** Multiple email accounts from different providers

### `config.enterprise.json`
Enterprise-focused configuration with optimized settings for corporate environments.

**Use case:** Corporate/business email accounts

### `config.icloud.json`
iCloud Mail specific configuration.

**Use case:** Apple iCloud email accounts

## Setup Instructions

### For Gmail (OAuth2)
1. Copy `config.sample.json` or extract the Gmail section from `config.multi-account.json`
2. Follow the OAuth2 setup guide in the main README.md
3. Replace `YOUR_GOOGLE_CLIENT_ID` and `YOUR_GOOGLE_CLIENT_SECRET` with your actual values

### For Yahoo Mail
1. Copy the Yahoo section from `config.multi-account.json`
2. Generate an app password in Yahoo Account Security settings
3. Replace `YOUR_YAHOO_APP_PASSWORD` with the generated app password

### For Outlook/Hotmail
1. Copy the Outlook section from `config.multi-account.json`
2. Generate an app password in Microsoft Account Security settings
3. Replace `YOUR_OUTLOOK_APP_PASSWORD` with the generated app password

### For iCloud Mail
1. Copy `config.icloud.json`
2. Generate an app password in Apple ID Account Security settings
3. Replace `YOUR_ICLOUD_APP_PASSWORD` with the generated app password

### For Enterprise Email
1. Copy `config.enterprise.json`
2. Replace server settings with your company's email server details
3. Replace credentials with your actual username/password

## Configuration Parameters

### App Settings
- `check_interval`: How often to check for new emails (in minutes)
- `max_messages_per_folder`: Maximum number of messages to load per folder
- `auto_mark_read`: Automatically mark emails as read when opened
- `download_attachments`: Automatically download attachments
- `data_dir`: Directory to store application data
- `log_level`: Logging level (debug, info, warn, error)

### UI Settings
- `theme`: UI theme (currently only "default" supported)
- `show_thread_tree`: Show email threading
- `show_preview_pane`: Show email preview pane
- `date_format`: Date display format
- `time_format`: Time display format
- `folder_pane_width`: Width of folder pane (percentage)
- `message_list_height`: Height of message list (percentage)

### Account Settings
- `id`: Unique identifier for the account
- `name`: Display name for the account
- `email`: Email address
- `imap`: IMAP server configuration
- `smtp`: SMTP server configuration
- `oauth_config`: OAuth2 configuration (for Gmail)

## Security Notes

1. **Never commit actual credentials** to version control
2. **Use app passwords** instead of regular passwords when possible
3. **Prefer OAuth2** over password authentication (especially for Gmail)
4. **Set proper file permissions**: `chmod 600 ~/.config/rustmail/config.json`
5. **Regularly update app passwords** for security

## Troubleshooting

If you encounter issues:
1. Verify your credentials are correct
2. Check server settings (hostname, port, TLS/STARTTLS)
3. Ensure app passwords are enabled (for Yahoo, Outlook, iCloud)
4. For Gmail OAuth2, verify your Google Cloud Console setup
5. Check the application logs for detailed error messages

For more detailed troubleshooting, see the main README.md file. 