use crate::{
    config::Configuration, 
    handler::GStreamerHandler, 
    pipeline::PipelineManager
};
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::sync::Arc;

/// REPL commands
enum ReplCommand {
    Help,
    List,
    Launch(String),
    Stop(String),
    Status(String),
    State(String, String),
    Inspect(String),
    Search(String),
    Validate(String),
    Exit,
}

impl ReplCommand {
    fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Ok(Self::Help);
        }

        match parts[0].to_lowercase().as_str() {
            "help" | "h" | "?" => Ok(Self::Help),
            "list" | "ls" | "pipelines" => Ok(Self::List),
            "launch" | "l" => {
                if parts.len() < 2 {
                    anyhow::bail!("Usage: launch <pipeline-description>");
                }
                Ok(Self::Launch(parts[1..].join(" ")))
            }
            "stop" | "s" => {
                if parts.len() != 2 {
                    anyhow::bail!("Usage: stop <pipeline-id>");
                }
                Ok(Self::Stop(parts[1].to_string()))
            }
            "status" | "stat" => {
                if parts.len() != 2 {
                    anyhow::bail!("Usage: status <pipeline-id>");
                }
                Ok(Self::Status(parts[1].to_string()))
            }
            "state" => {
                if parts.len() != 3 {
                    anyhow::bail!("Usage: state <pipeline-id> <null|ready|paused|playing>");
                }
                Ok(Self::State(parts[1].to_string(), parts[2].to_string()))
            }
            "inspect" | "i" => {
                if parts.len() != 2 {
                    anyhow::bail!("Usage: inspect <element-name>");
                }
                Ok(Self::Inspect(parts[1].to_string()))
            }
            "search" => {
                if parts.len() < 2 {
                    anyhow::bail!("Usage: search <query>");
                }
                Ok(Self::Search(parts[1..].join(" ")))
            }
            "validate" | "v" => {
                if parts.len() < 2 {
                    anyhow::bail!("Usage: validate <pipeline-description>");
                }
                Ok(Self::Validate(parts[1..].join(" ")))
            }
            "exit" | "quit" | "q" => Ok(Self::Exit),
            _ => anyhow::bail!("Unknown command: {}. Type 'help' for available commands.", parts[0]),
        }
    }
}

