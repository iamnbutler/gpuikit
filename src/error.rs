use std::fmt;

/// General GPUIKit error type
#[derive(Debug)]
pub enum Error {
    /// Theme not found
    ThemeNotFound(String),
    /// Asset not found
    AssetNotFound(String),
    /// Invalid configuration
    InvalidConfig(String),
    /// Other error with message
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ThemeNotFound(name) => write!(f, "Theme not found: {}", name),
            Error::AssetNotFound(path) => write!(f, "Asset not found: {}", path),
            Error::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type alias for GPUIKit operations
pub type Result<T> = std::result::Result<T, Error>;
