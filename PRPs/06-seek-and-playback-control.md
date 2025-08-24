# PRP: GStreamer Seek and Playback Control Tools

## Overview
This PRP covers implementing seeking, playback rate control, and frame stepping capabilities in the GStreamer MCP server. These tools will enable precise control over media playback including seeking to specific positions, changing playback speed (fast-forward/rewind), and frame-by-frame navigation.

## Context & References

### Prerequisites
- Completed PRP-01 and PRP-02: Working MCP server with pipeline management
- GStreamer pipelines already manageable through existing tools
- Pipeline state tracking and bus message handling in place

### Key Patterns to Follow
- **Existing Pipeline Access Pattern**: Reference `src/pipeline.rs` methods like `set_pipeline_state()` which get pipeline instance and perform operations
- **Tool Definition Pattern**: Reference `src/handler.rs` for how tools are defined with JsonSchema derive and Parameters extraction
- **Error Handling**: Follow existing GStreamerMcpError patterns in `src/error.rs`

### GStreamer Seek API Documentation (Local Source Code)

**IMPORTANT**: The gstreamer-rs source code is located at `C:\Users\deste\repos\gstreamer-rs`

- **seek_simple method**: Located in `C:\Users\deste\repos\gstreamer-rs\gstreamer\src\element.rs` lines 623-639
  - Takes SeekFlags and position as FormattedValue
  - Simpler API for basic position seeking
  - Returns Result<(), BoolError>
  - Usage: `element.seek_simple(seek_flags, seek_pos)`
  
- **seek method**: Located in `C:\Users\deste\repos\gstreamer-rs\gstreamer\src\element.rs` lines 594-620
  - Takes rate (f64) for playback speed control
  - Allows segment seeks with start/stop positions
  - More complex but enables fast-forward/rewind via rate parameter
  - Usage: `element.seek(rate, flags, start_type, start, stop_type, stop)`

- **Step Event**: For frame-by-frame navigation
  - Step struct defined in `C:\Users\deste\repos\gstreamer-rs\gstreamer\src\event.rs`
  - See `C:\Users\deste\repos\gstreamer-rs\tutorials\src\bin\basic-tutorial-13.rs` line 174 for usage
  - Requires sending Step event to video-sink element

- **SeekFlags**: Located in `C:\Users\deste\repos\gstreamer-rs\gstreamer\src\auto\flags.rs`
  - Common flags: FLUSH, ACCURATE, KEY_UNIT, SEGMENT, SKIP

### Reference Examples (Local Source Code)

- **Basic Seeking Example**: `C:\Users\deste\repos\gstreamer-rs\tutorials\src\bin\basic-tutorial-4.rs`
  - Lines 85-91: Shows seek_simple usage with FLUSH and KEY_UNIT flags
  - Demonstrates seeking to specific time position
  - Full working example of seeking after 10 seconds of playback

- **Advanced Rate Control Example**: `C:\Users\deste\repos\gstreamer-rs\tutorials\src\bin\basic-tutorial-13.rs`
  - Lines 25-46: `send_seek_event` function showing rate-based seeking
  - Lines 157-170: Examples of 2x speed, 0.5x speed, and reverse playback
  - Line 174: Frame stepping using Step event
  - Complete example with keyboard controls for all features

- **Additional Examples**:
  - Thumbnail generation with seeking: `C:\Users\deste\repos\gstreamer-rs\examples\src\bin\thumbnail.rs`
  - Query position/duration: Throughout tutorials, search for `query_position` and `query_duration`

## Implementation Blueprint

### Module Structure
Extend existing modules rather than creating new ones:
- `src/pipeline.rs`: Add seek methods to PipelineManager
- `src/handler.rs`: Add new tool definitions and implementations
- `src/error.rs`: May need new error variants if seeking fails

### Core Components

