# PRP: GStreamer Plugin Development Assistant for gst-plugins-rs

## Overview
This PRP covers implementing a specialized assistant tool that helps users create custom GStreamer elements using Rust, providing templates, boilerplate code, explanations, and integration with the gst-plugins-rs ecosystem.

## Context & References

### Prerequisites
- Completed PRPs 01-04: MCP server with element discovery and programming assistance
- Understanding of GStreamer element concepts
- Familiarity with Rust subclassing patterns

### Key Resources
- **gst-plugins-rs examples**: ../gst-plugins-rs/tutorial/ contains complete plugin tutorial
- **Existing plugins**: ../gst-plugins-rs/audio/, video/, generic/, etc.
- **Tutorial documentation**: ../gst-plugins-rs/tutorial/tutorial-1.md through tutorial-2.md
- **Plugin patterns**: Various element types (filters, sources, sinks, transforms)

### Element Development Concepts
- GObject subclassing in Rust
- Element lifecycle (init, start, stop, cleanup)
- Pad templates and caps negotiation
- Property implementation with GObject
- Signal handling and emission
- Buffer processing and transformation
- State management and threading

## Implementation Blueprint

### Additional Module Structure
```
src/
├── plugin_assistant/
│   ├── mod.rs                  # Plugin assistant module entry
│   ├── element_generator.rs    # Element boilerplate generation
│   ├── templates.rs            # Element type templates
│   ├── property_builder.rs     # Property definition helpers
│   ├── pad_builder.rs          # Pad template generation
│   └── tutorial_guide.rs       # Interactive tutorial system
```

### Core Components

#### 1. Element Generator (src/plugin_assistant/element_generator.rs)
- Generate complete element skeleton
- Include proper module structure
- Add registration boilerplate
- Create test templates

#### 2. Template Library (src/plugin_assistant/templates.rs)
- BaseTransform elements (filters)
- BaseSrc elements (sources)
- BaseSink elements (sinks)
- Aggregator elements (mixers)
- Bin elements (compound elements)

#### 3. Property Builder (src/plugin_assistant/property_builder.rs)
- Generate GObject property definitions
- Create getter/setter implementations
- Add property metadata
- Handle property change notifications

#### 4. Pad Builder (src/plugin_assistant/pad_builder.rs)
- Generate pad templates
- Create caps definitions
- Handle dynamic pads
- Implement pad event handling

### Tool Definitions

#### Tool 22: GenerateGstElement
- **Description**: "Generate a complete GStreamer element in Rust"
- **Parameters**:
  - element_name: Required element name (e.g., "myfilter")
  - element_type: Required type ("filter", "source", "sink", "transform", "aggregator", "bin")
  - base_class: Optional specific base class override
  - properties: Optional array of property definitions
  - input_caps: Optional input capabilities (for filters/sinks)
  - output_caps: Optional output capabilities (for filters/sources)
  - description: Element description for metadata
- **Returns**:
  - element_code: Complete element implementation
  - mod_code: Module file content
  - lib_code: Library registration code
  - cargo_toml: Dependencies to add
  - build_rs: Build script if needed
  - test_code: Basic test template

#### Tool 23: GenerateElementProperty
- **Description**: "Generate property implementation for an element"
- **Parameters**:
  - property_name: Property name
  - property_type: Type ("bool", "int", "uint", "float", "string", "enum", "flags")
  - default_value: Default value
  - description: Property description
  - mutable_in_state: Optional state where property can be changed
  - range: Optional range for numeric properties
- **Returns**:
  - property_spec: ParamSpec definition
  - storage_field: Struct field definition
  - getter_impl: Property getter implementation
  - setter_impl: Property setter implementation

#### Tool 24: GeneratePadTemplate
- **Description**: "Generate pad template and caps for an element"
- **Parameters**:
  - pad_name: Pad name
  - pad_direction: "src" or "sink"
  - pad_presence: "always", "sometimes", or "request"
  - media_type: Media type (e.g., "video/x-raw", "audio/x-raw")
  - format_constraints: Optional format constraints
  - additional_caps: Optional additional capabilities
- **Returns**:
  - pad_template_code: Pad template definition
  - caps_code: Caps definition
  - pad_handling_code: Event/query handling stubs

#### Tool 25: GenerateTransformFunction
- **Description**: "Generate transform function for filter elements"
- **Parameters**:
  - transform_type: "in_place", "copy", "metadata_only"
  - input_format: Input data format
  - output_format: Output data format
  - processing_logic: Description of transformation
- **Returns**:
  - transform_impl: Transform function implementation
  - helper_functions: Any helper functions needed
  - buffer_handling: Buffer management code

#### Tool 26: GeneratePluginBoilerplate
- **Description**: "Generate complete plugin structure with multiple elements"
- **Parameters**:
  - plugin_name: Plugin name
  - elements: Array of element names to include
  - description: Plugin description
  - license: License identifier
  - package_name: Package name
- **Returns**:
  - lib_rs: Main library file
  - cargo_toml: Complete Cargo.toml
  - build_rs: Build script
  - directory_structure: Recommended file organization

#### Tool 27: ExplainElementCode
- **Description**: "Explain gst-plugins-rs element implementation"
- **Parameters**:
  - code: Element implementation code
  - focus_area: Optional specific area ("properties", "pads", "transform", "state")
