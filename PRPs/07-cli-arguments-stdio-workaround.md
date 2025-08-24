# PRP: CLI Arguments Handling with stdio Transport Workaround

## Executive Summary

This PRP addresses the blocking issue preventing command-line argument parsing in the GStreamer MCP server. The rmcp server immediately takes over stdio for MCP protocol communication, preventing traditional CLI argument parsing. This PRP provides two complementary solutions: parsing arguments before rmcp initialization and using environment variables as a fallback mechanism.

## Problem Statement

### Current State
- The GStreamer MCP server cannot accept command-line arguments (as planned in PRP-00)
- The rmcp library's `serve(stdio())` method immediately takes over stdin/stdout
- There's no opportunity to use standard CLI parsing libraries like clap after main() starts
- The server always runs with all tools enabled, which may not be appropriate for all use cases
- No way to implement the operational modes (live, dev, discovery) defined in PRP-00

### Desired State
- Users can specify operational modes via command-line arguments OR environment variables
- The server can filter tools based on the selected mode before initialization
- Maintain compatibility with MCP client expectations (stdio transport)
- Support the full CLI interface designed in PRP-00

### Business Value
- Enables different user personas (developers vs operators) to use appropriate tool sets
- Improves security by limiting available tools based on context
- Allows for better resource management and performance optimization
- Provides flexibility for deployment in different environments

## Requirements

### Functional Requirements

1. **Early Argument Parsing**: Parse command-line arguments BEFORE calling `serve(stdio())`
2. **Environment Variable Fallback**: Support configuration via environment variables when CLI args aren't accessible
3. **Mode Selection**: Implement the operational modes from PRP-00 (live, dev, discovery, all)
4. **Tool Filtering**: Filter available tools based on selected mode before server initialization
5. **Configuration Priority**: CLI args override env vars, which override config file, which overrides defaults
6. **REPL Mode**: Support interactive REPL mode for human testing/debugging without stdio takeover

### Non-Functional Requirements

1. **Performance**: Argument parsing must complete in <10ms to avoid startup delay
2. **Reliability**: Must gracefully handle invalid arguments without crashing
3. **Security**: Validate all inputs to prevent injection or unexpected behavior
4. **Compatibility**: Must work with existing MCP clients expecting stdio transport

### Context and Research

Key insight: The stdio takeover happens when `serve(stdio())` is called. Arguments must be parsed BEFORE this call. The rmcp library itself uses `tokio::io::stdin()` and `tokio::io::stdout()` directly without any argument handling.

Alternative transports (SSE, WebSocket) exist but would break compatibility with Claude Desktop and other MCP clients expecting stdio.

### Command-Line Interface Design

```bash
# Default MCP server mode (all tools)
gstreamer-mcp

# REPL mode for interactive testing (NEW)
gstreamer-mcp --repl
gstreamer-mcp -r

# Operational modes (from PRP-00)
gstreamer-mcp --mode live
gstreamer-mcp --mode dev
gstreamer-mcp --mode discovery

# Combined with REPL
gstreamer-mcp --repl --mode live

# Help and version
gstreamer-mcp --help
gstreamer-mcp --version
```

REPL mode provides an interactive shell for testing GStreamer pipelines and MCP tools without the stdio takeover issue. This is particularly useful for debugging and development.

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/clap/latest/clap/
  why: Clap derive API for argument parsing, specifically Parser trait and try_parse_from
  
- file: src/main.rs
  why: Current implementation showing where stdio takeover occurs (line 43)
  
- file: PRPs/00-command-line-modes.md
  why: Original CLI design specification with all modes and arguments
  
- file: src/config.rs
  why: Existing configuration structure to extend with CLI options

- file: ../modelcontextprotocol--rust-sdk/crates/rmcp/src/transport/io.rs
  why: rmcp's stdio implementation showing it just returns tokio::io::stdin/stdout
  
- file: ../modelcontextprotocol--rust-sdk/examples/servers/src/counter_stdio.rs
  why: Example showing typical rmcp stdio server pattern without CLI args

- url: https://rust-cli.github.io/book/tutorial/cli-args.html
  why: Rust CLI best practices for argument parsing before main logic
```

### List of tasks to be completed to fulfill the PRP in the order they should be completed

```yaml
Task 1: Add clap dependency and create CLI module
MODIFY Cargo.toml:
  - FIND dependencies section
  - ADD clap = { version = "4.5", features = ["derive", "env"] }
  - ADD rustyline = "14.0" for REPL readline support

CREATE src/cli.rs:
  - MIRROR pattern from: PRPs/00-command-line-modes.md (lines 94-130)
  - DEFINE Cli struct with Parser derive
  - DEFINE OperationalMode enum (live, dev, discovery, all)
  - ADD repl flag for interactive mode
  - IMPLEMENT parse_args() function that returns ParsedConfig

Task 2: Create tool registry system
CREATE src/tool_registry.rs:
  - DEFINE ToolMetadata struct with name, category, description
  - DEFINE ToolCategory enum (Discovery, Pipeline, Development, etc.)
  - CREATE registry of all tools with their metadata
  - IMPLEMENT filter_tools_by_mode() function

