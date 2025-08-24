# GStreamer MCP Server

A Model Context Protocol (MCP) server that provides GStreamer element discovery, inspection, and pipeline management capabilities to AI assistants and other MCP clients.

## Features

This MCP server implements comprehensive tools for GStreamer element discovery and pipeline management:

### Element Discovery Tools
1. **gst_list_elements** - List all available GStreamer elements with optional filtering
2. **gst_inspect_element** - Get detailed information about a specific GStreamer element including properties, pad templates, and signals
3. **gst_list_plugins** - List all available GStreamer plugins and their elements
4. **gst_search_elements** - Search for elements by keyword with relevance ranking

### Pipeline Management Tools
5. **gst_launch_pipeline** - Launch a GStreamer pipeline from a description string
6. **gst_set_pipeline_state** - Change the state of a running pipeline (null, ready, paused, playing)
7. **gst_get_pipeline_status** - Get current status, position, and messages for a pipeline
8. **gst_stop_pipeline** - Stop and cleanup a pipeline
9. **gst_list_pipelines** - List all active pipelines
10. **gst_validate_pipeline** - Validate a pipeline description without launching it

## Installation

### Prerequisites

- Rust 1.70 or later
- GStreamer 1.16 or later installed on your system
- Windows, Linux, or macOS

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/gstreamer-mcp.git
cd gstreamer-mcp

# Build the project
cargo build --release

# The binary will be available at target/release/gstreamer-mcp
```

## Usage

### As an MCP Server

The server communicates via stdio and implements the Model Context Protocol:

```bash
# Run the server
./target/release/gstreamer-mcp

# Or with custom configuration
GSTREAMER_MCP_CONFIG=custom-config.toml ./target/release/gstreamer-mcp
```

### Configuration

Create a `gstreamer-mcp.toml` file to customize the server behavior:

```toml
# Enable/disable caching of element discovery results
cache_enabled = true

# Cache time-to-live in seconds
cache_ttl_seconds = 300

# Maximum number of search results to return
max_search_results = 100
```

You can also use environment variables:
- `GSTREAMER_MCP_CACHE_ENABLED` - Enable/disable caching (true/false)
- `GSTREAMER_MCP_CACHE_TTL` - Cache TTL in seconds
- `GSTREAMER_MCP_MAX_RESULTS` - Maximum search results

### Testing

Run the included test script to verify the server is working:

```bash
# Windows
scripts\test_server.bat

