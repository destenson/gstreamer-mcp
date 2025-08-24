# TODO List - GStreamer MCP Server

## Project Status
Core MCP server with element discovery and pipeline management tools complete. All 10 tools from PRP-01 and PRP-02 are functional with enhanced descriptions (PRP-08 completed). CLI parsing implemented (PRP-07) but tool filtering not working. Ready to implement element suggestions (PRP-03) and seek/playback control (PRP-06).

## High Priority 🔴

### 1. Command-Line Interface and Modes (PRP-00/07)
- [x] **Implement CLI argument parsing** (COMPLETED)
  - ✅ Added clap dependency for command-line parsing
  - ✅ Created cli.rs module with mode selection
  - ✅ Support --mode flag (live, dev, discovery, all)
  - ✅ Added --tools and --exclude-tools options
  
- [x] **Create Tool Registry System** (COMPLETED)
  - ✅ Implemented tool metadata and categorization
  - ✅ Added mode-based tool filtering logic
  - ⚠️ **BUG**: Tool filtering not actually working - enabled_tools checked but all tools exposed
  
- [ ] **Fix Tool Filtering Implementation**
  - [ ] Actually filter tools in handler based on enabled_tools
  - [ ] Test mode-based filtering works correctly

### 2. Core MCP Server Implementation
- [x] **Initialize Rust project structure** (PRP-01) (COMPLETED)
  - Set up Cargo.toml with rmcp and gstreamer dependencies
  - Create basic module structure (main.rs, lib.rs, handler.rs, etc.)
  - Configure build scripts if needed
  
- [x] **Implement basic MCP server** (PRP-01) (COMPLETED)
  - Set up stdio transport following cargo-mcp pattern
  - Implement ServerHandler trait
  - Add server metadata and initialization
  - Integrate with tool registry for mode-based filtering

### 3. GStreamer Integration
- [x] **Initialize GStreamer context** (PRP-01) (COMPLETED)
  - Set up GStreamer initialization in discovery module
  - Handle GStreamer cleanup on shutdown
  - Add error handling for missing GStreamer installation

## Medium Priority 🟡

### 3. Element Discovery Tools (PRP-01)
- [x] Implement `ListGstElements` tool (COMPLETED)
- [x] Implement `InspectGstElement` tool with property extraction (COMPLETED)
- [x] Implement `ListGstPlugins` tool (COMPLETED)
- [x] Implement `SearchGstElements` with keyword matching (COMPLETED)
- [x] **Enhanced tool descriptions** (PRP-08 COMPLETED - All descriptions improved)
- [ ] **Add element information caching for performance**
  - Config support exists but not implemented
  - See DiscoveryCache in discovery.rs
- [ ] **Implement signal discovery** (src/discovery.rs:188)
  - Currently returns empty Vec with TODO comment

### 4. Pipeline Management (PRP-02)
- [x] Implement `LaunchPipeline` tool with gst::parse_launch (COMPLETED)
- [x] Add pipeline state management with unique IDs (COMPLETED)
- [x] Implement `SetPipelineState` for pipeline control (COMPLETED)
- [x] Implement `GetPipelineStatus` with bus message handling (COMPLETED)
- [x] Add `StopPipeline` with proper cleanup (COMPLETED)
- [x] Implement `ListGstPipelines` for active pipeline tracking (COMPLETED)
- [x] Add `ValidatePipeline` for syntax validation (COMPLETED)

### 5. Seek and Playback Control (PRP-06) - NEW
- [ ] Implement `SeekToPipeline` tool for position seeking
- [ ] Add `SetPlaybackRate` for speed control (fast-forward/rewind)
- [ ] Implement `StepFramePipeline` for frame-by-frame navigation
- [ ] Add `GetPipelinePosition` to query current playback position
- [ ] Implement `GetPipelineDuration` to get media duration

### 6. Element Suggestions (PRP-03)
- [ ] Build element index with keyword extraction
- [ ] Implement `SuggestGstElements` with intent mapping
- [ ] Add `FindSimilarGstElements` with similarity scoring
- [ ] Implement `SuggestPipelineElements` for pipeline building
- [ ] Add `AutocompleteElement` for partial name completion
- [ ] Implement `ExplainElementPurpose` with descriptions

