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
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            cache_enabled: default_cache_enabled(),
            cache_ttl_seconds: default_cache_ttl(),
            max_search_results: default_max_results(),
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