Task 3: Update configuration to include CLI options
MODIFY src/config.rs:
  - ADD operational_mode field to Configuration
  - ADD included_tools and excluded_tools fields
  - IMPLEMENT merge_cli_args() method
  - PRESERVE existing merge_env_vars() functionality

Task 4: Integrate CLI parsing and branching logic
MODIFY src/main.rs:
  - FIND line before "let handler = GStreamerHandler"
  - INSERT CLI argument parsing using clap
  - IF repl flag is set, call run_repl() instead of serve(stdio())
  - ELSE apply parsed config to handler initialization
  - PRESERVE all existing functionality after serve(stdio())

Task 5: Update handler to respect tool filtering
MODIFY src/handler.rs:
  - FIND tool registration in serve() method
  - ADD conditional registration based on tool_registry filtering
  - PRESERVE all existing tool implementations

Task 6: Add environment variable support as fallback
MODIFY src/cli.rs:
  - ADD support for GSTREAMER_MCP_MODE env var
  - ADD support for GSTREAMER_MCP_TOOLS env var
  - ADD support for GSTREAMER_MCP_EXCLUDE_TOOLS env var
  - IMPLEMENT fallback chain: CLI -> ENV -> Config file -> Defaults

Task 7: Add validation and error handling
MODIFY src/cli.rs:
  - ADD validation for conflicting options
  - ADD helpful error messages for invalid modes
  - ADD --help and --version support

Task 8: Implement REPL mode for interactive testing
CREATE src/repl.rs:
  - IMPLEMENT run_repl() function using rustyline
  - CREATE command parser for REPL commands
  - ADD commands: list (pipelines), launch, stop, status, inspect, help, exit
  - INTEGRATE with GStreamerHandler for tool execution
  - ENSURE proper error handling and user feedback

Task 9: Update logging to work with CLI args
MODIFY src/main.rs:
  - FIND tracing initialization
  - ADD support for -v/-vv verbosity flags
  - ENSURE logging still goes to stderr (not stdout)
```

### Out of Scope
- Changing from stdio to another transport mechanism
- Modifying the rmcp library itself
- Creating a wrapper executable that launches the MCP server
- Interactive mode selection after startup
- Dynamic tool loading/unloading during runtime

## Success Criteria

- [ ] CLI arguments can be parsed before stdio is taken over
- [ ] All modes from PRP-00 are functional (live, dev, discovery, all)
- [ ] Tools are correctly filtered based on selected mode
- [ ] Environment variables work as fallback when CLI args unavailable
- [ ] Server starts successfully with Claude Desktop and other MCP clients
- [ ] --help displays usage information without starting the server
- [ ] Invalid arguments show helpful error messages
- [ ] REPL mode provides interactive testing without stdio conflicts
- [ ] REPL commands work correctly with GStreamer tools

## Dependencies

### Technical Dependencies
- clap v4.5+ for argument parsing
- rustyline v14.0+ for REPL readline support
- Existing rmcp and gstreamer dependencies
- tokio runtime (already present)

### Knowledge Dependencies
- Understanding of MCP protocol and stdio transport
- Knowledge of Rust's std::env for environment variables
- Familiarity with clap's derive API

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| CLI parsing interferes with stdio | Low | High | Parse args before any stdio operations |
| Environment variables conflict with existing config | Medium | Medium | Define clear precedence order |
| Some MCP clients pass unexpected args | Medium | Low | Use strict parsing with clear errors |
| Tool filtering breaks existing functionality | Low | High | Comprehensive testing of each mode |

## Architecture Decisions

### Decision: Parse Arguments Before stdio Initialization
**Options Considered:**
1. Parse arguments before serve(stdio())
2. Use only environment variables
3. Create wrapper script that parses args then launches server
4. Fork rmcp to add argument support

**Decision:** Option 1 - Parse arguments before serve(stdio())

**Rationale:** This is the simplest solution that maintains compatibility while solving the core problem. It requires minimal changes and follows Rust CLI best practices.

### Decision: Dual Support for CLI Args and Environment Variables
**Options Considered:**
1. CLI arguments only
2. Environment variables only
3. Support both with clear precedence

**Decision:** Option 3 - Support both with clear precedence

**Rationale:** Maximum flexibility for different deployment scenarios. Some environments may not allow CLI args but can set env vars.

## Validation Strategy

- **Unit Testing**: Test argument parsing, mode selection, tool filtering logic
- **Integration Testing**: Verify server starts with different argument combinations
- **Compatibility Testing**: Test with Claude Desktop, Cursor, and other MCP clients
- **Error Testing**: Verify graceful handling of invalid arguments

## Future Considerations

- Dynamic tool loading/unloading during runtime
- Configuration hot-reloading without restart
- Web UI for configuration management
- Multiple configuration profiles
- Tool-specific configuration options

## References

- [MCP Specification](https://spec.modelcontextprotocol.io)
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Rust CLI Book](https://rust-cli.github.io/book/)
- [Original CLI Design (PRP-00)](PRPs/00-command-line-modes.md)

---

## PRP Metadata

- **Author**: Claude (via user research)
- **Created**: 2025-08-24
- **Last Modified**: 2025-08-24
- **Status**: Draft
- **Confidence Level**: 9/10 - High confidence due to clear problem understanding and straightforward solution path. The only uncertainty is around edge cases with different MCP clients.