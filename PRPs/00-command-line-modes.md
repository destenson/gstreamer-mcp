# PRP: Command-Line Options and Operational Modes

## Overview
This PRP defines command-line argument handling and operational modes for the GStreamer MCP server, allowing users to run the server with different tool sets based on their use case (live operations vs development).

## Context & References

### Motivation
Different users have different needs:
- **Live/Production Users**: Need pipeline control, monitoring, element discovery
- **Development Users**: Need code generation, plugin development assistance
- **Admin Users**: May need all tools including debugging and system management

### Existing Patterns
- Reference cargo-mcp's configuration approach via environment variables
- Standard Rust CLI patterns using clap or structopt
- MCP servers typically run as `command | mcp-server` or via stdio

## Implementation Blueprint

### Command-Line Interface Design

```bash
# Default mode (all tools)
gstreamer-mcp

# Live operations mode (runtime tools only)
gstreamer-mcp --mode live
gstreamer-mcp -m live

# Development mode (development tools only)  
gstreamer-mcp --mode dev
gstreamer-mcp -m dev

# Discovery only mode (read-only operations)
gstreamer-mcp --mode discovery

# Custom tool selection
gstreamer-mcp --tools discovery,pipeline
gstreamer-mcp --exclude-tools development,suggestions

# Configuration file
gstreamer-mcp --config config.toml

# Verbose output for debugging
gstreamer-mcp -v
gstreamer-mcp -vv  # More verbose
```

### Operational Modes

#### 1. **Live Mode** (`--mode live`)
Focus on pipeline operations and monitoring
- **Enabled Tools**:
  - Element Discovery (ListGstElements, InspectGstElement, etc.)
  - Pipeline Management (LaunchPipeline, SetPipelineState, etc.)
  - Pipeline Monitoring (GetPipelineStatus, ListGstPipelines)
- **Disabled Tools**:
  - Code generation tools
  - Plugin development tools
  - Programming assistants

#### 2. **Development Mode** (`--mode dev`)
Focus on code generation and plugin development
- **Enabled Tools**:
  - Programming Assistant (GeneratePipelineCode, ConvertLaunchToCode, etc.)
  - Plugin Development (GenerateGstElement, GenerateElementProperty, etc.)
  - Code explanation tools
  - Element suggestions and similarity
- **Disabled Tools**:
  - Live pipeline control (for safety)
  - Pipeline state changes

#### 3. **Discovery Mode** (`--mode discovery`)
Read-only operations for exploration
- **Enabled Tools**:
  - Element Discovery (all read-only tools)
  - Element Suggestions
  - Search and autocomplete
  - ValidatePipeline (read-only validation)
- **Disabled Tools**:
  - Pipeline launching
  - State changes
  - Code generation

#### 4. **All Mode** (default)
All tools enabled - useful for development and testing
- **Enabled Tools**: Everything
- **Use Case**: Development environments, testing

### Implementation Structure

```rust
// src/cli.rs - Command-line argument parsing
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "gstreamer-mcp")]
#[command(about = "GStreamer Model Context Protocol Server")]
struct Cli {
    /// Operational mode
    #[arg(short, long, value_enum, default_value = "all")]
    mode: Mode,
    
    /// Specific tools to enable (comma-separated)
    #[arg(long, value_delimiter = ',')]
    tools: Option<Vec<String>>,
    
    /// Tools to exclude (comma-separated)
    #[arg(long, value_delimiter = ',', conflicts_with = "tools")]
    exclude_tools: Option<Vec<String>>,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Verbose output (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    
    /// Disable colored output
    #[arg(long)]
    no_color: bool,
}

#[derive(ValueEnum, Clone, Debug)]
enum Mode {
    All,
    Live,
    Dev,
    Discovery,
}
```

### Configuration File Format

```toml
# config.toml
[server]
mode = "live"
verbose = 2

[tools]
# Enable specific tool categories
enabled = ["discovery", "pipeline"]
# Or disable specific tools
disabled = ["GenerateGstElement", "GeneratePipelineCode"]

[pipeline]
# Safety limits for live mode
max_concurrent_pipelines = 10
allow_network_elements = false
allow_file_access = true
allowed_paths = ["/media", "/tmp"]

[discovery]
# Cache element registry for performance
cache_enabled = true
cache_duration = 3600  # seconds

[development]
# Code generation settings
default_language = "rust"
include_comments = true
include_error_handling = true
```