## Low Priority 🟢

### 7. Programming Assistants (PRP-04 & PRP-05)
- [ ] **GStreamer-rs Code Assistant**
  - [ ] Implement `GeneratePipelineCode` for Rust code generation
  - [ ] Add `ConvertLaunchToCode` for pipeline conversion
  - [ ] Create `GetPipelinePattern` with common patterns
  - [ ] Implement `ExplainGstreamerCode` for code analysis
  - [ ] Add `GenerateElementCode` for element creation
  - [ ] Build `GetGstreamerExample` database

- [ ] **Plugin Development Assistant**
  - [ ] Implement `GenerateGstElement` with templates
  - [ ] Add `GenerateElementProperty` for GObject properties
  - [ ] Create `GeneratePadTemplate` for pad definitions
  - [ ] Implement `GenerateTransformFunction` for filters
  - [ ] Add `GeneratePluginBoilerplate` for complete plugins
  - [ ] Implement `ExplainElementCode` for learning

## Technical Debt & Improvements 📋

### Testing
- [x] Basic unit tests (5 tests) for CLI and tool registry
- [ ] Add comprehensive unit tests for element discovery
- [ ] Create integration tests for pipeline management
- [ ] Add tests for bus message handling
- [ ] Test tool filtering with different modes
- [ ] Add tests for suggestion algorithms (when implemented)
- [ ] Test code generation output compilation (when implemented)

### Documentation
- [x] Create comprehensive README.md (COMPLETED)
- [x] Add API documentation for all tools (COMPLETED)
- [x] Enhanced tool descriptions (PRP-08 COMPLETED)
- [ ] Write user guide for MCP client integration
- [ ] Document GStreamer setup requirements for different platforms
- [ ] Add troubleshooting guide for common issues

### Performance
- [ ] Optimize element registry scanning
- [ ] Consider caching frequently accessed element data
- [ ] Profile and optimize similarity search algorithms
- [ ] Add configurable resource limits for pipelines

### Security
- [ ] Implement pipeline description sanitization
- [ ] Add resource usage limits
- [ ] Consider file system access restrictions
- [ ] Add element whitelist/blacklist support

## Code Quality Issues 🐛

### Clippy Warnings (46 issues)
- [ ] Fix uninlined format args throughout codebase
- [ ] Replace while-let-loop patterns
- [ ] Add Default implementations where needed
- [ ] Remove unnecessary let bindings
- [ ] Fix useless format! calls

### Minor Issues
- [ ] Unused parameters in some functions (prefixed with _)
- [ ] REPL mode skeleton exists but not fully integrated

## Future Enhancements 💡

### Advanced Features
- [ ] Pipeline templates for common use cases
- [ ] Visual pipeline builder integration
- [ ] Performance monitoring tools
- [ ] Pipeline debugging assistance
- [ ] Machine learning for better element suggestions
- [ ] Integration with GStreamer debugging tools

### MCP Enhancements
- [ ] Support for WebSocket transport
- [ ] Add authentication mechanisms
- [ ] Implement rate limiting
- [ ] Add metrics and monitoring

## Dependencies to Track
- rmcp (0.6.0+) - Monitor for updates
- gstreamer-rs - Use local version from ../gstreamer-rs
- gst-plugins-rs - Reference for plugin patterns

## Notes
- PRPs are complete and located in `/PRPs/` directory (including PRP-00 for CLI modes)
- Following patterns from cargo-mcp implementation
- GStreamer must be installed (minimum version 1.14)
- All tools should follow consistent naming convention
- Focus on read-only operations before implementing pipeline control
- Different operational modes (live, dev, discovery) provide targeted tool sets for specific use cases

---
*Last Updated: 2025-01-23 - Updated after TODO scan and PRP-08 completion*
*Previous Update: 2025-01-23 - Completed PRP-08 Tool Descriptions Enhancement*
*Priority Levels: 🔴 High (Critical for MVP) | 🟡 Medium (Core features) | 🟢 Low (Nice to have)*

## Files with TODOs:
- `src/discovery.rs:188` - Signal discovery not implemented
- `src/tool_registry.rs:159` - Note about future tools to be added
