//! Audio Unit plugin adapter surfaces for Signal.

use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginInstanceId, PluginIoLayout,
    PluginLifecycleContract, PluginParameterDescriptor, PluginParameterDomain,
    PluginParameterFlags, PluginProcessingContract, PluginSandboxCapabilities, PluginStateContract,
    PluginTypeId, SandboxTransport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuHostPlatform {
    MacOs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuScanRootKind {
    UserComponentRoot,
    SystemComponentRoot,
    BuiltInComponentRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuScanRoot {
    pub root: String,
    pub platform: AuHostPlatform,
    pub kind: AuScanRootKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuDiscoveredPluginType {
    pub plugin_type_id: PluginTypeId,
    pub component_type: String,
    pub component_subtype: String,
    pub manufacturer_code: String,
    pub bundle_root: String,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuInstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub component_type: String,
    pub component_subtype: String,
    pub manufacturer_code: String,
    pub bundle_root: String,
    pub default_io_layout: PluginIoLayout,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuProcessSessionPlan {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub component_type: String,
    pub component_subtype: String,
    pub manufacturer_code: String,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub transport: SandboxTransport,
    pub bundle_root: String,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuHostAdapter {
    strict_sandbox_default: bool,
}

impl Default for AuHostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
        }
    }
}

impl AuHostAdapter {
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Au)
    }

    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }

    pub fn default_scan_roots(&self, platform: AuHostPlatform) -> Vec<AuScanRoot> {
        match platform {
            AuHostPlatform::MacOs => vec![
                AuScanRoot {
                    root: "~/Library/Audio/Plug-Ins/Components".into(),
                    platform,
                    kind: AuScanRootKind::UserComponentRoot,
                },
                AuScanRoot {
                    root: "/Library/Audio/Plug-Ins/Components".into(),
                    platform,
                    kind: AuScanRootKind::SystemComponentRoot,
                },
                AuScanRoot {
                    root: "/System/Library/Components".into(),
                    platform,
                    kind: AuScanRootKind::BuiltInComponentRoot,
                },
            ],
        }
    }

    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<AuDiscoveredPluginType> {
        au_discovered_plugin_type(plugin_type_id)
    }

    pub fn discover_plugins_for_roots(
        &self,
        platform: AuHostPlatform,
        roots: &[String],
    ) -> Vec<AuDiscoveredPluginType> {
        let known_roots = self
            .default_scan_roots(platform)
            .into_iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        let matched_roots = if roots.is_empty() {
            known_roots
        } else {
            roots
                .iter()
                .filter(|root| known_root_matches(&known_roots, root))
                .cloned()
                .collect::<Vec<_>>()
        };
        if matched_roots.is_empty() {
            return Vec::new();
        }

        [
            "plugin:au:instrument",
            "plugin:au:multiout-instrument",
            "plugin:au:utility",
            "plugin:au:bus-fx",
        ]
        .into_iter()
        .filter_map(|plugin_type_id| {
            let mut discovered = self.discover_plugin_type(plugin_type_id)?;
            discovered.bundle_root = format!(
                "{}/{}",
                matched_roots[0],
                au_fixture_bundle_name(plugin_type_id)
            );
            Some(discovered)
        })
        .collect()
    }

    pub fn instantiate_plugin(
        &self,
        discovered: &AuDiscoveredPluginType,
        instance_id: &str,
    ) -> AuInstanceControlSurface {
        AuInstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            component_type: discovered.component_type.clone(),
            component_subtype: discovered.component_subtype.clone(),
            manufacturer_code: discovered.manufacturer_code.clone(),
            bundle_root: discovered.bundle_root.clone(),
            default_io_layout: discovered.default_io_layout,
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        }
    }

    pub fn prepare_session(
        &self,
        instance: &AuInstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> AuProcessSessionPlan {
        AuProcessSessionPlan {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            component_type: instance.component_type.clone(),
            component_subtype: instance.component_subtype.clone(),
            manufacturer_code: instance.manufacturer_code.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: instance.default_io_layout,
            transport: SandboxTransport::SharedMemory,
            bundle_root: instance.bundle_root.clone(),
            summary: format!(
                "plugin_type={} component_type={} component_subtype={} manufacturer={} sample_rate={} max_block_frames={} bundle_root={}",
                instance.plugin_type_id.0,
                instance.component_type,
                instance.component_subtype,
                instance.manufacturer_code,
                sample_rate_hz,
                max_block_frames,
                instance.bundle_root,
            ),
        }
    }
}

fn known_root_matches(known_roots: &[String], root: &str) -> bool {
    known_roots.iter().any(|known| known == root)
}

fn au_fixture_bundle_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:au:instrument" => "Signal Instrument.component",
        "plugin:au:multiout-instrument" => "Signal Multi Output Instrument.component",
        "plugin:au:utility" => "Signal Utility.component",
        "plugin:au:bus-fx" => "Signal Bus FX.component",
        _ => "Signal Unknown.component",
    }
}

fn au_default_io_layout(plugin_type_id: &str) -> PluginIoLayout {
    match plugin_type_id {
        "plugin:au:multiout-instrument" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:au:instrument" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:au:bus-fx" => PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        "plugin:au:utility" => PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        _ => PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 0,
            midi_outputs: 0,
        },
    }
}

fn au_fixture_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:au:instrument" => "Signal Instrument AU Plugin",
        "plugin:au:multiout-instrument" => "Signal Multi Output Instrument AU Plugin",
        "plugin:au:utility" => "Signal Utility AU Plugin",
        "plugin:au:bus-fx" => "Signal Bus FX AU Plugin",
        _ => "Signal Generic AU Plugin",
    }
}

