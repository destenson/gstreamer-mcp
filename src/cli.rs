use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// GStreamer Model Context Protocol Server
#[derive(Parser, Debug)]
#[command(name = "gstreamer-mcp")]
#[command(about = "GStreamer MCP server for element discovery and pipeline management")]
#[command(version)]
pub struct Cli {
    /// Operational mode for the server
    #[arg(short, long, value_enum, default_value = "all", env = "GSTREAMER_MCP_MODE")]
    pub mode: OperationalMode,
    
    /// Run in REPL mode for interactive testing
    #[arg(short, long)]
    pub repl: bool,
    
    /// Specific tools to enable (comma-separated)
    #[arg(long, value_delimiter = ',', env = "GSTREAMER_MCP_TOOLS")]
    pub tools: Option<Vec<String>>,
    
    /// Tools to exclude (comma-separated)
    #[arg(long, value_delimiter = ',', conflicts_with = "tools", env = "GSTREAMER_MCP_EXCLUDE_TOOLS")]
    pub exclude_tools: Option<Vec<String>>,
    
    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    
    /// Verbose output (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    
    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationalMode {
    /// All tools enabled (default)
    All,
    /// Live operations mode (pipeline control, monitoring)
    Live,
    /// Development mode (code generation, suggestions)
    Dev,
    /// Discovery mode (read-only operations)
    Discovery,
}

impl Default for OperationalMode {
    fn default() -> Self {
        OperationalMode::All
    }
}

/// Parsed configuration from CLI arguments
#[derive(Debug, Clone)]
pub struct ParsedConfig {
    pub mode: OperationalMode,
    pub repl: bool,
    pub included_tools: Option<Vec<String>>,
    pub excluded_tools: Option<Vec<String>>,
    pub config_path: Option<PathBuf>,
    pub verbose_level: u8,
    pub no_color: bool,
}

impl Cli {
    /// Parse command-line arguments and return configuration
    pub fn parse_args() -> ParsedConfig {
        let cli = Cli::parse();
        
        ParsedConfig {
            mode: cli.mode,
            repl: cli.repl,
            included_tools: cli.tools,
            excluded_tools: cli.exclude_tools,
            config_path: cli.config,
            verbose_level: cli.verbose,
            no_color: cli.no_color,
        }
    }
    
    /// Parse with environment variable fallback
    pub fn parse_with_env() -> ParsedConfig {
        // First try CLI args, which will also pick up env vars via clap's env feature
        let config = Self::parse_args();
        
        // Additional env var handling if needed
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode() {
        assert_eq!(OperationalMode::default(), OperationalMode::All);
    }
    
    #[test]
    fn test_mode_serialization() {
        let mode = OperationalMode::Live;
        let serialized = serde_json::to_string(&mode).unwrap();
        assert_eq!(serialized, "\"live\"");
        
        let deserialized: OperationalMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, OperationalMode::Live);
    }
}