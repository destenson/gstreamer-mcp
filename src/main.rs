use anyhow::Result;
use gstreamer_mcp::{config::Configuration, handler::GStreamerHandler};
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing to stderr
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting gstreamer-mcp server");

    // Load configuration
    let mut config = if let Ok(config_path) = std::env::var("GSTREAMER_MCP_CONFIG") {
        Configuration::load_from_file(&config_path)?
    } else if std::path::Path::new("gstreamer-mcp.toml").exists() {
        Configuration::load_from_file("gstreamer-mcp.toml")?
    } else {
        Configuration::default()
    };

    // Merge environment variables
    config.merge_env_vars();

    tracing::info!(
        "Configuration loaded, cache enabled: {}",
        config.cache_enabled
    );

    // Create handler with configuration
    let handler = GStreamerHandler::with_config(config).await?;

    tracing::info!("GStreamer initialized, starting MCP server on stdio");

    // Start the MCP server
    let service = handler
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Server error: {:?}", e);
        })?;

    // Wait for the service to complete
    service.waiting().await?;

    tracing::info!("Server shutting down");
    Ok(())
}