#### 1. Pipeline Seek Methods (src/pipeline.rs)
Add these methods to PipelineManager impl:
- `seek_position()`: Use seek_simple for basic position seeking
- `seek_with_rate()`: Use seek for rate-based playback control  
- `step_frame()`: Send Step event to video-sink for frame stepping
- `get_seekable()`: Query if pipeline supports seeking
  - Use `gst::query::Seeking::new(gst::Format::Time)`
  - Call `pipeline.query(&mut seeking)` 
  - Extract seekable, start, end with `seeking.result()`
  - Example at `C:\Users\deste\repos\gstreamer-rs\tutorials\src\bin\basic-tutorial-4.rs:140-148`

#### 2. Tool Parameter Structures (src/handler.rs)
Define parameter structs with JsonSchema derive:
- SeekPipelineParams: pipeline_id, position_ns or position_percent
- SetPlaybackRateParams: pipeline_id, rate (f64)
- StepFrameParams: pipeline_id, frames (i32), forward (bool)
- GetSeekableParams: pipeline_id

#### 3. Tool Implementations (src/handler.rs)
Following existing pattern with #[tool] attribute:
- gst_seek_pipeline: Call pipeline_manager.seek_position()
- gst_set_playback_rate: Call pipeline_manager.seek_with_rate()
- gst_step_frame: Call pipeline_manager.step_frame()
- gst_get_seekable: Query pipeline seekability

### Tool Definitions

#### Tool 11: SeekPipeline
- **Description**: "Seek to a specific position in the pipeline"
- **Parameters**:
  - pipeline_id: Required pipeline identifier
  - position: Required position (choose representation):
    - Option A: position_ns (i64) - nanoseconds
    - Option B: position_percent (f64) - 0.0 to 1.0
  - flush: Optional boolean for FLUSH flag (default: true)
  - accurate: Optional boolean for ACCURATE flag (default: false)
- **Returns**: 
  - success: Boolean
  - new_position: Actual position after seek

#### Tool 12: SetPlaybackRate
- **Description**: "Change playback speed (fast-forward/rewind)"
- **Parameters**:
  - pipeline_id: Required pipeline identifier
  - rate: Required playback rate (f64)
    - 1.0 = normal speed
    - 2.0 = 2x fast-forward
    - 0.5 = slow motion
    - -1.0 = reverse playback
- **Returns**:
  - success: Boolean
  - actual_rate: Rate after change

#### Tool 13: StepFrame
- **Description**: "Step forward or backward by frames"
- **Parameters**:
  - pipeline_id: Required pipeline identifier
  - frames: Number of frames to step (default: 1)
  - forward: Direction (default: true)
- **Returns**:
  - success: Boolean
  - message: Status message

#### Tool 14: GetSeekable
- **Description**: "Check if pipeline supports seeking"
- **Parameters**:
  - pipeline_id: Required pipeline identifier
- **Returns**:
  - seekable: Boolean
  - current_position: Current position if available
  - duration: Total duration if available

## Implementation Tasks

1. **Add Dependencies**
   - Check if gstreamer::event::Seek is accessible (should be via prelude)
   - Check if gstreamer::event::Step is accessible

2. **Extend Pipeline Manager**
   - Add seek_position method using element.seek_simple()
   - Add seek_with_rate method using element.seek()
   - Add step_frame method using Step event
   - Add get_seekable method using query

3. **Add Tool Parameters**
   - Define all parameter structs with JsonSchema derive
   - Follow existing pattern in handler.rs (see LaunchPipelineParams)

4. **Implement Tools**
   - Add tool methods with #[tool] attribute
   - Follow error handling pattern from existing tools
   - Format responses consistently

5. **Handle Edge Cases**
   - Live streams may not be seekable
   - Some formats don't support reverse playback
   - Frame stepping requires paused state
   - Rate changes may have limits per format

## Validation Gates

```bash
# Build and lint
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

# Compile check
cargo build --release

# Manual validation - test seeking
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"gst_seek_pipeline","arguments":{"pipeline_id":"test-1","position_ns":10000000000}},"id":1}' | cargo run

# Test playback rate
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"gst_set_playback_rate","arguments":{"pipeline_id":"test-1","rate":2.0}},"id":2}' | cargo run

# Test frame stepping
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"gst_step_frame","arguments":{"pipeline_id":"test-1","frames":1}},"id":3}' | cargo run
```

## Error Handling Considerations

