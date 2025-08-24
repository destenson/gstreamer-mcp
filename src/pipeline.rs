use gstreamer as gst;
use gstreamer::prelude::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::discovery::ensure_gstreamer_initialized;
use crate::error::{GStreamerMcpError, Result as McpResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineInfo {
    pub id: String,
    pub description: String,
    pub state: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_state_change: chrono::DateTime<chrono::Utc>,
    pub error_count: u32,
    pub warning_count: u32,
}

#[derive(Debug)]
pub struct PipelineInstance {
    pub pipeline: gst::Pipeline,
    pub info: PipelineInfo,
    pub bus_messages: Vec<BusMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message_type: String,
    pub message: String,
    pub source: Option<String>,
}

impl Drop for PipelineInstance {
    fn drop(&mut self) {
        // Ensure pipeline is stopped and cleaned up
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

pub struct PipelineManager {
    pipelines: Arc<RwLock<HashMap<String, Arc<RwLock<PipelineInstance>>>>>,
    max_pipelines: usize,
}

impl PipelineManager {
    pub fn new(max_pipelines: usize) -> Self {
        Self {
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            max_pipelines,
        }
    }

    pub fn create_pipeline(
        &self,
        description: &str,
        custom_id: Option<String>,
    ) -> McpResult<String> {
        // Ensure GStreamer is initialized
        ensure_gstreamer_initialized()?;
        
        // Check pipeline limit
        {
            let pipelines = self.pipelines.read();
            if pipelines.len() >= self.max_pipelines {
                return Err(GStreamerMcpError::PipelineError(format!(
                    "Maximum pipeline limit ({}) reached",
                    self.max_pipelines
                )));
            }
        }

        // Parse the pipeline
        let element = gst::parse::launch(description)
            .map_err(|e| GStreamerMcpError::PipelineError(format!("Failed to parse pipeline: {}", e)))?;

        let pipeline = element
            .downcast::<gst::Pipeline>()
            .map_err(|_| GStreamerMcpError::PipelineError("Failed to cast to Pipeline".to_string()))?;

        // Generate or use custom ID
        let id = custom_id.unwrap_or_else(|| format!("pipeline-{}", Uuid::new_v4()));

        // Create pipeline info
        let info = PipelineInfo {
            id: id.clone(),
            description: description.to_string(),
            state: format!("{:?}", gst::State::Null),
            created_at: chrono::Utc::now(),
            last_state_change: chrono::Utc::now(),
            error_count: 0,
            warning_count: 0,
        };

        // Create pipeline instance
        let instance = PipelineInstance {
            pipeline,
            info,
            bus_messages: Vec::new(),
        };

        // Store the pipeline
        let mut pipelines = self.pipelines.write();
        pipelines.insert(id.clone(), Arc::new(RwLock::new(instance)));

        Ok(id)
    }

    pub fn get_pipeline(&self, id: &str) -> Option<Arc<RwLock<PipelineInstance>>> {
        let pipelines = self.pipelines.read();
        pipelines.get(id).cloned()
    }

    pub fn remove_pipeline(&self, id: &str) -> McpResult<()> {
        let mut pipelines = self.pipelines.write();
        if let Some(_instance) = pipelines.remove(id) {
            // Pipeline cleanup happens in Drop trait
            Ok(())
        } else {
            Err(GStreamerMcpError::PipelineError(format!(
                "Pipeline '{}' not found",
                id
            )))
        }
    }

    pub fn list_pipelines(&self) -> Vec<PipelineInfo> {
        let pipelines = self.pipelines.read();
        pipelines
            .values()
            .map(|instance| {
                let inst = instance.read();
                inst.info.clone()
            })
            .collect()
    }

    pub fn set_pipeline_state(&self, id: &str, state: gst::State) -> McpResult<gst::State> {
        let pipeline = self
            .get_pipeline(id)
            .ok_or_else(|| GStreamerMcpError::PipelineError(format!("Pipeline '{}' not found", id)))?;

        let mut instance = pipeline.write();
        
        // Set the state
        let state_change_result = instance.pipeline.set_state(state);
        
        match state_change_result {
            Ok(gst::StateChangeSuccess::Success) | Ok(gst::StateChangeSuccess::Async) => {
                // Update info
                instance.info.state = format!("{:?}", state);
                instance.info.last_state_change = chrono::Utc::now();
                
                // Get the actual current state
                let (_, current_state, _) = instance.pipeline.state(Some(gst::ClockTime::from_seconds(1)));
                Ok(current_state)
            }
            Ok(gst::StateChangeSuccess::NoPreroll) => {
                // Live sources don't preroll
                instance.info.state = format!("{:?}", state);
                instance.info.last_state_change = chrono::Utc::now();
                Ok(state)
            }
            Err(gst::StateChangeError) => {
                Err(GStreamerMcpError::PipelineError(format!(
                    "Failed to change pipeline state to {:?}",
                    state
                )))
            }
        }
    }

    pub fn get_pipeline_status(&self, id: &str) -> McpResult<PipelineStatus> {
        let pipeline = self
            .get_pipeline(id)
            .ok_or_else(|| GStreamerMcpError::PipelineError(format!("Pipeline '{}' not found", id)))?;

        let instance = pipeline.read();
        
        // Get current state
        let (_, current_state, pending_state) = instance.pipeline.state(Some(gst::ClockTime::from_seconds(0)));
        
        // Try to get position and duration
        let position = instance.pipeline.query_position::<gst::ClockTime>()
            .map(|t| t.nseconds() as i64)
            .unwrap_or(-1);
            
        let duration = instance.pipeline.query_duration::<gst::ClockTime>()
            .map(|t| t.nseconds() as i64)
            .unwrap_or(-1);

        Ok(PipelineStatus {
            id: instance.info.id.clone(),
            description: instance.info.description.clone(),
            state: format!("{:?}", current_state),
            pending_state: if pending_state == gst::State::VoidPending {
                None
            } else {
                Some(format!("{:?}", pending_state))
            },
            position,
            duration,
            error_count: instance.info.error_count,
            warning_count: instance.info.warning_count,
            created_at: instance.info.created_at,
            last_state_change: instance.info.last_state_change,
        })
    }

    pub fn add_bus_message(&self, id: &str, message: BusMessage) {
        if let Some(pipeline) = self.get_pipeline(id) {
            let mut instance = pipeline.write();
            
            // Update error/warning counts
            match message.message_type.as_str() {
                "Error" => instance.info.error_count += 1,
                "Warning" => instance.info.warning_count += 1,
                _ => {}
            }
            
            // Keep last 100 messages
            if instance.bus_messages.len() >= 100 {
                instance.bus_messages.remove(0);
            }
            instance.bus_messages.push(message);
        }
    }

    pub fn get_bus_messages(&self, id: &str, limit: usize) -> Vec<BusMessage> {
        if let Some(pipeline) = self.get_pipeline(id) {
            let instance = pipeline.read();
            let start = if instance.bus_messages.len() > limit {
                instance.bus_messages.len() - limit
            } else {
                0
            };
            instance.bus_messages[start..].to_vec()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub id: String,
    pub description: String,
    pub state: String,
    pub pending_state: Option<String>,
    pub position: i64,
    pub duration: i64,
    pub error_count: u32,
    pub warning_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_state_change: chrono::DateTime<chrono::Utc>,
}

pub fn validate_pipeline_description(description: &str) -> McpResult<Vec<String>> {
    // Ensure GStreamer is initialized
    ensure_gstreamer_initialized()?;
    
    // Try to parse the pipeline
    match gst::parse::launch(description) {
        Ok(element) => {
            // Extract element names from the pipeline description
            // This is a simple parser that extracts element names
            let mut elements = Vec::new();
            let parts: Vec<&str> = description.split('!').collect();
            
            for part in parts {
                // Extract the element name (first word before any properties)
                if let Some(element_name) = part.trim().split_whitespace().next() {
                    // Remove any property assignments
                    let clean_name = element_name.split('=').next().unwrap_or(element_name);
                    if !clean_name.is_empty() {
                        elements.push(clean_name.to_string());
                    }
                }
            }
            
            // Cleanup
            if let Ok(pipeline) = element.downcast::<gst::Pipeline>() {
                let _ = pipeline.set_state(gst::State::Null);
            }
            
            Ok(elements)
        }
        Err(e) => Err(GStreamerMcpError::PipelineError(format!(
            "Invalid pipeline description: {}",
            e
        ))),
    }
}