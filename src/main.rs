use anyhow::Result;
use gstreamer_mcp::{
    cli::Cli,
    config::Configuration,
    handler::GStreamerHandler,
    repl,
};
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments BEFORE stdio takeover
    let cli_config = Cli::parse_with_env();
    
    // Configure logging based on verbosity level
    let log_level = match cli_config.verbose_level {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    
    // Initialize tracing to stderr
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(log_level.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(!cli_config.no_color)
        .init();

    tracing::info!("Starting gstreamer-mcp server");
    tracing::debug!("CLI config: {:?}", cli_config);

    // Load configuration from file
    let mut config = if let Some(ref config_path) = cli_config.config_path {
        Configuration::load_from_file(config_path)?
    } else if let Ok(config_path) = std::env::var("GSTREAMER_MCP_CONFIG") {
        Configuration::load_from_file(&config_path)?
    } else if std::path::Path::new("gstreamer-mcp.toml").exists() {
        Configuration::load_from_file("gstreamer-mcp.toml")?
    } else {
        Configuration::default()
    };

    // Merge environment variables
    config.merge_env_vars();
    
    // Merge CLI arguments (highest priority)
    config.merge_cli_args(&cli_config);

    tracing::info!(
        "Configuration loaded, mode: {:?}, cache: {}",
        config.operational_mode,
        config.cache_enabled
    );

    // Check if running in REPL mode
    if cli_config.repl {
        tracing::info!("Starting REPL mode");
        repl::run_repl(config).await?;
    } else {
        // Normal MCP server mode
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
    }

    Ok(())
}