1. **Seeking Failures**
   - Non-seekable media (live streams)
   - Invalid position (beyond duration)
   - Pipeline not in proper state

2. **Rate Limitations**
   - Some formats don't support reverse playback
   - Hardware decoders may have rate limits
   - Audio pitch preservation at different rates

3. **Frame Stepping**
   - Only works in PAUSED state
   - Requires video-sink element
   - May not work with all codecs

## Important Implementation Notes

1. **SeekFlags Usage**
   - FLUSH: Clear buffers, immediate seek (recommended for interactive seeking)
   - KEY_UNIT: Seek to nearest keyframe (faster but less accurate)
   - ACCURATE: Precise seeking (slower but exact)
   - Default should be FLUSH | KEY_UNIT for responsiveness

2. **Rate Considerations**
   - Negative rates require formats that support reverse playback
   - Rates > 2.0 or < 0.5 may cause audio artifacts
   - Rate of 0.0 is invalid and should be rejected

3. **Position Formats**
   - Use gst::ClockTime for time-based seeking
   - Consider supporting percentage-based seeking for convenience
   - Always validate position against duration before seeking

4. **State Requirements**
   - Seeking works best in PAUSED or PLAYING states
   - Frame stepping requires PAUSED state
   - Some elements allow seeking in READY state

## Dependencies & Resources

### GStreamer APIs to Use (with Local Source Locations)

**Base path**: `C:\Users\deste\repos\gstreamer-rs`

- **gst::Element::seek_simple()** - Basic position seeking
  - Source: `gstreamer\src\element.rs:623-639`
  - Import: Already available via `gstreamer::prelude::*`
  
- **gst::Element::seek()** - Rate and segment seeking
  - Source: `gstreamer\src\element.rs:594-620`
  - Import: Already available via `gstreamer::prelude::*`
  
- **gst::event::Seek** - Seek event construction
  - Source: `gstreamer\src\event.rs`
  - Import: `use gstreamer::event::Seek;`
  
- **gst::event::Step** - Frame stepping event
  - Source: `gstreamer\src\event.rs`
  - Import: `use gstreamer::event::Step;`
  
- **gst::SeekFlags** - Flags for seek behavior
  - Source: `gstreamer\src\auto\flags.rs`
  - Import: `use gstreamer::SeekFlags;`
  
- **gst::SeekType** - Type of seek (Set, End, None)
  - Source: `gstreamer\src\auto\enums.rs`
  - Import: `use gstreamer::SeekType;`

- **gst::ClockTime** - Time representation
  - Already in use in existing code
  - Import: `use gstreamer::ClockTime;`

- **gst::query::Seeking** - Query seeking capabilities
  - Source: `gstreamer\src\query.rs`
  - Import: `use gstreamer::query::Seeking;`
  - Usage example: `C:\Users\deste\repos\gstreamer-rs\tutorials\src\bin\basic-tutorial-4.rs:140-148`

### Reference Documentation
- GStreamer seeking design: https://gstreamer.freedesktop.org/documentation/additional/design/seeking.html
- Position tracking: https://gstreamer.freedesktop.org/documentation/application-development/advanced/queryevents.html
- Tutorial on seeking: Check basic-tutorial-4.rs in gstreamer-rs/tutorials

### Test Media Files
The user has test files at `C:\Users\deste\Videos\` including:
- `download_20200705_192949.mp4` - 33 second video file
- Various processed versions of the same file
- Use these for testing seek functionality

## Success Criteria

1. Can seek to any position in a seekable media file
2. Can change playback speed (2x, 0.5x, reverse)
3. Can step frame-by-frame in paused state
4. Properly reports seeking capability for different media types
5. Handles non-seekable streams gracefully
6. Maintains pipeline state consistency after operations

## Confidence Score: 8/10

The implementation is straightforward as it builds on existing pipeline management infrastructure. The GStreamer APIs are well-documented in the local gstreamer-rs source, and we have working examples to reference. The main complexity is in handling various media formats and their seeking limitations gracefully.

The score is not 10/10 because:
- Different media formats have varying seek support that needs testing
- Frame stepping behavior can be codec-dependent
- Rate limits vary by decoder implementation