//! Dotnet Core Engine Library in Rust

pub mod catalog;
pub mod config;
pub mod environment;
pub mod error;
pub mod installation;
pub mod models;
pub mod persistence;
pub mod services;

pub use error::{CoreError, Result};
pub use persistence::AppPaths;
