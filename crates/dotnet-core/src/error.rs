use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Windows subsystem error: {0}")]
    Windows(#[from] dotnet_windows::WindowsError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Tool validation failed: {0}")]
    Validation(String),

    #[error("Service error: {0}")]
    Service(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
