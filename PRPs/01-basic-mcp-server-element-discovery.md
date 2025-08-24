# PRP: Basic MCP Server Setup & GStreamer Element Discovery

## Overview
This PRP covers the initial setup of a GStreamer MCP server in Rust that provides tools for querying available GStreamer elements and their properties, similar to gst-inspect functionality.

## Context & References

### Existing Patterns to Follow
- **cargo-mcp Implementation**: Reference ../cargo-mcp/src/main.rs for MCP server initialization pattern using rmcp and stdio transport
- **Handler Structure**: Follow ../cargo-mcp/src/handler.rs pattern for implementing ServerHandler trait and tool definitions
- **Tool Router Pattern**: Use the #[tool_router] and #[tool] macros from rmcp as seen in cargo-mcp

### Key Dependencies
- **rmcp**: Version 0.6.0+ with features ["server", "transport-io", "macros"] - see ../cargo-mcp/Cargo.toml
- **gstreamer**: Use local ../gstreamer-rs bindings for programmatic GStreamer access
- **GStreamer Documentation**: https://gstreamer.freedesktop.org/documentation/tools/gst-inspect.html

### GStreamer Element Discovery Concepts
- Elements are the basic building blocks of GStreamer pipelines
- Each element has properties, pad templates, signals, and capabilities
- The GStreamer registry contains all available elements and plugins
- gst-inspect provides detailed information about elements and plugins

## Implementation Blueprint

### Project Structure
```
gstreamer-mcp/
├── Cargo.toml          # Dependencies and project metadata
├── src/
│   ├── main.rs         # Entry point with stdio server setup
│   ├── lib.rs          # Public API exports
│   ├── handler.rs      # GStreamerHandler with tool implementations
│   ├── discovery.rs    # Element discovery and registry interaction
│   ├── error.rs        # Error types following cargo-mcp pattern
│   └── config.rs       # Configuration management (optional initially)
```

### Core Components

#### 1. Main Entry Point (src/main.rs)
- Initialize tracing to stderr (following cargo-mcp pattern)
- Create GStreamerHandler instance
- Start MCP server on stdio transport
- Handle graceful shutdown

#### 2. Handler Implementation (src/handler.rs)
- Implement ServerHandler trait from rmcp
- Use #[tool_router] macro for automatic tool routing
- Define tool parameter structs with JsonSchema derive
- Implement element discovery tools

#### 3. Discovery Module (src/discovery.rs)
- Initialize GStreamer context
- Access element registry programmatically
- Extract element metadata, properties, pad templates
- Format output for MCP responses

### Tool Definitions

#### Tool 1: ListGstElements
- **Description**: "List all available GStreamer elements"
- **Parameters**: 
  - filter: Optional string to filter element names
  - category: Optional string to filter by element category
- **Returns**: List of element names with brief descriptions

#### Tool 2: InspectGstElement
- **Description**: "Get detailed information about a GStreamer element"
- **Parameters**:
  - element_name: Required string specifying the element
- **Returns**: Detailed element information including:
  - Properties with types and descriptions
  - Pad templates (sink/src capabilities)
  - Element classification and rank
  - Plugin information

#### Tool 3: ListGstPlugins
- **Description**: "List all available GStreamer plugins"
- **Parameters**:
  - filter: Optional string to filter plugin names
- **Returns**: List of plugins with their elements

#### Tool 4: SearchGstElements
- **Description**: "Search elements by keyword in name or description"
- **Parameters**:
  - query: Required search string
- **Returns**: Matching elements with relevance ranking

## Implementation Tasks

1. **Project Setup**
   - Initialize Rust project with cargo init
   - Add dependencies to Cargo.toml (rmcp, gstreamer, tokio, serde, etc.)
   - Set up basic project structure

2. **Core MCP Server**
   - Implement main.rs with stdio server setup
   - Create GStreamerHandler struct
   - Implement ServerHandler trait with server info

3. **GStreamer Integration**
   - Initialize GStreamer in discovery module
   - Implement registry access functions
   - Create element information extraction logic

4. **Tool Implementations**
   - Implement gst_list_elements tool
   - Implement gst_inspect_element tool with full property extraction
   - Implement gst_list_plugins tool
   - Implement gst_search_elements with basic keyword matching

5. **Error Handling**
   - Define error types for GStreamer failures
   - Implement proper error conversion to MCP errors
   - Add validation for element names

6. **Testing & Validation**
   - Test element listing functionality
   - Verify element inspection returns correct properties
   - Test search functionality with various queries

## Validation Gates

```bash
# Build and format check
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

# Build the project
cargo build --release

# Run basic tests (once implemented)
cargo test --all-features

# Manual validation - test with MCP client
# Should list available elements
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"gst_list_elements"},"id":1}' | cargo run

# Should inspect videotestsrc element
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"gst_inspect_element","arguments":{"element_name":"videotestsrc"}},"id":2}' | cargo run
```

## Dependencies & Resources

### Required Crates
- rmcp (0.6.0+) - MCP server framework
- gstreamer (from ../gstreamer-rs) - GStreamer bindings
- tokio (1.47+) - Async runtime
- serde (1.0+) - Serialization
- serde_json (1.0+) - JSON handling
- thiserror (2.0+) - Error handling
- anyhow (1.0+) - Error context
- tracing (0.1+) - Logging

### Documentation Links
- MCP Protocol: https://modelcontextprotocol.com/
- rmcp examples: ../modelcontextprotocol--rust-sdk/examples/servers/
- GStreamer Registry API: https://gstreamer.pages.freedesktop.org/gstreamer-rs/stable/latest/docs/gstreamer/
- gst-inspect documentation: https://gstreamer.freedesktop.org/documentation/tools/gst-inspect.html

## Success Criteria

1. MCP server starts successfully and responds to initialization
2. Can list all available GStreamer elements
3. Can inspect individual elements with full property details
4. Search functionality returns relevant results
5. Proper error handling for invalid element names
6. Clean shutdown on termination

## Notes for Implementation

- Start with read-only operations (discovery/inspection) before adding pipeline control
- Use GStreamer's programmatic API rather than parsing CLI output for better reliability
- Follow cargo-mcp's pattern for configuration and security considerations
- Consider caching element information for performance if registry scanning is slow
- Ensure all GStreamer operations are properly initialized and cleaned up

## Confidence Score: 8/10

The implementation path is clear with good reference implementations available. The main complexity lies in properly interfacing with GStreamer's registry and extracting all relevant element information in a structured format suitable for MCP responses.
