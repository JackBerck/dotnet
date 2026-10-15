//! Windows native error definitions

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WindowsError {
    #[error("IP helper failed with error code: {0}")]
    IpHelperError(u32),

    #[error("Registry operation failed: {0}")]
    RegistryError(u32),

    #[error("Environment broadcast timeout or failure: {0}")]
    BroadcastError(String),

    #[error("Process inspection error: {0}")]
    ProcessError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, WindowsError>;
