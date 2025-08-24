pub mod config;
pub mod discovery;
pub mod error;
pub mod handler;

pub use error::{GStreamerMcpError, Result};
pub use handler::GStreamerHandler;