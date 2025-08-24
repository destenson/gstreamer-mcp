use crate::cli::{OperationalMode, ParsedConfig};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,
    
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
    
    #[serde(default = "default_max_results")]
    pub max_search_results: usize,
    
    #[serde(default)]
    pub operational_mode: OperationalMode,
    
    #[serde(default)]
    pub included_tools: Option<Vec<String>>,
    
    #[serde(default)]
    pub excluded_tools: Option<Vec<String>>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            cache_enabled: default_cache_enabled(),
            cache_ttl_seconds: default_cache_ttl(),
            max_search_results: default_max_results(),
            operational_mode: OperationalMode::default(),
            included_tools: None,
            excluded_tools: None,
        }
    }
}

impl Configuration {
    pub fn load_from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Configuration = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn merge_env_vars(&mut self) {
        if let Ok(val) = std::env::var("GSTREAMER_MCP_CACHE_ENABLED") {
            if let Ok(enabled) = val.parse::<bool>() {
                self.cache_enabled = enabled;
            }
        }
        
        if let Ok(val) = std::env::var("GSTREAMER_MCP_CACHE_TTL") {
            if let Ok(ttl) = val.parse::<u64>() {
                self.cache_ttl_seconds = ttl;
            }
        }
        
        if let Ok(val) = std::env::var("GSTREAMER_MCP_MAX_RESULTS") {
            if let Ok(max) = val.parse::<usize>() {
                self.max_search_results = max;
            }
        }
    }
    
    /// Merge CLI arguments into configuration
    /// Priority: CLI args > env vars > config file > defaults
    pub fn merge_cli_args(&mut self, cli_config: &ParsedConfig) {
        // Override operational mode
        self.operational_mode = cli_config.mode.clone();
        
        // Override tool lists if provided
        if cli_config.included_tools.is_some() {
            self.included_tools = cli_config.included_tools.clone();
        }
        
        if cli_config.excluded_tools.is_some() {
            self.excluded_tools = cli_config.excluded_tools.clone();
        }
    }
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}

fn default_max_results() -> usize {
    100
}