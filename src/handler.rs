use crate::config::Configuration;
use crate::discovery::{
    discover_all_elements, discover_all_plugins, inspect_element, search_elements, DiscoveryCache,
};
use crate::pipeline::{PipelineManager, validate_pipeline_description};
use crate::tool_registry::ToolRegistry;
use gstreamer as gst;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{*, ErrorCode},
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

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LaunchPipelineParams {
    #[schemars(description = "Pipeline description in gst-launch syntax")]
    pub pipeline_description: String,
    #[schemars(description = "Whether to start the pipeline immediately")]
    pub auto_play: Option<bool>,
    #[schemars(description = "Optional custom pipeline ID")]
    pub pipeline_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SetPipelineStateParams {
    #[schemars(description = "Pipeline identifier")]
    pub pipeline_id: String,
    #[schemars(description = "Target state (null, ready, paused, playing)")]
    pub state: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetPipelineStatusParams {
    #[schemars(description = "Pipeline identifier")]
    pub pipeline_id: String,
    #[schemars(description = "Include recent bus messages")]
    pub include_messages: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StopPipelineParams {
    #[schemars(description = "Pipeline identifier")]
    pub pipeline_id: String,
    #[schemars(description = "Force termination")]
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListGstPipelinesParams {
    #[schemars(description = "Include detailed information")]
    pub include_details: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ValidatePipelineParams {
    #[schemars(description = "Pipeline description to validate")]
    pub pipeline_description: String,
}

#[derive(Clone)]
pub struct GStreamerHandler {
    pub config: Arc<RwLock<Configuration>>,
    pub cache: Arc<DiscoveryCache>,
    pub pipeline_manager: Arc<PipelineManager>,
    pub tool_registry: Arc<ToolRegistry>,
    pub enabled_tools: Arc<RwLock<std::collections::HashSet<String>>>,
    tool_router: ToolRouter<GStreamerHandler>,
}

#[tool_router]
impl GStreamerHandler {
    pub async fn new() -> crate::Result<Self> {
        let config = Configuration::default();
        let cache = DiscoveryCache::new();
        let pipeline_manager = PipelineManager::new(10); // Max 10 concurrent pipelines
        let tool_registry = Arc::new(ToolRegistry::new());
        
        // Get enabled tools based on default configuration
        let enabled_tools = tool_registry.filter_tools(
            &config.operational_mode,
            config.included_tools.as_deref(),
            config.excluded_tools.as_deref(),
        );

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            cache: Arc::new(cache),
            pipeline_manager: Arc::new(pipeline_manager),
            tool_registry,
            enabled_tools: Arc::new(RwLock::new(enabled_tools)),
            tool_router: Self::tool_router(),
        })
    }

    pub async fn with_config(config: Configuration) -> crate::Result<Self> {
        let cache = DiscoveryCache::new();
        let pipeline_manager = PipelineManager::new(10); // Max 10 concurrent pipelines
        let tool_registry = Arc::new(ToolRegistry::new());
        
        // Get enabled tools based on configuration
        let enabled_tools = tool_registry.filter_tools(
            &config.operational_mode,
            config.included_tools.as_deref(),
            config.excluded_tools.as_deref(),
        );

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            cache: Arc::new(cache),
            pipeline_manager: Arc::new(pipeline_manager),
            tool_registry,
            enabled_tools: Arc::new(RwLock::new(enabled_tools)),
            tool_router: Self::tool_router(),
        })
    }
    
    /// Check if a tool is enabled based on current configuration
    async fn is_tool_enabled(&self, tool_name: &str) -> bool {
        self.enabled_tools.read().await.contains(tool_name)
    }

    #[tool(description = "List all available GStreamer elements")]
    async fn gst_list_elements(
        &self,
        Parameters(params): Parameters<ListElementsParams>,
    ) -> Result<CallToolResult, McpError> {
        // Check if tool is enabled
        if !self.is_tool_enabled("gst_list_elements").await {
            return Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("Tool 'gst_list_elements' is not available in the current mode"),
                None::<serde_json::Value>,
            ));
        }
        
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

    #[tool(description = "Launch a GStreamer pipeline from description")]
    async fn gst_launch_pipeline(
        &self,
        Parameters(params): Parameters<LaunchPipelineParams>,
    ) -> Result<CallToolResult, McpError> {
        // Check if tool is enabled
        if !self.is_tool_enabled("gst_launch_pipeline").await {
            return Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("Tool 'gst_launch_pipeline' is not available in the current mode"),
                None::<serde_json::Value>,
            ));
        }
        
        // Create the pipeline
        let pipeline_id = self.pipeline_manager
            .create_pipeline(&params.pipeline_description, params.pipeline_id)
            .map_err(Into::<McpError>::into)?;
        
        // Auto-play if requested (default is true)
        let auto_play = params.auto_play.unwrap_or(true);
        if auto_play {
            let state = self.pipeline_manager
                .set_pipeline_state(&pipeline_id, gst::State::Playing)
                .map_err(Into::<McpError>::into)?;
            
            let output = format!(
                "Pipeline '{}' launched successfully.\nState: {:?}\nDescription: {}",
                pipeline_id, state, params.pipeline_description
            );
            Ok(CallToolResult::success(vec![Content::text(output)]))
        } else {
            let output = format!(
                "Pipeline '{}' created successfully in NULL state.\nDescription: {}",
                pipeline_id, params.pipeline_description
            );
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
    }

    #[tool(description = "Change the state of a running pipeline")]
    async fn gst_set_pipeline_state(
        &self,
        Parameters(params): Parameters<SetPipelineStateParams>,
    ) -> Result<CallToolResult, McpError> {
        // Check if tool is enabled
        if !self.is_tool_enabled("gst_set_pipeline_state").await {
            return Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                format!("Tool 'gst_set_pipeline_state' is not available in the current mode"),
                None::<serde_json::Value>,
            ));
        }
        
        let state = match params.state.to_lowercase().as_str() {
            "null" => gst::State::Null,
            "ready" => gst::State::Ready,
            "paused" => gst::State::Paused,
            "playing" => gst::State::Playing,
            _ => return Err(McpError {
                code: rmcp::model::ErrorCode(-32003),
                message: format!("Invalid state '{}'. Must be one of: null, ready, paused, playing", params.state).into(),
                data: None,
            }),
        };
        
        let current_state = self.pipeline_manager
            .set_pipeline_state(&params.pipeline_id, state)
            .map_err(Into::<McpError>::into)?;
        
        let output = format!(
            "Pipeline '{}' state changed to {:?}",
            params.pipeline_id, current_state
        );
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Get current status of a pipeline")]
    async fn gst_get_pipeline_status(
        &self,
        Parameters(params): Parameters<GetPipelineStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        let status = self.pipeline_manager
            .get_pipeline_status(&params.pipeline_id)
            .map_err(Into::<McpError>::into)?;
        
        let mut output = format!(
            "Pipeline: {}\nDescription: {}\nState: {}\n",
            status.id, status.description, status.state
        );
        
        if let Some(pending) = status.pending_state {
            output.push_str(&format!("Pending State: {}\n", pending));
        }
        
        if status.position >= 0 {
            output.push_str(&format!("Position: {} ns\n", status.position));
        }
        if status.duration >= 0 {
            output.push_str(&format!("Duration: {} ns\n", status.duration));
        }
        
        output.push_str(&format!("Errors: {}, Warnings: {}\n", status.error_count, status.warning_count));
        output.push_str(&format!("Created: {}\nLast State Change: {}\n", 
            status.created_at, status.last_state_change));
        
        // Include messages if requested
        if params.include_messages.unwrap_or(false) {
            let messages = self.pipeline_manager.get_bus_messages(&params.pipeline_id, 10);
            if !messages.is_empty() {
                output.push_str("\nRecent Messages:\n");
                for msg in messages {
                    output.push_str(&format!("  [{}] {}: {}\n", 
                        msg.timestamp, msg.message_type, msg.message));
                }
            }
        }
        
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Stop and cleanup a pipeline")]
    async fn gst_stop_pipeline(
        &self,
        Parameters(params): Parameters<StopPipelineParams>,
    ) -> Result<CallToolResult, McpError> {
        // First set to NULL state
        let _ = self.pipeline_manager
            .set_pipeline_state(&params.pipeline_id, gst::State::Null);
        
        // Then remove the pipeline
        self.pipeline_manager
            .remove_pipeline(&params.pipeline_id)
            .map_err(Into::<McpError>::into)?;
        
        let output = format!("Pipeline '{}' stopped and removed successfully", params.pipeline_id);
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "List all active pipelines")]
    async fn gst_list_pipelines(
        &self,
        Parameters(params): Parameters<ListGstPipelinesParams>,
    ) -> Result<CallToolResult, McpError> {
        let pipelines = self.pipeline_manager.list_pipelines();
        
        if pipelines.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text("No active pipelines".to_string())]));
        }
        
        let mut output = format!("Active pipelines: {}\n\n", pipelines.len());
        
        for pipeline in pipelines {
            if params.include_details.unwrap_or(false) {
                output.push_str(&format!(
                    "ID: {}\n  Description: {}\n  State: {}\n  Created: {}\n  Errors: {}, Warnings: {}\n\n",
                    pipeline.id, pipeline.description, pipeline.state, 
                    pipeline.created_at, pipeline.error_count, pipeline.warning_count
                ));
            } else {
                output.push_str(&format!("- {} ({})\n", pipeline.id, pipeline.state));
            }
        }
        
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Validate a pipeline description without launching")]
    async fn gst_validate_pipeline(
        &self,
        Parameters(params): Parameters<ValidatePipelineParams>,
    ) -> Result<CallToolResult, McpError> {
        match validate_pipeline_description(&params.pipeline_description) {
            Ok(elements) => {
                let mut output = format!(
                    "Pipeline description is valid!\n\nElements that would be created ({}):\n",
                    elements.len()
                );
                for element in elements {
                    output.push_str(&format!("- {}\n", element));
                }
                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            Err(e) => {
                let output = format!("Pipeline validation failed:\n{}", e);
                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
        }
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
                "This server provides GStreamer element discovery, inspection, and pipeline management tools. \
                 You can list elements, inspect their properties, list plugins, search for elements by keyword, \
                 launch pipelines, control pipeline states, and monitor pipeline status."
                    .to_string(),
            ),
        }
    }
}