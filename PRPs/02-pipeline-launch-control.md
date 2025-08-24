# PRP: GStreamer Pipeline Launch and Control via MCP

## Overview
This PRP covers implementing pipeline launching and control functionality in the GStreamer MCP server, providing programmatic access to gst-launch-like capabilities with proper pipeline management.

## Context & References

### Prerequisites
- Completed PRP-01: Basic MCP Server with element discovery
- Working GStreamer MCP server structure in place
- GStreamer context initialization already implemented

### Key Patterns to Follow
- **Executor Pattern**: Reference ../cargo-mcp/src/executor.rs for command execution patterns
- **State Management**: Use Arc<RwLock<>> for shared pipeline state similar to cargo-mcp's command storage
- **Async Execution**: Follow tokio patterns from cargo-mcp for long-running operations

### GStreamer Pipeline Concepts
- Pipeline syntax: elements separated by ! with properties as key=value
- Pipeline states: NULL, READY, PAUSED, PLAYING
- Bus messages for error handling and state changes
- Parse launch API for creating pipelines from strings
- Pipeline lifecycle management and cleanup

### Documentation References
- gst-launch syntax: https://gstreamer.freedesktop.org/documentation/tools/gst-launch.html
- GStreamer parse API: https://gstreamer.pages.freedesktop.org/gstreamer-rs/stable/latest/docs/gstreamer/parse/index.html
- Pipeline management: https://gstreamer.freedesktop.org/documentation/tutorials/basic/

## Implementation Blueprint

### Additional Module Structure
```
src/
├── pipeline.rs         # Pipeline management and state tracking
├── executor.rs         # Pipeline execution and control
├── parser.rs          # Pipeline description validation
└── bus_handler.rs     # Message bus handling for events/errors
```

### Core Components

#### 1. Pipeline Manager (src/pipeline.rs)
- Store active pipelines with unique IDs
- Track pipeline states and metadata
- Implement cleanup on drop
- Handle multiple concurrent pipelines

#### 2. Pipeline Executor (src/executor.rs)
- Parse pipeline descriptions using gst::parse_launch
- Set up bus message handlers
- Control pipeline state transitions
- Capture pipeline output/errors

#### 3. Bus Handler (src/bus_handler.rs)
- Monitor pipeline bus for messages
- Handle EOS, ERROR, WARNING, STATE_CHANGED
- Format messages for MCP responses
- Implement timeout handling

### Tool Definitions

#### Tool 5: LaunchPipeline
- **Description**: "Launch a GStreamer pipeline from description"
- **Parameters**:
  - pipeline_description: Required string with gst-launch syntax
  - auto_play: Optional boolean to start pipeline immediately (default: true)
  - timeout_seconds: Optional timeout for pipeline execution
  - pipeline_id: Optional custom ID (auto-generated if not provided)
- **Returns**: 
  - pipeline_id: Unique identifier for the pipeline
  - status: Initial pipeline status
  - errors: Any parse errors encountered

#### Tool 6: SetPipelineState
- **Description**: "Change the state of a running pipeline"
- **Parameters**:
  - pipeline_id: Required pipeline identifier
  - state: Required state (null, ready, paused, playing)
- **Returns**:
  - success: Boolean indicating state change success
  - current_state: Actual pipeline state after change
  - messages: Any bus messages during transition

#### Tool 7: GetPipelineStatus
- **Description**: "Get current status of a pipeline"
- **Parameters**:
  - pipeline_id: Required pipeline identifier
  - include_messages: Optional boolean to include recent bus messages
- **Returns**:
  - state: Current pipeline state
  - position: Current position (if applicable)
  - duration: Total duration (if applicable)
  - recent_messages: Recent bus messages (if requested)

#### Tool 8: StopPipeline
- **Description**: "Stop and cleanup a pipeline"
- **Parameters**:
  - pipeline_id: Required pipeline identifier
  - force: Optional boolean to force termination
- **Returns**:
  - success: Boolean indicating successful cleanup
  - final_messages: Any final bus messages

#### Tool 9: ListGstPipelines
- **Description**: "List all active pipelines"
- **Parameters**:
  - include_details: Optional boolean for detailed info
- **Returns**:
  - Array of pipeline summaries with IDs, descriptions, and states

#### Tool 10: ValidatePipeline
- **Description**: "Validate a pipeline description without launching"
- **Parameters**:
  - pipeline_description: Pipeline string to validate
