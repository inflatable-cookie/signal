use std::{path::Path, sync::Arc};

use signal_plugin::{PluginFormat, PluginIsolationTier};
use signal_plugin_bridge::{
    InProcessAuProcessor, InProcessClapProcessor, InProcessLv2Processor, InProcessVst3Processor,
    ShmPluginProcessor,
};
use signal_render_plane::{PluginBlockProcessor, RenderPluginProcessor};
use signal_runtime::{
    PluginSandboxSpec, RuntimeError, RuntimeErrorKind, RuntimeSupervisorApi,
    SandboxPluginActivateOutcome,
};

use super::super::LocalRuntimeHost;

impl LocalRuntimeHost {
    /// Construct a render-plane plugin processor from a previously scanned type.
    ///
    /// Isolation routing:
    /// - [`PluginIsolationTier::InProcess`] loads and activates the matching
    ///   in-process bridge backend.
    /// - [`PluginIsolationTier::DedicatedSandbox`] requires a live broker
    ///   session for that type, then attaches [`ShmPluginProcessor`] from the
    ///   child's audio-block lease.
    /// - [`PluginIsolationTier::SharedSandbox`] is rejected until `g11.002`.
    pub fn prepare_plugin_processor(
        &mut self,
        plugin_type_id: &str,
        tier: PluginIsolationTier,
    ) -> Result<RenderPluginProcessor, RuntimeError> {
        match tier {
            PluginIsolationTier::SharedSandbox => Err(RuntimeError::new(
                RuntimeErrorKind::UnsupportedCapability,
                "shared_sandbox_unimplemented",
            )),
            PluginIsolationTier::InProcess => {
                let discovered = self.require_discovered_plugin(plugin_type_id)?;
                load_in_process_backend(
                    &discovered,
                    self.runtime.config().sample_rate.0,
                    self.runtime.config().graph.block_size as u32,
                )
                .map(RenderPluginProcessor::new)
                .map_err(map_bridge_error)
            }
            PluginIsolationTier::DedicatedSandbox => {
                self.prepare_dedicated_sandbox_processor(plugin_type_id)
            }
        }
    }

    fn prepare_dedicated_sandbox_processor(
        &mut self,
        plugin_type_id: &str,
    ) -> Result<RenderPluginProcessor, RuntimeError> {
        let discovered = self.require_discovered_plugin(plugin_type_id)?;
        let sandbox_id = self.ensure_broker_session_for_discovered(&discovered)?;
        let sample_rate_hz = self.runtime.config().sample_rate.0;
        let max_frames = self.runtime.config().graph.block_size as u32;
        let session = self.sandbox_broker_sessions.get_mut(&sandbox_id).ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!(
                    "no broker lease is available for dedicated-sandbox plugin type {plugin_type_id}"
                ),
            )
        })?;
        session
            .client
            .load_plugin(discovered.library_path(), &discovered.load_key())
            .map_err(|error| {
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!("sandbox broker load-plugin failed: {error}"),
                )
            })?;
        let lease = match session
            .client
            .activate_plugin(sample_rate_hz, 1, max_frames)
            .map_err(|error| {
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!("sandbox broker activate failed: {error}"),
                )
            })? {
            SandboxPluginActivateOutcome::Activated(lease) => lease,
            SandboxPluginActivateOutcome::LayoutUnsupported { detail } => {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("layout_unsupported: {detail}"),
                ));
            }
        };
        session.client.start_processing().map_err(|error| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!("sandbox broker start-processing failed: {error}"),
            )
        })?;
        wrap_backend(ShmPluginProcessor::attach(
            &lease.region_id,
            &lease.shm_path,
            lease.shm_bytes,
            lease.max_frames,
            lease.channels,
            sample_rate_hz,
        ))
    }

    fn ensure_broker_session_for_discovered(
        &mut self,
        discovered: &DiscoveredHostPlugin,
    ) -> Result<String, RuntimeError> {
        if let Some(sandbox_id) = self.existing_broker_sandbox_id(discovered.plugin_type_id()) {
            return Ok(sandbox_id);
        }
        let sandbox_id = format!("prepare:{}", discovered.plugin_type_id());
        self.ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: sandbox_id.clone(),
            plugin_format: discovered.format(),
            plugin_type_id: Some(discovered.plugin_type_id().to_string()),
        })?;
        Ok(sandbox_id)
    }

    fn existing_broker_sandbox_id(&self, plugin_type_id: &str) -> Option<String> {
        self.active_sandbox_specs
            .iter()
            .find_map(|(sandbox_id, spec)| {
                let matches_type = spec.plugin_type_id.as_deref() == Some(plugin_type_id);
                (matches_type && self.sandbox_broker_sessions.contains_key(sandbox_id))
                    .then(|| sandbox_id.clone())
            })
    }

    fn require_discovered_plugin(
        &self,
        plugin_type_id: &str,
    ) -> Result<DiscoveredHostPlugin, RuntimeError> {
        if let Some(plugin) = self.discovered_clap_types.get(plugin_type_id) {
            return Ok(DiscoveredHostPlugin::Clap(plugin.clone()));
        }
        if let Some(plugin) = self.discovered_vst3_types.get(plugin_type_id) {
            return Ok(DiscoveredHostPlugin::Vst3(plugin.clone()));
        }
        if let Some(plugin) = self.discovered_au_types.get(plugin_type_id) {
            return Ok(DiscoveredHostPlugin::Au(plugin.clone()));
        }
        if let Some(plugin) = self.discovered_lv2_types.get(plugin_type_id) {
            return Ok(DiscoveredHostPlugin::Lv2(plugin.clone()));
        }
        Err(RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!("plugin type {plugin_type_id} was not discovered in the last local scan"),
        ))
    }
}

