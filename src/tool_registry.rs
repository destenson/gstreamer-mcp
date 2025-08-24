use crate::cli::OperationalMode;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Category of MCP tools
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolCategory {
    Discovery,
    Pipeline,
    Seek,
    Suggestions,
    CodeGeneration,
    PluginDevelopment,
}

/// Metadata for a single tool
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub category: ToolCategory,
    pub description: String,
    pub modes: Vec<OperationalMode>,
}

impl ToolMetadata {
    pub fn new(
        name: impl Into<String>,
        category: ToolCategory,
        description: impl Into<String>,
        modes: Vec<OperationalMode>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            description: description.into(),
            modes,
        }
    }

    /// Check if this tool is available in the given mode
    pub fn is_available_in_mode(&self, mode: &OperationalMode) -> bool {
        mode == &OperationalMode::All || self.modes.contains(mode)
    }
}

/// Registry of all available tools
pub struct ToolRegistry {
    tools: HashMap<String, ToolMetadata>,
}

impl ToolRegistry {
    /// Create and initialize the tool registry with all known tools
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // Element Discovery Tools (PRP-01)
        tools.insert(
            "gst_list_elements".to_string(),
            ToolMetadata::new(
                "gst_list_elements",
                ToolCategory::Discovery,
                "Lists all available GStreamer elements with optional filtering. Accepts name filter and category filter (both optional). Returns element names, descriptions, plugin sources, and rank values. Use to discover available media processing components.",
                vec![OperationalMode::All, OperationalMode::Live, OperationalMode::Dev, OperationalMode::Discovery],
            ),
        );

        tools.insert(
            "gst_inspect_element".to_string(),
            ToolMetadata::new(
                "gst_inspect_element",
                ToolCategory::Discovery,
                "Retrieves detailed information about a specific GStreamer element. Accepts element name (required). Returns properties with types/defaults, pad templates, signals, and classification. Use to understand element capabilities and configuration options.",
                vec![OperationalMode::All, OperationalMode::Live, OperationalMode::Dev, OperationalMode::Discovery],
            ),
        );

        tools.insert(
            "gst_list_plugins".to_string(),
            ToolMetadata::new(
                "gst_list_plugins",
                ToolCategory::Discovery,
                "Lists all available GStreamer plugins. Accepts name filter (optional). Returns plugin names, versions, descriptions, licenses, and contained elements. Use to explore available plugin functionality.",
                vec![OperationalMode::All, OperationalMode::Live, OperationalMode::Dev, OperationalMode::Discovery],
            ),
        );

        tools.insert(
            "gst_search_elements".to_string(),
            ToolMetadata::new(
                "gst_search_elements",
                ToolCategory::Discovery,
                "Searches for GStreamer elements by keyword. Accepts search query (required). Returns relevance-ranked results matching element names, descriptions, and classifications. Use to find elements for specific media processing tasks.",
                vec![OperationalMode::All, OperationalMode::Live, OperationalMode::Dev, OperationalMode::Discovery],
            ),
        );

        // Pipeline Management Tools (PRP-02)
        tools.insert(
            "gst_launch_pipeline".to_string(),
            ToolMetadata::new(
                "gst_launch_pipeline",
                ToolCategory::Pipeline,
                "Creates and launches a GStreamer pipeline from description. Accepts gst-launch syntax, auto-play flag (default: true), and custom ID (optional). Returns pipeline ID and current state. Use to start media processing pipelines.",
                vec![OperationalMode::All, OperationalMode::Live],
            ),
        );

        tools.insert(
            "gst_set_pipeline_state".to_string(),
            ToolMetadata::new(
                "gst_set_pipeline_state",
                ToolCategory::Pipeline,
                "Changes the state of an active pipeline. Accepts pipeline ID and target state (null/ready/paused/playing). Returns new state and transition success status. Use to control pipeline playback and processing.",
                vec![OperationalMode::All, OperationalMode::Live],
            ),
        );

        tools.insert(
            "gst_get_pipeline_status".to_string(),
            ToolMetadata::new(
                "gst_get_pipeline_status",
                ToolCategory::Pipeline,
                "Retrieves current status of a pipeline. Accepts pipeline ID and include_messages flag (optional). Returns state, position, duration, and recent bus messages. Use to monitor pipeline health and playback progress.",
                vec![OperationalMode::All, OperationalMode::Live, OperationalMode::Discovery],
            ),
        );