- **Returns**:
  - valid: Boolean indicating if pipeline is valid
  - errors: Parse errors if invalid
  - elements: List of elements that would be created

## Implementation Tasks

1. **Pipeline State Management**
   - Create PipelineManager struct with Arc<RwLock<HashMap>>
   - Implement pipeline ID generation (UUID or incremental)
   - Add pipeline metadata structure
   - Implement cleanup on drop trait

2. **Pipeline Parsing & Creation**
   - Implement safe wrapper around gst::parse_launch
   - Add validation for pipeline descriptions
   - Extract element list from pipeline description
   - Handle parse errors gracefully

3. **Pipeline Execution**
   - Implement pipeline launch with state management
   - Set up bus watch for message handling
   - Add timeout support using tokio::time
   - Implement graceful shutdown

4. **Bus Message Handling**
   - Create message handler for different message types
   - Format messages for MCP responses
   - Implement message buffering for status queries
   - Add error recovery strategies

5. **Tool Implementations**
   - Implement gst_launch_pipeline with validation
   - Implement state control tools
   - Add pipeline query tools
   - Implement cleanup and listing tools

6. **Safety & Resource Management**
   - Implement pipeline limits (max concurrent pipelines)
   - Add memory usage monitoring
   - Implement automatic cleanup for abandoned pipelines
   - Add pipeline description sanitization

## Validation Gates

```bash
# Build and lint
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

# Unit tests for pipeline parsing
cargo test pipeline_parser

# Integration tests with actual pipelines
cargo test --test pipeline_integration

# Manual validation - test pipeline launch
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"gst_launch_pipeline","arguments":{"pipeline_description":"videotestsrc ! autovideosink"}},"id":1}' | cargo run

# Test pipeline control
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"gst_pipeline_set_state","arguments":{"pipeline_id":"test-1","state":"paused"}},"id":2}' | cargo run
```

## Error Handling Considerations

1. **Parse Errors**: Invalid pipeline syntax should return clear error messages
2. **Missing Elements**: Handle cases where required GStreamer elements aren't installed
3. **State Transition Failures**: Properly report why state changes fail
4. **Resource Exhaustion**: Handle running out of memory or hitting pipeline limits
5. **Timeout Handling**: Gracefully handle pipeline timeouts
6. **Concurrent Access**: Prevent race conditions in pipeline state management

## Security Considerations

1. **Pipeline Validation**: Validate pipeline descriptions to prevent injection
2. **Resource Limits**: Enforce limits on pipeline count and resource usage
3. **File Access**: Consider restricting filesrc/filesink to specific directories
4. **Network Access**: Consider restrictions on network elements
5. **Plugin Whitelist**: Optionally restrict which elements can be used

## Dependencies & Resources

### Additional Crates Needed
- uuid (0.8+) - For pipeline ID generation
- parking_lot (0.12+) - Better RwLock implementation

### GStreamer Specific APIs
- gst::parse_launch - Pipeline parsing
- gst::Pipeline - Pipeline control
- gst::Bus - Message bus handling
- gst::State - State management

### Reference Documentation
- Pipeline states: https://gstreamer.freedesktop.org/documentation/additional/design/states.html
- Bus concepts: https://gstreamer.freedesktop.org/documentation/application-development/basics/bus.html
- Parse launch: https://gstreamer.pages.freedesktop.org/gstreamer-rs/stable/latest/docs/gstreamer/functions/fn.parse_launch.html

## Success Criteria

1. Can launch pipelines from description strings
2. Proper state management for multiple concurrent pipelines
3. Bus messages are captured and reported correctly
4. Pipelines can be controlled (play/pause/stop)
5. Resource cleanup happens automatically
6. Invalid pipelines are rejected with clear errors
7. Can query status of running pipelines

## Notes for Implementation

- Start with simple test pipelines (videotestsrc ! fakesink)
- Implement proper async handling for long-running pipelines
- Consider implementing a pipeline template system for common use cases
- Bus message handling should be non-blocking
- Pipeline IDs should be memorable but unique
- Consider adding pipeline naming for easier management
- Resource limits should be configurable

## Confidence Score: 7/10

Pipeline management adds complexity with state tracking and async message handling. The GStreamer parse API is well-documented, but proper resource management and error handling for concurrent pipelines requires careful implementation. Reference the gstreamer-rs examples for pipeline management patterns.