### Tool Registry System

```rust
// src/tool_registry.rs
pub struct ToolRegistry {
    tools: HashMap<String, ToolMetadata>,
    enabled_tools: HashSet<String>,
}

pub struct ToolMetadata {
    pub name: String,
    pub category: ToolCategory,
    pub description: String,
    pub mode_availability: Vec<Mode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToolCategory {
    Discovery,      // Read-only element discovery
    Pipeline,       // Pipeline control and management
    Monitoring,     // Pipeline status and monitoring
    Suggestions,    // Element suggestions and search
    Development,    // Code generation
    Plugin,         // Plugin development
}

impl ToolRegistry {
    pub fn new_for_mode(mode: Mode) -> Self {
        let mut registry = Self::new_with_all_tools();
        registry.filter_by_mode(mode);
        registry
    }
    
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        self.enabled_tools.contains(tool_name)
    }
}
```

### Handler Integration

```rust
// src/handler.rs modifications
impl GStreamerHandler {
    pub async fn with_config(config: Configuration) -> Result<Self> {
        let tool_registry = ToolRegistry::new_for_mode(config.mode);
        // ... initialization
    }
    
    // Tool implementations check if enabled
    #[tool(description = "Launch a GStreamer pipeline")]
    async fn launch_pipeline(&self, params: LaunchParams) -> Result<CallToolResult> {
        if !self.tool_registry.is_tool_enabled("LaunchPipeline") {
            return Err(McpError::new(
                ErrorCode::MethodNotFound,
                "Tool 'LaunchPipeline' is not available in current mode"
            ));
        }
        // ... implementation
    }
}
```

## Implementation Tasks

1. **CLI Argument Parsing**
   - Add clap dependency
   - Create cli.rs module
   - Parse command-line arguments
   - Handle configuration file loading

2. **Tool Registry**
   - Create tool metadata system
   - Implement mode-based filtering
   - Add runtime tool enable/disable

3. **Configuration System**
   - Define configuration structure
   - Implement TOML parsing
   - Merge CLI args with config file
   - Add environment variable support

4. **Mode Implementation**
   - Define tool sets for each mode
   - Implement safety restrictions
   - Add mode-specific limits

5. **Handler Integration**
   - Pass configuration to handler
   - Check tool availability at runtime
   - Return appropriate errors for disabled tools

6. **Documentation**
   - Document each mode's purpose
   - Provide example configurations
   - Add mode selection guide

## Validation Gates

```bash
# Test different modes
cargo run -- --mode live
cargo run -- --mode dev
cargo run -- --mode discovery

# Test tool selection
cargo run -- --tools discovery,pipeline
cargo run -- --exclude-tools development

# Test configuration file
cargo run -- --config example-config.toml

# Verify tool availability in each mode
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run -- --mode live
```

## Security Considerations

### Live Mode Restrictions
- Limit concurrent pipelines
- Restrict file system access
- Disable network elements by default
- Add resource usage limits

### Development Mode
- Warn about unsafe operations
- Sandbox code generation output
- Validate generated code syntax

## Dependencies

### Additional Crates
- clap (4.0+) - Command-line argument parsing
- toml (0.8+) - Configuration file parsing
- directories (5.0+) - Standard config paths

## Success Criteria

1. Server starts with specified mode
2. Only appropriate tools available per mode
3. Configuration file properly loaded
4. Tool availability correctly reported
5. Clear error messages for disabled tools
6. Mode restrictions enforced

## Example Usage Scenarios

### Production Media Server
```bash
# Restrict to live operations only
gstreamer-mcp --mode live --config /etc/gstreamer-mcp/production.toml
```

### Development Workstation
```bash
# Full access for development
gstreamer-mcp --mode dev --verbose
```

### CI/CD Pipeline
```bash
# Discovery only for validation
gstreamer-mcp --mode discovery --no-color
```

### Custom Tool Selection
```bash
# Only element discovery and suggestions
gstreamer-mcp --tools discovery,suggestions
```

## Confidence Score: 9/10

Command-line parsing and mode selection is straightforward with clap. The tool registry pattern provides clean separation of concerns and makes it easy to enable/disable tools at runtime. This approach provides flexibility while maintaining safety in production environments.