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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_io_display() {
        let err = Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn error_parse_display() {
        let err = Error::Parse("invalid number".into());
        assert_eq!(err.to_string(), "Parse error: invalid number");
    }

    #[test]
    fn error_process_display() {
        let err = Error::Process("command failed".into());
        assert_eq!(err.to_string(), "Process error: command failed");
    }

    #[test]
    fn error_not_found_display() {
        let err = Error::NotFound(1234);
        assert_eq!(err.to_string(), "Process not found: 1234");
    }

    #[test]
    fn error_conflict_display() {
        let err = Error::Conflict(3000);
        assert_eq!(err.to_string(), "Port conflict detected on port 3000");
    }

    #[test]
    fn error_debug_format() {
        let err = Error::Process("test".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Process"));
    }

    #[test]
    fn error_serde_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = Error::from(json_err);
        assert!(matches!(err, Error::Serde(_)));
    }

    #[test]
    fn error_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Error>();
    }

    #[test]
    fn error_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Error>();
    }
}