# Linux/macOS
scripts/test_server.sh
```

## MCP Tools

### gst_list_elements

Lists all available GStreamer elements with optional filtering.

**Parameters:**
- `filter` (optional): Filter element names containing this string
- `category` (optional): Filter by element classification/category

**Example:**
```json
{
  "name": "gst_list_elements",
  "arguments": {
    "filter": "video",
    "category": "Source"
  }
}
```

### gst_inspect_element

Get detailed information about a specific GStreamer element.

**Parameters:**
- `element_name` (required): Name of the element to inspect

**Example:**
```json
{
  "name": "gst_inspect_element",
  "arguments": {
    "element_name": "videotestsrc"
  }
}
```

Returns:
- Element description and classification
- All properties with types, descriptions, and flags
- Pad templates showing input/output capabilities
- Supported signals (if any)

### gst_launch_pipeline

Launch a GStreamer pipeline from a description string.

**Parameters:**
- `pipeline_description` (required): Pipeline description in gst-launch syntax
- `auto_play` (optional): Whether to start the pipeline immediately (default: true)
- `pipeline_id` (optional): Custom pipeline ID (auto-generated if not provided)

**Example:**
```json
{
  "name": "gst_launch_pipeline",
  "arguments": {
    "pipeline_description": "videotestsrc ! autovideosink",
    "auto_play": true
  }
}
```

Returns:
- Pipeline ID for future operations
- Current pipeline state
- Launch success status

### gst_set_pipeline_state

Change the state of a running pipeline.

**Parameters:**
- `pipeline_id` (required): Pipeline identifier
- `state` (required): Target state ("null", "ready", "paused", "playing")

**Example:**
```json
{
  "name": "gst_set_pipeline_state",
  "arguments": {
    "pipeline_id": "pipeline-abc123",
    "state": "paused"
  }
}
```

### gst_get_pipeline_status

Get current status and information about a pipeline.

**Parameters:**
- `pipeline_id` (required): Pipeline identifier
- `include_messages` (optional): Include recent bus messages (default: false)

**Example:**
```json
{
  "name": "gst_get_pipeline_status",
  "arguments": {
    "pipeline_id": "pipeline-abc123",
    "include_messages": true
  }
}
```

Returns:
- Current state and pending state
- Position and duration (if available)
- Error and warning counts
- Creation time and last state change
- Recent bus messages (if requested)

### gst_stop_pipeline

Stop and cleanup a pipeline.

**Parameters:**
- `pipeline_id` (required): Pipeline identifier
- `force` (optional): Force termination (default: false)

**Example:**
```json
{
  "name": "gst_stop_pipeline",
  "arguments": {
    "pipeline_id": "pipeline-abc123"
  }
}
```

### gst_list_pipelines

List all active pipelines.

**Parameters:**
- `include_details` (optional): Include detailed information (default: false)

**Example:**
```json
{
  "name": "gst_list_pipelines",
  "arguments": {
    "include_details": true
  }
}
```

### gst_validate_pipeline

Validate a pipeline description without launching it.

**Parameters:**
- `pipeline_description` (required): Pipeline description to validate

**Example:**
```json
{
  "name": "gst_validate_pipeline",
  "arguments": {
    "pipeline_description": "videotestsrc ! autovideosink"
  }
}
```

Returns:
- Validation status (valid/invalid)
- List of elements that would be created
- Error details (if invalid)

### gst_list_plugins

Lists all available GStreamer plugins.

**Parameters:**
- `filter` (optional): Filter plugin names containing this string

**Example:**
```json
{
  "name": "gst_list_plugins",
  "arguments": {
    "filter": "core"
  }
}
```

### gst_search_elements

Search for elements by keyword with intelligent ranking.

**Parameters:**
- `query` (required): Search term to match against element names, descriptions, and classifications

**Example:**
```json
{
  "name": "gst_search_elements",
  "arguments": {
    "query": "encoder"
  }
}
```

## Integration with AI Assistants

### Claude Desktop

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "gstreamer": {
      "command": "C:\\path\\to\\gstreamer-mcp.exe"
    }
  }
}
```

### Custom MCP Clients

Connect to the server via stdio and follow the MCP protocol specification. The server provides:
- Protocol version: 2024-11-05
- Tools capability enabled
- Comprehensive element discovery and inspection

## Development

### Project Structure

```
gstreamer-mcp/
├── src/
│   ├── main.rs         # Entry point and server initialization
│   ├── handler.rs      # MCP request handler and tool implementations
│   ├── discovery.rs    # GStreamer element discovery logic
│   ├── pipeline.rs     # Pipeline management and state tracking
│   ├── bus_handler.rs  # GStreamer bus message handling
│   ├── error.rs        # Error types and conversions
│   ├── config.rs       # Configuration management
│   └── lib.rs          # Library exports
├── scripts/
│   └── test_server.bat # Test script for Windows
└── Cargo.toml          # Project dependencies
```

### Adding New Tools

1. Define the tool parameters struct in `handler.rs` with JsonSchema derive
2. Implement the tool method with `#[tool]` attribute
3. Add logic in appropriate modules:
   - Element discovery: `discovery.rs`
   - Pipeline management: `pipeline.rs`
   - Bus message handling: `bus_handler.rs`
4. Update this README with the new tool documentation

## License

[Your chosen license]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Troubleshooting

### GStreamer Not Found

Ensure GStreamer is installed and available in your system PATH:
- Windows: Install GStreamer from https://gstreamer.freedesktop.org/download/
- Linux: Use your package manager (apt, dnf, pacman, etc.)
- macOS: Use Homebrew: `brew install gstreamer`

### Server Crashes on Startup

Check the error log by running with stderr visible:
```bash
./gstreamer-mcp 2>error.log
```

Common issues:
- GStreamer not properly installed
- Missing GStreamer plugins
- Incompatible GStreamer version

### No Elements Found

This usually indicates GStreamer plugins are not properly installed. Verify with:
```bash
gst-inspect-1.0
```

## Related Projects

- [Model Context Protocol](https://modelcontextprotocol.com/)
- [GStreamer](https://gstreamer.freedesktop.org/)
- [gstreamer-rs](https://gitlab.freedesktop.org/gstreamer/gstreamer-rs)