        tools.insert(
            "gst_stop_pipeline".to_string(),
            ToolMetadata::new(
                "gst_stop_pipeline",
                ToolCategory::Pipeline,
                "Stops and releases resources for a pipeline. Accepts pipeline ID and force flag (optional). Returns cleanup status. Use to properly terminate pipelines and free resources.",
                vec![OperationalMode::All, OperationalMode::Live],
            ),
        );

        tools.insert(
            "gst_list_pipelines".to_string(),
            ToolMetadata::new(
                "gst_list_pipelines",
                ToolCategory::Pipeline,
                "Lists all currently active pipelines. Accepts include_details flag (optional). Returns pipeline IDs, descriptions, states, and creation times. Use to manage multiple concurrent pipelines.",
                vec![OperationalMode::All, OperationalMode::Live, OperationalMode::Discovery],
            ),
        );

        tools.insert(
            "gst_validate_pipeline".to_string(),
            ToolMetadata::new(
                "gst_validate_pipeline",
                ToolCategory::Pipeline,
                "Validates pipeline description syntax without launching. Accepts gst-launch syntax description. Returns validation status and list of elements that would be created. Use to verify pipeline correctness before execution.",
                vec![OperationalMode::All, OperationalMode::Live, OperationalMode::Dev, OperationalMode::Discovery],
            ),
        );

        // Future tools (PRP-03, PRP-04, PRP-05, PRP-06) would be added here
        // For now, we're only including the implemented tools

        Self { tools }
    }

    /// Get all tools available in a specific mode
    pub fn get_tools_for_mode(&self, mode: &OperationalMode) -> Vec<String> {
        self.tools
            .values()
            .filter(|tool| tool.is_available_in_mode(mode))
            .map(|tool| tool.name.clone())
            .collect()
    }

    /// Filter tools based on mode and include/exclude lists
    pub fn filter_tools(
        &self,
        mode: &OperationalMode,
        included: Option<&[String]>,
        excluded: Option<&[String]>,
    ) -> HashSet<String> {
        // Start with tools available in the mode
        let mut available: HashSet<String> = self.get_tools_for_mode(mode).into_iter().collect();

        // If specific tools are included, only use those (if they're available in the mode)
        if let Some(included_tools) = included {
            let included_set: HashSet<String> = included_tools.iter().cloned().collect();
            available = available.intersection(&included_set).cloned().collect();
        }

        // Remove excluded tools
        if let Some(excluded_tools) = excluded {
            for tool in excluded_tools {
                available.remove(tool);
            }
        }

        available
    }

    /// Check if a specific tool is registered
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get metadata for a specific tool
    pub fn get_tool(&self, name: &str) -> Option<&ToolMetadata> {
        self.tools.get(name)
    }

    /// Get all registered tools
    pub fn all_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(registry.has_tool("gst_list_elements"));
        assert!(registry.has_tool("gst_launch_pipeline"));
        assert!(!registry.has_tool("nonexistent_tool"));
    }

    #[test]
    fn test_mode_filtering() {
        let registry = ToolRegistry::new();

        // Discovery mode should have read-only tools
        let discovery_tools = registry.get_tools_for_mode(&OperationalMode::Discovery);
        assert!(discovery_tools.contains(&"gst_list_elements".to_string()));
        assert!(discovery_tools.contains(&"gst_validate_pipeline".to_string()));
        assert!(!discovery_tools.contains(&"gst_launch_pipeline".to_string()));
        assert!(!discovery_tools.contains(&"gst_stop_pipeline".to_string()));

        // Live mode should have pipeline control tools
        let live_tools = registry.get_tools_for_mode(&OperationalMode::Live);
        assert!(live_tools.contains(&"gst_launch_pipeline".to_string()));
        assert!(live_tools.contains(&"gst_stop_pipeline".to_string()));

        // All mode should have everything
        let all_tools = registry.get_tools_for_mode(&OperationalMode::All);
        assert_eq!(all_tools.len(), 10); // We have 10 implemented tools
    }

    #[test]
    fn test_tool_filtering_with_includes_excludes() {
        let registry = ToolRegistry::new();

        // Test with included tools
        let included = vec![
            "gst_list_elements".to_string(),
            "gst_list_plugins".to_string(),
        ];
        let filtered = registry.filter_tools(&OperationalMode::All, Some(&included), None);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains("gst_list_elements"));
        assert!(filtered.contains("gst_list_plugins"));

        // Test with excluded tools
        let excluded = vec!["gst_launch_pipeline".to_string()];
        let filtered = registry.filter_tools(&OperationalMode::Live, None, Some(&excluded));
        assert!(!filtered.contains("gst_launch_pipeline"));
        assert!(filtered.contains("gst_stop_pipeline"));
    }
}
