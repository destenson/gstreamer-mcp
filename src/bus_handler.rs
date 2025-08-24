use futures::prelude::*;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::pipeline::{BusMessage, PipelineManager};

pub struct BusHandler {
    pipeline_manager: Arc<PipelineManager>,
}

impl BusHandler {
    pub fn new(pipeline_manager: Arc<PipelineManager>) -> Self {
        Self { pipeline_manager }
    }

    pub async fn watch_pipeline(
        &self,
        pipeline_id: String,
        pipeline: gst::Pipeline,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        let bus = pipeline.bus().expect("Pipeline should have a bus");
        
        // Set up bus watch
        bus.add_signal_watch();
        
        // Create a stream for bus messages
        let mut messages = bus.stream();
        
        loop {
            tokio::select! {
                Some(msg) = messages.next() => {
                    self.handle_message(&pipeline_id, &msg).await;
                }
                _ = shutdown_rx.recv() => {
                    debug!("Received shutdown signal for pipeline {}", pipeline_id);
                    break;
                }
            }
        }
        
        // Cleanup
        bus.remove_signal_watch();
    }

    async fn handle_message(&self, pipeline_id: &str, msg: &gst::Message) {
        let message = match msg.view() {
            gst::MessageView::Eos(_) => {
                info!("Pipeline {} reached end of stream", pipeline_id);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "Eos".to_string(),
                    message: "End of stream".to_string(),
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::Error(err) => {
                let error_msg = format!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                error!("Pipeline {} error: {}", pipeline_id, error_msg);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "Error".to_string(),
                    message: error_msg,
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::Warning(warn) => {
                let warning_msg = format!(
                    "Warning from {:?}: {} ({:?})",
                    warn.src().map(|s| s.path_string()),
                    warn.error(),
                    warn.debug()
                );
                warn!("Pipeline {} warning: {}", pipeline_id, warning_msg);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "Warning".to_string(),
                    message: warning_msg,
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::StateChanged(state_changed) => {
                // Only track pipeline state changes, not element state changes
                if msg.src().map(|s| s.type_().name()) == Some("GstPipeline") {
                    let message = format!(
                        "State changed from {:?} to {:?}",
                        state_changed.old(),
                        state_changed.current()
                    );
                    debug!("Pipeline {} state change: {}", pipeline_id, message);
                    BusMessage {
                        timestamp: chrono::Utc::now(),
                        message_type: "StateChanged".to_string(),
                        message,
                        source: msg.src().map(|s| s.path_string().to_string()),
                    }
                } else {
                    // Skip element state changes, only log pipeline state changes
                    return;
                }
            }
            gst::MessageView::Buffering(buffering) => {
                let percent = buffering.percent();
                debug!("Pipeline {} buffering: {}%", pipeline_id, percent);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "Buffering".to_string(),
                    message: format!("Buffering: {}%", percent),
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::Tag(tag) => {
                let tags = tag.tags();
                debug!("Pipeline {} tags: {:?}", pipeline_id, tags);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "Tag".to_string(),
                    message: format!("Tags: {:?}", tags),
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::StreamStatus(status) => {
                debug!(
                    "Pipeline {} stream status: {:?}",
                    pipeline_id,
                    status.type_()
                );
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "StreamStatus".to_string(),
                    message: format!("Stream status: {:?}", status.type_()),
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::Application(_app) => {
                debug!("Pipeline {} application message", pipeline_id);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "Application".to_string(),
                    message: "Application-specific message".to_string(),
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::Element(_element) => {
                debug!("Pipeline {} element message", pipeline_id);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "Element".to_string(),
                    message: "Element-specific message".to_string(),
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::DurationChanged(_) => {
                debug!("Pipeline {} duration changed", pipeline_id);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "DurationChanged".to_string(),
                    message: "Duration changed".to_string(),
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            gst::MessageView::Latency(_) => {
                debug!("Pipeline {} latency message", pipeline_id);
                BusMessage {
                    timestamp: chrono::Utc::now(),
                    message_type: "Latency".to_string(),
                    message: "Latency update".to_string(),
                    source: msg.src().map(|s| s.path_string().to_string()),
                }
            }
            _ => {
                // Skip other message types
                return;
            }
        };
        
        // Store the message
        self.pipeline_manager.add_bus_message(pipeline_id, message);
    }
}

pub fn create_blocking_bus_handler(
    pipeline: &gst::Pipeline,
    pipeline_id: &str,
    pipeline_manager: Arc<PipelineManager>,
    timeout: Option<gst::ClockTime>,
) -> Result<(), String> {
    let bus = pipeline
        .bus()
        .ok_or_else(|| "Pipeline has no bus".to_string())?;

    let timeout = timeout.unwrap_or(gst::ClockTime::from_seconds(5));
    
    loop {
        match bus.timed_pop(timeout) {
            Some(msg) => {
                let should_break = matches!(
                    msg.view(),
                    gst::MessageView::Eos(_) | gst::MessageView::Error(_)
                );
                
                // Create and store the message
                let bus_message = message_to_bus_message(&msg);
                pipeline_manager.add_bus_message(pipeline_id, bus_message);
                
                if should_break {
                    break;
                }
            }
            None => {
                // Timeout - no message received
                break;
            }
        }
    }
    
    Ok(())
}

fn message_to_bus_message(msg: &gst::Message) -> BusMessage {
    match msg.view() {
        gst::MessageView::Eos(_) => BusMessage {
            timestamp: chrono::Utc::now(),
            message_type: "Eos".to_string(),
            message: "End of stream".to_string(),
            source: msg.src().map(|s| s.path_string().to_string()),
        },
        gst::MessageView::Error(err) => BusMessage {
            timestamp: chrono::Utc::now(),
            message_type: "Error".to_string(),
            message: format!("Error: {} ({:?})", err.error(), err.debug()),
            source: msg.src().map(|s| s.path_string().to_string()),
        },
        gst::MessageView::Warning(warn) => BusMessage {
            timestamp: chrono::Utc::now(),
            message_type: "Warning".to_string(),
            message: format!("Warning: {} ({:?})", warn.error(), warn.debug()),
            source: msg.src().map(|s| s.path_string().to_string()),
        },
        _ => BusMessage {
            timestamp: chrono::Utc::now(),
            message_type: format!("{:?}", msg.type_()),
            message: "Message received".to_string(),
            source: msg.src().map(|s| s.path_string().to_string()),
        },
    }
}