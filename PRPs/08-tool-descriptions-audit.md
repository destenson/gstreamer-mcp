# PRP: Tool Descriptions Audit and Enhancement

## Status
- **Date**: 2025-01-23
- **Type**: Documentation Enhancement
- **Priority**: High
- **Estimated Effort**: 3-5 hours

## Context

The current MCP tool descriptions in the gstreamer-mcp server are too minimal and lack the informative detail needed for AI agents to effectively understand and utilize the tools. Testing shows descriptions like "List all available GStreamer elements" provide insufficient context about parameters, outputs, and use cases.

### Current State Analysis

#### Problems Identified:
1. **Minimal descriptions**: Single-line descriptions that don't explain what the tool actually returns or how to use it
2. **Missing parameter context**: No indication of optional vs required parameters in descriptions
3. **Unclear output format**: Users don't know what format data will be returned in
4. **No usage examples**: Missing practical context for when/how to use each tool
5. **Inconsistent verbosity**: Some tools have slightly more detail than others with no clear pattern
6. **Missing behavioral hints**: No tool annotations for read-only, destructive, or idempotent operations

#### Files Requiring Updates:
- `src/tool_registry.rs`: Lines 59-156 contain tool metadata definitions
- `src/handler.rs`: Lines 154-510 contain tool implementation decorators with descriptions
- Both locations have descriptions that need to be synchronized and enhanced

## Requirements

### Functional Requirements
1. Each tool description must clearly explain:
   - What the tool does
   - What parameters it accepts (with examples)
   - What output format it returns
   - When to use it vs similar tools

2. Descriptions must be consistent between:
   - Tool registry metadata (`tool_registry.rs`)
   - Handler decorators (`handler.rs`)
   - Any generated documentation

3. Add MCP tool annotations where appropriate:
   - `readOnlyHint` for discovery tools
   - `destructiveHint` for pipeline stop operations
   - `idempotentHint` for validation tools

### Non-Functional Requirements
1. Descriptions should be concise yet comprehensive (2-3 sentences max)
2. Use consistent terminology aligned with GStreamer documentation
3. Follow MCP specification guidelines for tool metadata
4. Maintain backward compatibility with existing API

## Implementation Blueprint

### Research Phase
1. Review MCP specification for tool description best practices:
   - https://modelcontextprotocol.io/docs/concepts/tools
   - https://spec.modelcontextprotocol.io/specification/2024-11-05/server/tools/

2. Study similar MCP server implementations for description patterns:
   - Official MCP server examples
   - Community MCP servers with good documentation

3. Review GStreamer documentation for proper terminology:
   - Element vs Plugin distinctions
   - Pipeline state terminology
   - Property naming conventions

### Implementation Tasks

#### Task 1: Create Description Template
Define a standard template for tool descriptions that includes:
- One-sentence summary
- Parameter hints inline
- Output format indication
- Optional usage context

Example format: "Action summary. Accepts X (optional: Y). Returns Z format. Use for A scenarios."

#### Task 2: Enhance Tool Registry Descriptions
Update all tool descriptions in `src/tool_registry.rs` following the template:
- Lines 59-65: gst_list_elements
- Lines 68-74: gst_inspect_element
- Lines 77-85: gst_list_plugins
- Lines 88-95: gst_search_elements
- Lines 99-106: gst_launch_pipeline
- Lines 109-116: gst_set_pipeline_state
- Lines 119-126: gst_get_pipeline_status
- Lines 129-136: gst_stop_pipeline
- Lines 139-146: gst_list_pipelines
- Lines 149-156: gst_validate_pipeline

#### Task 3: Synchronize Handler Descriptions
Update matching descriptions in `src/handler.rs`:
- Line 154: gst_list_elements
- Line 206: gst_inspect_element
- Line 265: gst_list_plugins
- Line 308: gst_search_elements
- Line 342: gst_launch_pipeline
- Line 382: gst_set_pipeline_state
- Line 419: gst_get_pipeline_status
- Line 463: gst_stop_pipeline
- Line 481: gst_list_pipelines
- Line 509: gst_validate_pipeline

#### Task 4: Add Tool Annotations
Implement tool annotations structure in handler:
- Add annotations field to tool metadata
- Set readOnlyHint=true for discovery tools
- Set destructiveHint=true for stop_pipeline
- Set idempotentHint=true for validate_pipeline

#### Task 5: Update Parameter Descriptions
Enhance schemars descriptions for all parameter structs in `src/handler.rs`:
- Lines 22-26: ListElementsParams
- Lines 30-32: InspectElementParams
- Lines 36-38: ListPluginsParams
- Lines 42-44: SearchElementsParams
- Lines 48-54: LaunchPipelineParams
- Lines 58-62: SetPipelineStateParams
- Lines 66-70: GetPipelineStatusParams
- Lines 74-78: StopPipelineParams
- Lines 82-84: ListGstPipelinesParams
- Lines 88-90: ValidatePipelineParams

## Validation Gates

```bash
# Build and format check
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

# Run tests to ensure no breaking changes
cargo test --all-targets --all-features -- --nocapture

# Manual validation - test with MCP client
cargo run -- --mode all

# Verify descriptions are readable via MCP tools/list
# Check that all tools have enhanced descriptions
# Confirm parameter hints are visible in tool schemas
```

## Expected Outcomes

### Before (Current):
- "List all available GStreamer elements"
- "Launch a GStreamer pipeline from description"
- "Search GStreamer elements by keyword"

### After (Enhanced):
- "List all available GStreamer elements with optional filtering by name pattern or category. Returns element names, descriptions, plugin sources, and rank values. Use to discover available processing components."
- "Launch a GStreamer pipeline from gst-launch syntax description. Accepts pipeline description string, optional auto-play flag, and custom ID. Returns pipeline ID and status. Use to create and start media processing pipelines."
- "Search for GStreamer elements by keyword matching names and descriptions. Returns relevance-ranked results with element details. Use to find elements for specific media processing tasks."

## Success Criteria

1. All 10 implemented tools have enhanced descriptions
2. Descriptions follow consistent template format
3. Tool annotations correctly indicate behavior hints
4. Parameter descriptions include all constraints and examples
5. No breaking changes to existing API
6. AI agents can understand tool purposes without additional context

## Implementation Order

1. Create and validate description template
2. Update tool_registry.rs descriptions
3. Synchronize handler.rs decorators
4. Add parameter description enhancements
5. Implement tool annotations
6. Test with MCP client
7. Update any generated documentation

## Risk Mitigation

- **Risk**: Breaking existing integrations
  - **Mitigation**: Only modify description strings, not tool names or schemas

- **Risk**: Descriptions become too verbose
  - **Mitigation**: Enforce 2-3 sentence limit with template

- **Risk**: Inconsistency between registry and handler
  - **Mitigation**: Create shared constants or single source of truth

## References

- MCP Tools Specification: https://spec.modelcontextprotocol.io/specification/2024-11-05/server/tools/
- MCP Tool Annotations: https://modelcontextprotocol.io/docs/concepts/tools#annotations
- GStreamer Element Documentation: https://gstreamer.freedesktop.org/documentation/
- JSON Schema Description Guidelines: https://json-schema.org/understanding-json-schema/

## Confidence Score: 9/10

The requirements are clear, the scope is well-defined, and the implementation path is straightforward. The only minor uncertainty is around the exact wording of descriptions, which can be refined during implementation based on GStreamer documentation standards.
