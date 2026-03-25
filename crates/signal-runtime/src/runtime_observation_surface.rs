#[path = "runtime_observation_surface/diagnostics.rs"]
mod diagnostics;
#[path = "runtime_observation_surface/observation_api.rs"]
mod observation_api;
#[path = "runtime_observation_surface/plugin_chain.rs"]
mod plugin_chain;
#[path = "runtime_observation_surface/plugin_chain_stage.rs"]
mod plugin_chain_stage;
#[path = "runtime_observation_surface/plugin_handoff.rs"]
mod plugin_handoff;
#[path = "runtime_observation_surface/plugin_lifecycle.rs"]
mod plugin_lifecycle;
#[path = "runtime_observation_surface/plugin_recall.rs"]
mod plugin_recall;
#[path = "runtime_observation_surface/tempo_media.rs"]
mod tempo_media;
#[path = "runtime_observation_surface/transform_chain.rs"]
mod transform_chain;
#[path = "runtime_observation_surface/transport_media.rs"]
mod transport_media;

use super::*;
use plugin_recall::{runtime_plugin_compensation_observation, runtime_plugin_recall_snapshot};
