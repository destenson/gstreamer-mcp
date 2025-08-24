use crate::config::Configuration;
use crate::discovery::{
    discover_all_elements, discover_all_plugins, inspect_element, search_elements, DiscoveryCache,
};
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars, schemars::JsonSchema,
    tool, tool_handler, tool_router,
    ErrorData as McpError, ServerHandler,
};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListElementsParams {
    #[schemars(description = "Optional filter to match element names")]
    pub filter: Option<String>,
    #[schemars(description = "Optional category to filter elements by classification")]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InspectElementParams {
    #[schemars(description = "Name of the GStreamer element to inspect")]
    pub element_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListPluginsParams {
    #[schemars(description = "Optional filter to match plugin names")]
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SearchElementsParams {
    #[schemars(description = "Search query to match against element names and descriptions")]
    pub query: String,
}

#[derive(Clone)]
pub struct GStreamerHandler {
    pub config: Arc<RwLock<Configuration>>,
    pub cache: Arc<DiscoveryCache>,
    tool_router: ToolRouter<GStreamerHandler>,
}

#[tool_router]
impl GStreamerHandler {
    pub async fn new() -> crate::Result<Self> {
        let config = Configuration::default();
        let cache = DiscoveryCache::new();

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            cache: Arc::new(cache),
            tool_router: Self::tool_router(),
        })
    }

    pub async fn with_config(config: Configuration) -> crate::Result<Self> {
        let cache = DiscoveryCache::new();

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            cache: Arc::new(cache),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "List all available GStreamer elements")]
    async fn gst_list_elements(
        &self,
        Parameters(params): Parameters<ListElementsParams>,
    ) -> Result<CallToolResult, McpError> {
        let elements = if self.config.read().await.cache_enabled {
            self.cache.get_elements().await
        } else {
            discover_all_elements()
        }
        .map_err(Into::<McpError>::into)?;

        let mut filtered_elements = elements;

        // Apply name filter if provided
        if let Some(filter) = params.filter {
            let filter_lower = filter.to_lowercase();
            filtered_elements.retain(|e| e.name.to_lowercase().contains(&filter_lower));
        }

        // Apply category filter if provided
        if let Some(category) = params.category {
            let category_lower = category.to_lowercase();
            filtered_elements
                .retain(|e| e.classification.to_lowercase().contains(&category_lower));
        }

        let output = if filtered_elements.is_empty() {
            "No elements found matching the criteria.".to_string()
        } else {
            let mut output = format!("Found {} elements:\n\n", filtered_elements.len());
            for element in filtered_elements {
                output.push_str(&format!(
                    "- {} ({})\n  Plugin: {}, Rank: {}\n",
                    element.name, element.description, element.plugin_name, element.rank
                ));
            }
            output
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Get detailed information about a GStreamer element")]
    async fn gst_inspect_element(
        &self,
        Parameters(params): Parameters<InspectElementParams>,
    ) -> Result<CallToolResult, McpError> {
        let info = inspect_element(&params.element_name).map_err(Into::<McpError>::into)?;

        let mut output = format!("Element: {}\n", info.name);
        output.push_str(&format!("Description: {}\n", info.description));
        output.push_str(&format!("Plugin: {}\n", info.plugin_name));
        output.push_str(&format!("Rank: {}\n", info.rank));
        output.push_str(&format!("Classification: {}\n\n", info.classification));

        // Properties section
        if !info.properties.is_empty() {
            output.push_str("Properties:\n");
            for prop in &info.properties {
                output.push_str(&format!(
                    "  {} ({}) [{}]\n    {}\n",
                    prop.name,
                    prop.type_name,
                    prop.flags.join(", "),
                    prop.description
                ));
                if let Some(default) = &prop.default_value {
                    output.push_str(&format!("    Default: {}\n", default));
                }
            }
            output.push('\n');
        }

        // Pad templates section
        if !info.pad_templates.is_empty() {
            output.push_str("Pad Templates:\n");
            for pad in &info.pad_templates {
                output.push_str(&format!(
                    "  {} ({}, {})\n    Caps: {}\n",
                    pad.name, pad.direction, pad.presence, pad.caps
                ));
            }
            output.push('\n');
        }

        // Signals section
        if !info.signals.is_empty() {
            output.push_str("Signals:\n");
            for signal in &info.signals {
                output.push_str(&format!(
                    "  {} -> {}\n    Parameters: {}\n",
                    signal.name,
                    signal.return_type,
                    signal.parameters.join(", ")
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "List all available GStreamer plugins")]
    async fn gst_list_plugins(
        &self,
        Parameters(params): Parameters<ListPluginsParams>,
    ) -> Result<CallToolResult, McpError> {
        let plugins = if self.config.read().await.cache_enabled {
            self.cache.get_plugins().await
        } else {
            discover_all_plugins()
        }
        .map_err(Into::<McpError>::into)?;

        let mut filtered_plugins = plugins;

        // Apply filter if provided
        if let Some(filter) = params.filter {
            let filter_lower = filter.to_lowercase();
            filtered_plugins.retain(|p| p.name.to_lowercase().contains(&filter_lower));
        }

        let output = if filtered_plugins.is_empty() {
            "No plugins found matching the criteria.".to_string()
        } else {
            let mut output = format!("Found {} plugins:\n\n", filtered_plugins.len());
            for plugin in filtered_plugins {
                output.push_str(&format!(
                    "- {} (v{}) - {}\n  License: {}, Source: {}\n",
                    plugin.name, plugin.version, plugin.description, plugin.license, plugin.source
                ));
                if !plugin.elements.is_empty() {
                    output.push_str(&format!(
                        "  Elements: {}\n",
                        plugin.elements.join(", ")
                    ));
                }
                output.push('\n');
            }
            output
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Search GStreamer elements by keyword")]
    async fn gst_search_elements(
        &self,
        Parameters(params): Parameters<SearchElementsParams>,
    ) -> Result<CallToolResult, McpError> {
        let config = self.config.read().await;
        let max_results = config.max_search_results;

        let results =
            search_elements(&params.query, max_results).map_err(Into::<McpError>::into)?;

        let output = if results.is_empty() {
            format!("No elements found matching '{}'", params.query)
        } else {
            let mut output = format!(
                "Found {} elements matching '{}':\n\n",
                results.len(),
                params.query
            );
            for element in results {
                output.push_str(&format!(
                    "- {} ({})\n  Plugin: {}, Classification: {}\n",
                    element.name,
                    element.description,
                    element.plugin_name,
                    element.classification
                ));
            }
            output
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

#[tool_handler]
impl ServerHandler for GStreamerHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "gstreamer-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "This server provides GStreamer element discovery and inspection tools. \
                 You can list elements, inspect their properties, list plugins, and search for elements by keyword."
                    .to_string(),
            ),
        }
    }
}