fn au_fixture_features(plugin_type_id: &str) -> Vec<PluginFeature> {
    match plugin_type_id {
        "plugin:au:instrument" | "plugin:au:multiout-instrument" => {
            vec![PluginFeature::Instrument, PluginFeature::Analyzer]
        }
        "plugin:au:bus-fx" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        "plugin:au:utility" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        _ => vec![PluginFeature::Utility],
    }
}

fn au_fixture_descriptor(plugin_type_id: &str, io_layout: PluginIoLayout) -> PluginDescriptor {
    let mut descriptor = PluginDescriptor::new(
        plugin_type_id.to_string(),
        "Signal",
        au_fixture_name(plugin_type_id),
        PluginFormat::Au,
    )
    .with_version("0.1.0")
    .with_audio_buses(io_layout.main_audio_buses())
    .with_parameters(vec![
        PluginParameterDescriptor {
            parameter_id: 1,
            name: "Output Trim".into(),
            unit: Some("dB".into()),
            domain: PluginParameterDomain::Decibels,
            default_normalized: 0.5,
            min_plain: -24.0,
            max_plain: 24.0,
            flags: PluginParameterFlags::automatable(),
        },
        PluginParameterDescriptor {
            parameter_id: 2,
            name: "Bypass".into(),
            unit: None,
            domain: PluginParameterDomain::Bypass,
            default_normalized: 0.0,
            min_plain: 0.0,
            max_plain: 1.0,
            flags: PluginParameterFlags::bypass(),
        },
    ])
    .with_state_contract(PluginStateContract {
        supports_snapshot: true,
        supports_reset: true,
        supports_bypass: true,
        exposes_latency: false,
        exposes_tail: true,
    })
    .with_processing_contract(PluginProcessingContract {
        max_block_frames: 4_096,
        sample_accurate_automation: false,
        accepts_midi: io_layout.midi_inputs > 0,
        accepts_note_events: io_layout.midi_inputs > 0,
        supports_note_expression: io_layout.midi_inputs > 0,
        produces_midi: false,
        silence_aware: false,
    })
    .with_lifecycle_contract(PluginLifecycleContract {
        requires_main_thread_for_state: true,
        supports_prepare: true,
        supports_activate: true,
        supports_reset_while_active: false,
    });
    for feature in au_fixture_features(plugin_type_id) {
        descriptor = descriptor.with_feature(feature);
    }
    descriptor
}

fn au_discovered_plugin_type(plugin_type_id: &str) -> Option<AuDiscoveredPluginType> {
    let (component_type, component_subtype, manufacturer_code) = match plugin_type_id {
        "plugin:au:instrument" => ("aumu", "sigi", "sigl"),
        "plugin:au:multiout-instrument" => ("aumu", "sigm", "sigl"),
        "plugin:au:utility" => ("aufx", "sigu", "sigl"),
        "plugin:au:bus-fx" => ("aufx", "sigb", "sigl"),
        _ => return None,
    };
    let default_io_layout = au_default_io_layout(plugin_type_id);
    Some(AuDiscoveredPluginType {
        plugin_type_id: PluginTypeId(plugin_type_id.to_string()),
        component_type: component_type.into(),
        component_subtype: component_subtype.into(),
        manufacturer_code: manufacturer_code.into(),
        bundle_root: format!("fixture://{}", au_fixture_bundle_name(plugin_type_id)),
        descriptor: au_fixture_descriptor(plugin_type_id, default_io_layout),
        default_io_layout,
    })
}

#[cfg(test)]
mod tests {
    use super::{AuHostAdapter, AuHostPlatform};
    use signal_plugin::PluginFormat;

    #[test]
    fn au_adapter_reports_supported_format_and_capabilities() {
        let adapter = AuHostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Au));
        assert!(!adapter.supports_format(PluginFormat::Clap));
        assert!(adapter.strict_sandbox_default());
        assert!(adapter.advertised_capabilities(2048).supports_state);
    }

    #[test]
    fn au_adapter_discovers_macos_scan_roots_and_plugin_types() {
        let adapter = AuHostAdapter::default();
        let mac_roots = adapter
            .default_scan_roots(AuHostPlatform::MacOs)
            .into_iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        assert!(mac_roots
            .iter()
            .any(|root| root == "~/Library/Audio/Plug-Ins/Components"));
        assert!(mac_roots
            .iter()
            .any(|root| root == "/Library/Audio/Plug-Ins/Components"));

        let discovered = adapter.discover_plugins_for_roots(
            AuHostPlatform::MacOs,
            &[
                String::from("~/Library/Audio/Plug-Ins/Components"),
                String::from("/Library/Audio/Plug-Ins/Components"),
            ],
        );
        assert_eq!(discovered.len(), 4);
        assert_eq!(discovered[0].descriptor.format, PluginFormat::Au);
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:au:instrument"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:au:multiout-instrument"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:au:bus-fx"));
        assert!(discovered.iter().all(|plugin| plugin
            .bundle_root
            .starts_with("~/Library/Audio/Plug-Ins/Components/")));
    }

    #[test]
    fn au_session_plan_preserves_component_identity_and_transport() {
        let adapter = AuHostAdapter::default();
        let discovered = adapter
            .discover_plugin_type("plugin:au:instrument")
            .expect("discovered au instrument");
        let instance = adapter.instantiate_plugin(&discovered, "instance:au:test");
        let session = adapter.prepare_session(&instance, 48_000, 512);

        assert_eq!(session.plugin_type_id.0, "plugin:au:instrument");
        assert_eq!(session.component_type, "aumu");
        assert_eq!(session.component_subtype, "sigi");
        assert_eq!(session.io_layout.audio_outputs, 2);
        assert_eq!(
            session.transport,
            signal_plugin::SandboxTransport::SharedMemory
        );
        assert!(session.summary.contains("plugin:au:instrument"));
    }
}
