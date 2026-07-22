use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Process not found: {0}")]
    NotFound(u32),

    #[error("Port conflict detected on port {0}")]
    Conflict(u16),
}

pub type Result<T> = std::result::Result<T, Error>;