/// Run the interactive REPL mode
pub async fn run_repl(config: Configuration) -> Result<()> {
    println!("GStreamer MCP REPL - Interactive Testing Mode");
    println!("Type 'help' for available commands or 'exit' to quit\n");

    // Initialize GStreamer
    gstreamer::init()?;

    // Create handler (which contains the pipeline manager)
    let handler = GStreamerHandler::with_config(config).await?;
    
    // Get the pipeline manager from the handler
    let pipeline_manager = handler.pipeline_manager.clone();

    // Create readline editor
    let mut rl = DefaultEditor::new()?;
    let prompt = "gst> ";

    // Store pipeline IDs for convenience
    let mut last_pipeline_id: Option<String> = None;

    loop {
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(line);

                // Parse and execute command
                match ReplCommand::parse(line) {
                    Ok(cmd) => {
                        let result = execute_command(cmd, &handler, &pipeline_manager, &mut last_pipeline_id).await;
                        match result {
                            Ok(should_exit) => {
                                if should_exit {
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Use 'exit' to quit");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}

async fn execute_command(
    cmd: ReplCommand,
    _handler: &GStreamerHandler,
    pipeline_manager: &Arc<PipelineManager>,
    last_pipeline_id: &mut Option<String>,
) -> Result<bool> {
    match cmd {
        ReplCommand::Help => {
            print_help();
            Ok(false)
        }
        ReplCommand::List => {
            let pipelines = pipeline_manager.list_pipelines();
            if pipelines.is_empty() {
                println!("No active pipelines");
            } else {
                println!("Active pipelines:");
                for info in pipelines {
                    println!("  {} - State: {}, Description: {}", 
                             info.id, info.state, info.description);
                }
            }
            Ok(false)
        }
        ReplCommand::Launch(description) => {
            match pipeline_manager.create_pipeline(&description, None) {
                Ok(id) => {
                    // Auto-play the pipeline
                    if let Err(e) = pipeline_manager.set_pipeline_state(&id, gstreamer::State::Playing) {
                        eprintln!("Pipeline created but failed to play: {}", e);
                    } else {
                        println!("Pipeline launched successfully with ID: {}", id);
                    }
                    *last_pipeline_id = Some(id);
                }
                Err(e) => {
                    eprintln!("Failed to launch pipeline: {}", e);
                }
            }
            Ok(false)
        }
        ReplCommand::Stop(id) => {
            let pipeline_id = resolve_pipeline_id(&id, last_pipeline_id)?;
            match pipeline_manager.stop_pipeline(&pipeline_id) {
                Ok(_) => {
                    println!("Pipeline {} stopped", pipeline_id);
                    if last_pipeline_id.as_ref() == Some(&pipeline_id) {
                        *last_pipeline_id = None;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to stop pipeline: {}", e);
                }
            }
            Ok(false)
        }
        ReplCommand::Status(id) => {
            let pipeline_id = resolve_pipeline_id(&id, last_pipeline_id)?;
            match pipeline_manager.get_pipeline_status(&pipeline_id) {
                Ok(status) => {
                    println!("Pipeline {} status:", pipeline_id);
                    println!("  State: {:?}", status.state);
                    println!("  Pending: {:?}", status.pending_state);
                    if status.position > 0 {
                        println!("  Position: {}s", status.position);
                    }
                    if status.duration > 0 {
                        println!("  Duration: {}s", status.duration);
                    }
                    println!("  Errors: {}, Warnings: {}", 
                             status.error_count, status.warning_count);
                }
                Err(e) => {
                    eprintln!("Failed to get pipeline status: {}", e);
                }
            }
            Ok(false)
        }
        ReplCommand::State(id, state) => {
            let pipeline_id = resolve_pipeline_id(&id, last_pipeline_id)?;
            let gst_state = match state.to_lowercase().as_str() {
                "null" => gstreamer::State::Null,
                "ready" => gstreamer::State::Ready,
                "paused" => gstreamer::State::Paused,
                "playing" => gstreamer::State::Playing,
                _ => {
                    eprintln!("Invalid state: {}. Use null, ready, paused, or playing", state);
                    return Ok(false);
                }
            };
            
            match pipeline_manager.set_pipeline_state(&pipeline_id, gst_state) {
                Ok(new_state) => {
                    println!("Pipeline {} state changed to: {:?}", pipeline_id, new_state);
                }
                Err(e) => {
                    eprintln!("Failed to change pipeline state: {}", e);
                }
            }
            Ok(false)
        }
        ReplCommand::Inspect(element) => {
            match crate::discovery::inspect_element(&element) {
                Ok(info) => {
                    println!("Element: {}", info.name);
                    println!("Description: {}", info.description);
                    println!("Classification: {}", info.classification);
                    println!("\nProperties:");
                    for prop in &info.properties {
                        println!("  {} ({}) - {}", prop.name, prop.type_name, prop.description);
                    }
                    println!("\nPad Templates:");
                    for pad in &info.pad_templates {
                        println!("  {} ({:?}) - Caps: {}", pad.name, pad.direction, pad.caps);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to inspect element: {}", e);
                }
            }
            Ok(false)
        }
        ReplCommand::Search(query) => {
            match crate::discovery::search_elements(&query, 100) {
                Ok(results) => {
                    if results.is_empty() {
                        println!("No elements found matching '{}'", query);
                    } else {
                        println!("Elements matching '{}':", query);
                        for result in results.iter().take(10) {
                            println!("  {} - {}", 
                                     result.name, result.classification);
                        }
                        if results.len() > 10 {
                            println!("  ... and {} more", results.len() - 10);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to search elements: {}", e);
                }
            }
            Ok(false)
        }
        ReplCommand::Validate(description) => {
            match crate::pipeline::validate_pipeline_description(&description) {
                Ok(elements) => {
                    println!("Pipeline is valid!");
                    println!("Elements that would be created:");
                    for element in &elements {
                        println!("  - {}", element);
                    }
                }
                Err(e) => {
                    println!("Pipeline is invalid: {}", e);
                }
            }
            Ok(false)
        }
        ReplCommand::Exit => Ok(true),
    }
}

fn resolve_pipeline_id(id: &str, last_pipeline_id: &Option<String>) -> Result<String> {
    if id == "last" || id == "." {
        last_pipeline_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No pipeline has been launched yet"))
    } else {
        Ok(id.to_string())
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  help, h, ?                    - Show this help message");
    println!("  list, ls, pipelines           - List active pipelines");
    println!("  launch <description>          - Launch a new pipeline");
    println!("  stop <pipeline-id>            - Stop a pipeline");
    println!("  status <pipeline-id>          - Get pipeline status");
    println!("  state <id> <state>            - Set pipeline state (null/ready/paused/playing)");
    println!("  inspect <element>             - Inspect an element");
    println!("  search <query>                - Search for elements");
    println!("  validate <description>        - Validate pipeline syntax");
    println!("  exit, quit, q                 - Exit REPL");
    println!();
    println!("Tips:");
    println!("  - Use 'last' or '.' as pipeline-id to refer to the last launched pipeline");
    println!("  - Pipeline descriptions use gst-launch syntax");
    println!("  - Example: launch videotestsrc ! autovideosink");
}