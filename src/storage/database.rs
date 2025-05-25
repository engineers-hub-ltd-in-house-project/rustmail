use rusqlite::{params, Connection};
use std::path::Path;

use super::{StorageError, StorageResult};
use crate::mail::Message;

#[allow(dead_code)]
pub struct Database {
    conn: Connection,
}

#[allow(dead_code)]
impl Database {
    pub fn new<P: AsRef<Path>>(db_path: P) -> StorageResult<Self> {
        let conn = Connection::open(db_path)
            .map_err(|e| StorageError::Database(format!("Failed to open database: {}", e)))?;

        let mut db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&mut self) -> StorageResult<()> {
        // メッセージテーブル
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                folder TEXT NOT NULL,
                subject TEXT,
                from_addr TEXT,
                to_addr TEXT,
                date INTEGER,
                body TEXT,
                flags TEXT,
                raw_message TEXT
            )",
                [],
            )
            .map_err(|e| {
                StorageError::Database(format!("Failed to create messages table: {}", e))
            })?;

        // アカウントテーブル
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                config TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| {
                StorageError::Database(format!("Failed to create accounts table: {}", e))
            })?;

        // フォルダテーブル
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                server_name TEXT NOT NULL,
                message_count INTEGER DEFAULT 0,
                unread_count INTEGER DEFAULT 0,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            )",
                [],
            )
            .map_err(|e| {
                StorageError::Database(format!("Failed to create folders table: {}", e))
            })?;

        // 全文検索用FTSテーブル
        self.conn
            .execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                subject, from_addr, to_addr, body,
                content=messages,
                content_rowid=rowid
            )",
                [],
            )
            .map_err(|e| StorageError::Database(format!("Failed to create FTS table: {}", e)))?;

        // インデックス作成
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_messages_account_folder 
             ON messages(account_id, folder)",
                [],
            )
            .map_err(|e| StorageError::Database(format!("Failed to create index: {}", e)))?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_messages_date 
             ON messages(date DESC)",
                [],
            )
            .map_err(|e| StorageError::Database(format!("Failed to create date index: {}", e)))?;

        Ok(())
    }

    pub fn store_message(&mut self, message: &Message) -> StorageResult<()> {
        let flags_json = serde_json::to_string(&message.flags)
            .map_err(|e| StorageError::Database(format!("Failed to serialize flags: {}", e)))?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO messages (
                id, account_id, folder, subject, from_addr, to_addr,
                date, body, flags, raw_message
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    message.id,
                    message.account_id,
                    message.folder,
                    message.subject,
                    message.get_sender_display(),
                    message.get_recipients_display(),
                    message.date.timestamp(),
                    message.body.get_display_content(),
                    flags_json,
                    message.body.get_display_content()
                ],
            )
            .map_err(|e| StorageError::Database(format!("Failed to store message: {}", e)))?;

        // FTSテーブルも更新
        self.conn
            .execute(
                "INSERT OR REPLACE INTO messages_fts (
                rowid, subject, from_addr, to_addr, body
            ) VALUES (
                (SELECT rowid FROM messages WHERE id = ?1),
                ?2, ?3, ?4, ?5
            )",
                params![
                    message.id,
                    message.subject,
                    message.get_sender_display(),
                    message.get_recipients_display(),
                    message.body.get_display_content()
                ],
            )
            .map_err(|e| StorageError::Database(format!("Failed to update FTS: {}", e)))?;

        Ok(())
    }

    pub fn get_messages(
        &self,
        account_id: &str,
        folder: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> StorageResult<Vec<Message>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, folder, subject, from_addr, to_addr, 
                    date, body, flags, raw_message
             FROM messages 
             WHERE account_id = ?1 AND folder = ?2 
             ORDER BY date DESC 
             LIMIT ?3 OFFSET ?4",
            )
            .map_err(|e| StorageError::Database(format!("Failed to prepare statement: {}", e)))?;

        let message_iter = stmt
            .query_map(params![account_id, folder, limit, offset], |row| {
                // TODO: データベースから完全なメッセージオブジェクトを復元
                // 現在は簡単な実装のみ
                Ok(Message::new(
                    row.get(0)?,
                    vec![], // from
                    vec![], // to
                    row.get(3)?,
                    crate::mail::MessageBody::new_plain(row.get(7)?),
                    row.get(1)?,
                    row.get(2)?,
                ))
            })
            .map_err(|e| StorageError::Database(format!("Failed to query messages: {}", e)))?;

        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(
                message.map_err(|e| {
                    StorageError::Database(format!("Failed to load message: {}", e))
                })?,
            );
        }

        Ok(messages)
    }

    pub fn search_messages(&self, account_id: &str, query: &str) -> StorageResult<Vec<Message>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, m.account_id, m.folder, m.subject, m.from_addr, 
                    m.to_addr, m.date, m.body, m.flags, m.raw_message
             FROM messages m
             JOIN messages_fts fts ON m.rowid = fts.rowid
             WHERE m.account_id = ?1 AND messages_fts MATCH ?2
             ORDER BY m.date DESC",
            )
            .map_err(|e| {
                StorageError::Database(format!("Failed to prepare search statement: {}", e))
            })?;

        let message_iter = stmt
            .query_map(params![account_id, query], |row| {
                // TODO: 完全なメッセージオブジェクトを復元
                Ok(Message::new(
                    row.get(0)?,
                    vec![], // from
                    vec![], // to
                    row.get(3)?,
                    crate::mail::MessageBody::new_plain(row.get(7)?),
                    row.get(1)?,
                    row.get(2)?,
                ))
            })
            .map_err(|e| StorageError::Database(format!("Failed to search messages: {}", e)))?;

        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message.map_err(|e| {
                StorageError::Database(format!("Failed to load search result: {}", e))
            })?);
        }

        Ok(messages)
    }

    pub fn delete_message(&mut self, message_id: &str) -> StorageResult<()> {
        self.conn
            .execute("DELETE FROM messages WHERE id = ?1", params![message_id])
            .map_err(|e| StorageError::Database(format!("Failed to delete message: {}", e)))?;

        Ok(())
    }

    pub fn update_message_flags(
        &mut self,
        message_id: &str,
        flags: &[crate::mail::Flag],
    ) -> StorageResult<()> {
        let flags_json = serde_json::to_string(flags)
            .map_err(|e| StorageError::Database(format!("Failed to serialize flags: {}", e)))?;

        self.conn
            .execute(
                "UPDATE messages SET flags = ?1 WHERE id = ?2",
                params![flags_json, message_id],
            )
            .map_err(|e| StorageError::Database(format!("Failed to update flags: {}", e)))?;

        Ok(())
    }

    pub fn get_folder_stats(
        &self,
        account_id: &str,
        folder: &str,
    ) -> StorageResult<(usize, usize)> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT COUNT(*) as total,
                    COUNT(CASE WHEN flags NOT LIKE '%\"Seen\"%' THEN 1 END) as unread
             FROM messages 
             WHERE account_id = ?1 AND folder = ?2",
            )
            .map_err(|e| StorageError::Database(format!("Failed to prepare stats query: {}", e)))?;

        let row = stmt
            .query_row(params![account_id, folder], |row| {
                Ok((row.get::<_, usize>(0)?, row.get::<_, usize>(1)?))
            })
            .map_err(|e| StorageError::Database(format!("Failed to get folder stats: {}", e)))?;

        Ok(row)
    }

    pub fn vacuum(&mut self) -> StorageResult<()> {
        self.conn
            .execute("VACUUM", [])
            .map_err(|e| StorageError::Database(format!("Failed to vacuum database: {}", e)))?;
        Ok(())
    }
}