- **Returns**:
  - explanation: Detailed explanation
  - gstreamer_concepts: Concepts demonstrated
  - improvement_suggestions: Possible improvements
  - common_patterns: Related patterns in gst-plugins-rs

## Implementation Tasks

1. **Template Development**
   - Create templates for each element type
   - Extract patterns from existing plugins
   - Build modular code generation
   - Include comprehensive comments

2. **Property System**
   - Generate GObject property specs
   - Create thread-safe property storage
   - Implement property change handling
   - Add property validation

3. **Pad Management**
   - Generate static pad templates
   - Handle dynamic pad creation
   - Implement caps negotiation
   - Add event/query handling

4. **Transform Functions**
   - Create transform templates for different scenarios
   - Generate buffer processing code
   - Handle different data formats
   - Add performance optimizations

5. **Testing Templates**
   - Generate unit tests for elements
   - Create integration test templates
   - Add pipeline test examples
   - Include property testing

6. **Documentation Generation**
   - Generate inline documentation
   - Create README templates
   - Add usage examples
   - Include debugging tips

## Code Generation Templates

### Basic Element Template
```rust
// Template for a basic video filter element
use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gst_video::subclass::prelude::*;

use std::sync::Mutex;
use std::sync::LazyLock;

static CAT: LazyLock<gst::DebugCategory> = LazyLock::new(|| {
    gst::DebugCategory::new(
        "${ELEMENT_NAME}",
        gst::DebugColorFlags::empty(),
        Some("${DESCRIPTION}"),
    )
});

#[derive(Default)]
struct Settings {
    // Property storage
    ${PROPERTIES}
}

#[derive(Default)]
pub struct ${ElementName} {
    settings: Mutex<Settings>,
}

#[glib::object_subclass]
impl ObjectSubclass for ${ElementName} {
    const NAME: &'static str = "${GstElementName}";
    type Type = super::${ElementName};
    type ParentType = ${BaseClass};
}

// Property implementation
impl ObjectImpl for ${ElementName} {
    fn properties() -> &'static [glib::ParamSpec] {
        ${PROPERTY_SPECS}
    }
    
    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        ${PROPERTY_SETTERS}
    }
    
    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        ${PROPERTY_GETTERS}
    }
}

// Element implementation
impl ElementImpl for ${ElementName} {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: LazyLock<gst::subclass::ElementMetadata> = LazyLock::new(|| {
            gst::subclass::ElementMetadata::new(
                "${LONG_NAME}",
                "${CLASSIFICATION}",
                "${DESCRIPTION}",
                "${AUTHOR}",
            )
        });
        Some(&*ELEMENT_METADATA)
    }
    
    fn pad_templates() -> &'static [gst::PadTemplate] {
        ${PAD_TEMPLATES}
    }
}
```

## Validation Gates

```bash
# Build and test generated plugin
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
cargo test

# Test element generation
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"GenerateGstElement","arguments":{"element_name":"myfilter","element_type":"filter"}},"id":1}' | cargo run

# Verify generated plugin loads
GST_PLUGIN_PATH=target/release gst-inspect-1.0 ${PLUGIN_NAME}
```

## Element Type Patterns

### Common Element Types
1. **Video Filters**: RGB2Gray, effects, color correction
2. **Audio Filters**: Volume, EQ, loudness normalization
3. **Sources**: Test sources, file readers, network sources
4. **Sinks**: File writers, display sinks, network sinks
5. **Demuxers**: Container format parsers
6. **Muxers**: Container format writers
7. **Encoders/Decoders**: Codec implementations

### Key Implementation Areas
1. **Buffer Processing**: Efficient data transformation
2. **Caps Negotiation**: Format agreement between elements
3. **State Management**: Proper state transitions
4. **Thread Safety**: Mutex/Arc patterns for properties
5. **Error Handling**: Proper error propagation
6. **Performance**: Zero-copy where possible

## Dependencies & Resources

### Required Knowledge Base
- GObject type system basics
- GStreamer element lifecycle
- Rust ownership and borrowing
- Subclassing patterns in Rust

### Reference Examples
- Tutorial plugin: ../gst-plugins-rs/tutorial/
- Audio effects: ../gst-plugins-rs/audio/audiofx/
- Video effects: ../gst-plugins-rs/video/videofx/
- Generic elements: ../gst-plugins-rs/generic/

### Documentation
- gst-plugins-rs tutorial: Tutorial markdown files
- GStreamer plugin writer's guide
- GObject subclassing in Rust

## Success Criteria

1. Generate compilable element code for all basic types
2. Include working property system
3. Proper pad template and caps handling
4. Follow gst-plugins-rs conventions
5. Include comprehensive documentation
6. Generate working tests
7. Support common use cases

## Notes for Implementation

- Start with simple filter elements
- Use existing plugins as reference
- Ensure generated code follows Rust idioms
- Include safety documentation
- Provide migration path from C plugins
- Consider performance implications
- Add debugging helpers

## Confidence Score: 7/10

The implementation requires deep understanding of both GStreamer's element model and Rust's type system. The gst-plugins-rs repository provides excellent examples to base templates on, but the complexity of different element types and the variety of use cases makes this challenging. The tutorial in gst-plugins-rs provides a solid foundation for the code generation templates.