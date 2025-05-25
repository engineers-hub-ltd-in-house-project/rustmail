pub mod config;
pub mod database;

pub use config::Config;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    Io(String),
    Parse(String),
    #[allow(dead_code)]
    Database(String),
    Config(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io(msg) => write!(f, "I/O error: {}", msg),
            StorageError::Parse(msg) => write!(f, "Parse error: {}", msg),
            StorageError::Database(msg) => write!(f, "Database error: {}", msg),
            StorageError::Config(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl Error for StorageError {}

pub type StorageResult<T> = Result<T, StorageError>;