#[derive(Clone)]
enum DiscoveredHostPlugin {
    Clap(signal_plugin_clap::ClapDiscoveredPluginType),
    Vst3(signal_plugin_vst3::Vst3DiscoveredPluginType),
    Au(signal_plugin_au::AuDiscoveredPluginType),
    Lv2(signal_plugin_lv2::Lv2DiscoveredPluginType),
}

impl DiscoveredHostPlugin {
    fn plugin_type_id(&self) -> &str {
        match self {
            Self::Clap(plugin) => plugin.plugin_type_id.0.as_str(),
            Self::Vst3(plugin) => plugin.plugin_type_id.0.as_str(),
            Self::Au(plugin) => plugin.plugin_type_id.0.as_str(),
            Self::Lv2(plugin) => plugin.plugin_type_id.0.as_str(),
        }
    }

    fn format(&self) -> PluginFormat {
        match self {
            Self::Clap(_) => PluginFormat::Clap,
            Self::Vst3(_) => PluginFormat::Vst3,
            Self::Au(_) => PluginFormat::Au,
            Self::Lv2(_) => PluginFormat::Lv2,
        }
    }

    fn library_path(&self) -> &str {
        match self {
            Self::Clap(plugin) => plugin.library_path.as_str(),
            Self::Vst3(plugin) => plugin.module_root.as_str(),
            Self::Au(plugin) => plugin.bundle_root.as_str(),
            Self::Lv2(plugin) => plugin.bundle_root.as_str(),
        }
    }

    fn load_key(&self) -> String {
        match self {
            Self::Clap(plugin) => plugin.plugin_type_id.0.clone(),
            Self::Vst3(plugin) => plugin.class_id.clone(),
            Self::Au(plugin) => plugin.load_key(),
            Self::Lv2(plugin) => plugin.plugin_uri.clone(),
        }
    }
}

fn load_in_process_backend(
    discovered: &DiscoveredHostPlugin,
    sample_rate_hz: u32,
    max_frames: u32,
) -> Result<Arc<dyn PluginBlockProcessor>, String> {
    let path = Path::new(discovered.library_path());
    let load_key = discovered.load_key();
    match discovered {
        DiscoveredHostPlugin::Clap(_) => {
            InProcessClapProcessor::load_and_activate(path, &load_key, sample_rate_hz, max_frames)
                .map(|backend| Arc::new(backend) as Arc<dyn PluginBlockProcessor>)
        }
        DiscoveredHostPlugin::Vst3(_) => {
            InProcessVst3Processor::load_and_activate(path, &load_key, sample_rate_hz, max_frames)
                .map(|backend| Arc::new(backend) as Arc<dyn PluginBlockProcessor>)
        }
        DiscoveredHostPlugin::Au(_) => {
            InProcessAuProcessor::load_and_activate(path, &load_key, sample_rate_hz, max_frames)
                .map(|backend| Arc::new(backend) as Arc<dyn PluginBlockProcessor>)
        }
        DiscoveredHostPlugin::Lv2(_) => {
            InProcessLv2Processor::load_and_activate(path, &load_key, sample_rate_hz, max_frames)
                .map(|backend| Arc::new(backend) as Arc<dyn PluginBlockProcessor>)
        }
    }
}

fn wrap_backend<T>(backend: Result<T, String>) -> Result<RenderPluginProcessor, RuntimeError>
where
    T: PluginBlockProcessor + 'static,
{
    backend
        .map(|backend| RenderPluginProcessor::new(Arc::new(backend)))
        .map_err(map_bridge_error)
}

fn map_bridge_error(error: String) -> RuntimeError {
    let kind = if error.contains("layout_unsupported") {
        RuntimeErrorKind::InvalidRequest
    } else {
        RuntimeErrorKind::ResourceUnavailable
    };
    RuntimeError::new(kind, error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_bridge_error_keeps_layout_unsupported_as_invalid_request() {
        let error = map_bridge_error("layout_unsupported".into());
        assert_eq!(error.kind, RuntimeErrorKind::InvalidRequest);
        assert_eq!(error.message, "layout_unsupported");
    }

    #[test]
    fn map_bridge_error_maps_load_failure_to_resource_unavailable() {
        let error = map_bridge_error("library_open_failed".into());
        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert_eq!(error.message, "library_open_failed");
    }
}
