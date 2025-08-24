use rmcp::ErrorData as McpError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GStreamerMcpError {
    #[error("GStreamer initialization failed: {0}")]
    GStreamerInit(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Invalid element name: {0}")]
    InvalidElementName(String),

    #[error("Registry access failed: {0}")]
    RegistryError(String),

    #[error("Property inspection failed: {0}")]
    PropertyError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GStreamerMcpError>;

impl From<GStreamerMcpError> for McpError {
    fn from(err: GStreamerMcpError) -> Self {
        McpError {
            code: rmcp::model::ErrorCode::from(match &err {
                GStreamerMcpError::ElementNotFound(_) => -32001,
                GStreamerMcpError::PluginNotFound(_) => -32002,
                GStreamerMcpError::InvalidElementName(_) => -32003,
                GStreamerMcpError::GStreamerInit(_) => -32004,
                GStreamerMcpError::RegistryError(_) => -32005,
                GStreamerMcpError::PropertyError(_) => -32006,
                _ => -32000,
            }),
            message: err.to_string().into(),
            data: None,
        }
    }
}