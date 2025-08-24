use crate::error::{GStreamerMcpError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

static GST_INITIALIZED: OnceCell<()> = OnceCell::new();

pub fn ensure_gstreamer_initialized() -> Result<()> {
    GST_INITIALIZED.get_or_try_init(|| {
        gst::init().map_err(|e| GStreamerMcpError::GStreamerInit(e.to_string()))
    })?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub name: String,
    pub description: String,
    pub plugin_name: String,
    pub rank: String,
    pub classification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDetailedInfo {
    pub name: String,
    pub description: String,
    pub plugin_name: String,
    pub rank: String,
    pub classification: String,
    pub properties: Vec<PropertyInfo>,
    pub pad_templates: Vec<PadTemplateInfo>,
    pub signals: Vec<SignalInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    pub name: String,
    pub type_name: String,
    pub description: String,
    pub flags: Vec<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadTemplateInfo {
    pub name: String,
    pub direction: String,
    pub presence: String,
    pub caps: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalInfo {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub filename: Option<String>,
    pub version: String,
    pub license: String,
    pub source: String,
    pub elements: Vec<String>,
}

pub struct DiscoveryCache {
    elements: Arc<RwLock<Option<Vec<ElementInfo>>>>,
    plugins: Arc<RwLock<Option<Vec<PluginInfo>>>>,
}

impl DiscoveryCache {
    pub fn new() -> Self {
        Self {
            elements: Arc::new(RwLock::new(None)),
            plugins: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_elements(&self) -> Result<Vec<ElementInfo>> {
        let mut cache = self.elements.write().await;
        if cache.is_none() {
            *cache = Some(discover_all_elements()?);
        }
        Ok(cache.as_ref().unwrap().clone())
    }

    pub async fn get_plugins(&self) -> Result<Vec<PluginInfo>> {
        let mut cache = self.plugins.write().await;
        if cache.is_none() {
            *cache = Some(discover_all_plugins()?);
        }
        Ok(cache.as_ref().unwrap().clone())
    }

    pub async fn clear(&self) {
        *self.elements.write().await = None;
        *self.plugins.write().await = None;
    }
}

pub fn discover_all_elements() -> Result<Vec<ElementInfo>> {
    ensure_gstreamer_initialized()?;

    let registry = gst::Registry::get();
    let mut elements = Vec::new();

    for plugin in registry.plugins() {
        let plugin_name = plugin.plugin_name().to_string();

        for feature in registry.features_by_plugin(&plugin_name) {
            if let Ok(factory) = feature.downcast::<gst::ElementFactory>() {
                elements.push(ElementInfo {
                    name: factory.name().to_string(),
                    description: factory.description().to_string(),
                    plugin_name: plugin_name.clone(),
                    rank: format!("{:?}", factory.rank()),
                    classification: factory.klass().to_string(),
                });
            }
        }
    }

    elements.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(elements)
}

pub fn discover_all_plugins() -> Result<Vec<PluginInfo>> {
    ensure_gstreamer_initialized()?;

    let registry = gst::Registry::get();
    let mut plugins = Vec::new();

    for plugin in registry.plugins() {
        let plugin_name = plugin.plugin_name().to_string();

        let mut element_names = Vec::new();
        for feature in registry.features_by_plugin(&plugin_name) {
            if let Ok(factory) = feature.downcast::<gst::ElementFactory>() {
                element_names.push(factory.name().to_string());
            }
        }

        plugins.push(PluginInfo {
            name: plugin_name,
            description: plugin.description().to_string(),
            filename: plugin.filename().map(|p| p.to_string_lossy().to_string()),
            version: plugin.version().to_string(),
            license: plugin.license().to_string(),
            source: plugin.source().to_string(),
            elements: element_names,
        });
    }

    plugins.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(plugins)
}

pub fn inspect_element(element_name: &str) -> Result<ElementDetailedInfo> {
    ensure_gstreamer_initialized()?;

    let registry = gst::Registry::get();
    let factory = registry
        .find_feature(element_name, gst::ElementFactory::static_type())
        .and_then(|f| f.downcast::<gst::ElementFactory>().ok())
        .ok_or_else(|| GStreamerMcpError::ElementNotFound(element_name.to_string()))?;

    let element = factory
        .create()
        .name(element_name)
        .build()
        .map_err(|e| GStreamerMcpError::ElementNotFound(format!("{}: {}", element_name, e)))?;

    // Get properties
    let properties = get_element_properties(&element)?;

    // Get pad templates
    let pad_templates = get_pad_templates(&factory);

    // Get signals (basic implementation - GStreamer signals are complex)
    let signals = Vec::new(); // TODO: Implement signal discovery if needed

    // Try to find the plugin that provides this element
    let plugin_name = registry
        .plugins()
        .into_iter()
        .find(|plugin| {
            registry
                .features_by_plugin(&plugin.plugin_name())
                .into_iter()
                .any(|feature| {
                    if let Ok(f) = feature.downcast::<gst::ElementFactory>() {
                        f.name() == element_name
                    } else {
                        false
                    }
                })
        })
        .map(|p| p.plugin_name().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(ElementDetailedInfo {
        name: element_name.to_string(),
        description: factory.longname().to_string(),
        plugin_name,
        rank: format!("{:?}", factory.rank()),
        classification: factory.klass().to_string(),
        properties,
        pad_templates,
        signals,
    })
}

fn get_element_properties(element: &gst::Element) -> Result<Vec<PropertyInfo>> {
    let properties = element.list_properties();
    let mut prop_infos = Vec::new();

    for prop in properties {
        let name = prop.name().to_string();
        let type_name = prop.value_type().name().to_string();
        let description = prop.blurb().unwrap_or("").to_string();

        let mut flags = Vec::new();
        let prop_flags = prop.flags();

        if prop_flags.contains(gstreamer::glib::ParamFlags::READABLE) {
            flags.push("readable".to_string());
        }
        if prop_flags.contains(gstreamer::glib::ParamFlags::WRITABLE) {
            flags.push("writable".to_string());
        }
        if prop_flags.contains(gstreamer::glib::ParamFlags::CONSTRUCT) {
            flags.push("construct".to_string());
        }
        if prop_flags.contains(gstreamer::glib::ParamFlags::CONSTRUCT_ONLY) {
            flags.push("construct-only".to_string());
        }

        // For default value, we'll use a simple string representation
        let default_value = {
            let value = element.property_value(prop.name());
            Some(format!("{:?}", value))
        };

        prop_infos.push(PropertyInfo {
            name,
            type_name,
            description,
            flags,
            default_value,
        });
    }

    prop_infos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(prop_infos)
}

fn get_pad_templates(factory: &gst::ElementFactory) -> Vec<PadTemplateInfo> {
    let mut templates = Vec::new();

    for pad_template in factory.static_pad_templates() {
        let caps = pad_template.caps();
        let caps_str = caps.to_string();

        templates.push(PadTemplateInfo {
            name: pad_template.name_template().to_string(),
            direction: format!("{:?}", pad_template.direction()),
            presence: format!("{:?}", pad_template.presence()),
            caps: caps_str,
        });
    }

    templates
}

pub fn search_elements(query: &str, max_results: usize) -> Result<Vec<ElementInfo>> {
    let all_elements = discover_all_elements()?;
    let query_lower = query.to_lowercase();

    let mut matches: Vec<(ElementInfo, i32)> = all_elements
        .into_iter()
        .filter_map(|element| {
            let name_lower = element.name.to_lowercase();
            let desc_lower = element.description.to_lowercase();
            let class_lower = element.classification.to_lowercase();

            let mut score = 0;

            // Exact name match gets highest score
            if name_lower == query_lower {
                score += 100;
            }
            // Name contains query
            else if name_lower.contains(&query_lower) {
                score += 50;
            }

            // Description contains query
            if desc_lower.contains(&query_lower) {
                score += 20;
            }

            // Classification contains query
            if class_lower.contains(&query_lower) {
                score += 10;
            }

            if score > 0 {
                Some((element, score))
            } else {
                None
            }
        })
        .collect();

    // Sort by score (highest first)
    matches.sort_by(|a, b| b.1.cmp(&a.1));

    // Take only the requested number of results
    Ok(matches
        .into_iter()
        .take(max_results)
        .map(|(element, _)| element)
        .collect())
}
