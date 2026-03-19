//! LV2 plugin adapter surfaces for Signal.

use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginInstanceId, PluginIoLayout,
    PluginLifecycleContract, PluginParameterDescriptor, PluginParameterDomain,
    PluginParameterFlags, PluginProcessingContract, PluginSandboxCapabilities, PluginStateContract,
    PluginTypeId, SandboxTransport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2HostPlatform {
    Linux,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2ScanRootKind {
    UserBundleRoot,
    SystemBundleRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2ScanRoot {
    pub root: String,
    pub platform: Lv2HostPlatform,
    pub kind: Lv2ScanRootKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lv2DiscoveredPluginType {
    pub plugin_type_id: PluginTypeId,
    pub plugin_uri: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub required_features: Vec<String>,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lv2InstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub plugin_uri: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub required_features: Vec<String>,
    pub default_io_layout: PluginIoLayout,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2ProcessSessionPlan {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub plugin_uri: String,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub transport: SandboxTransport,
    pub bundle_root: String,
    pub manifest_path: String,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lv2HostAdapter {
    strict_sandbox_default: bool,
}

impl Default for Lv2HostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
        }
    }
}

impl Lv2HostAdapter {
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Lv2)
    }

    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }

    pub fn default_scan_roots(&self, platform: Lv2HostPlatform) -> Vec<Lv2ScanRoot> {
        match platform {
            Lv2HostPlatform::Linux => vec![
                Lv2ScanRoot {
                    root: "~/.lv2".into(),
                    platform,
                    kind: Lv2ScanRootKind::UserBundleRoot,
                },
                Lv2ScanRoot {
                    root: "/usr/lib/lv2".into(),
                    platform,
                    kind: Lv2ScanRootKind::SystemBundleRoot,
                },
                Lv2ScanRoot {
                    root: "/usr/local/lib/lv2".into(),
                    platform,
                    kind: Lv2ScanRootKind::SystemBundleRoot,
                },
            ],
        }
    }

    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<Lv2DiscoveredPluginType> {
        lv2_discovered_plugin_type(plugin_type_id)
    }

    pub fn discover_plugins_for_roots(
        &self,
        platform: Lv2HostPlatform,
        roots: &[String],
    ) -> Vec<Lv2DiscoveredPluginType> {
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
            "plugin:lv2:linux-synth",
            "plugin:lv2:multiout-instrument",
            "plugin:lv2:utility",
            "plugin:lv2:bus-fx",
        ]
        .into_iter()
        .filter_map(|plugin_type_id| {
            let mut discovered = self.discover_plugin_type(plugin_type_id)?;
            let bundle_root = format!(
                "{}/{}",
                matched_roots[0],
                lv2_fixture_bundle_name(plugin_type_id)
            );
            discovered.bundle_root = bundle_root.clone();
            discovered.manifest_path = format!("{bundle_root}/manifest.ttl");
            Some(discovered)
        })
        .collect()
    }

    pub fn instantiate_plugin(
        &self,
        discovered: &Lv2DiscoveredPluginType,
        instance_id: &str,
    ) -> Lv2InstanceControlSurface {
        Lv2InstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            plugin_uri: discovered.plugin_uri.clone(),
            bundle_root: discovered.bundle_root.clone(),
            manifest_path: discovered.manifest_path.clone(),
            required_features: discovered.required_features.clone(),
            default_io_layout: discovered.default_io_layout,
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        }
    }

    pub fn prepare_session(
        &self,
        instance: &Lv2InstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> Lv2ProcessSessionPlan {
        Lv2ProcessSessionPlan {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            plugin_uri: instance.plugin_uri.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: instance.default_io_layout,
            transport: SandboxTransport::SharedMemory,
            bundle_root: instance.bundle_root.clone(),
            manifest_path: instance.manifest_path.clone(),
            summary: format!(
                "plugin_type={} uri={} sample_rate={} max_block_frames={} bundle_root={} manifest={} required_features={}",
                instance.plugin_type_id.0,
                instance.plugin_uri,
                sample_rate_hz,
                max_block_frames,
                instance.bundle_root,
                instance.manifest_path,
                instance.required_features.join(","),
            ),
        }
    }
}

fn known_root_matches(known_roots: &[String], root: &str) -> bool {
    known_roots.iter().any(|known| known == root)
}

fn lv2_fixture_bundle_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:lv2:linux-synth" => "Signal Linux Synth.lv2",
        "plugin:lv2:multiout-instrument" => "Signal Multi Output Instrument.lv2",
        "plugin:lv2:utility" => "Signal Utility.lv2",
        "plugin:lv2:bus-fx" => "Signal Bus FX.lv2",
        _ => "Signal Unknown.lv2",
    }
}

fn lv2_default_io_layout(plugin_type_id: &str) -> PluginIoLayout {
    match plugin_type_id {
        "plugin:lv2:multiout-instrument" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:lv2:linux-synth" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:lv2:bus-fx" => PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        "plugin:lv2:utility" => PluginIoLayout {
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

fn lv2_fixture_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:lv2:linux-synth" => "Signal Linux Synth LV2 Plugin",
        "plugin:lv2:multiout-instrument" => "Signal Multi Output Instrument LV2 Plugin",
        "plugin:lv2:utility" => "Signal Utility LV2 Plugin",
        "plugin:lv2:bus-fx" => "Signal Bus FX LV2 Plugin",
        _ => "Signal Generic LV2 Plugin",
    }
}

fn lv2_fixture_uri(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:lv2:linux-synth" => "https://signal.dev/plugins/lv2/linux-synth",
        "plugin:lv2:multiout-instrument" => "https://signal.dev/plugins/lv2/multiout-instrument",
        "plugin:lv2:utility" => "https://signal.dev/plugins/lv2/utility",
        "plugin:lv2:bus-fx" => "https://signal.dev/plugins/lv2/bus-fx",
        _ => "https://signal.dev/plugins/lv2/unknown",
    }
}

fn lv2_fixture_required_features(plugin_type_id: &str) -> Vec<String> {
    match plugin_type_id {
        "plugin:lv2:linux-synth" | "plugin:lv2:multiout-instrument" => vec![
            "http://lv2plug.in/ns/ext/urid#map".into(),
            "http://lv2plug.in/ns/ext/worker#schedule".into(),
        ],
        "plugin:lv2:bus-fx" => vec!["http://lv2plug.in/ns/ext/urid#map".into()],
        "plugin:lv2:utility" => vec!["http://lv2plug.in/ns/ext/options#options".into()],
        _ => Vec::new(),
    }
}

fn lv2_fixture_features(plugin_type_id: &str) -> Vec<PluginFeature> {
    match plugin_type_id {
        "plugin:lv2:linux-synth" | "plugin:lv2:multiout-instrument" => {
            vec![PluginFeature::Instrument, PluginFeature::Analyzer]
        }
        "plugin:lv2:bus-fx" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        "plugin:lv2:utility" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        _ => vec![PluginFeature::Utility],
    }
}

fn lv2_fixture_descriptor(plugin_type_id: &str, io_layout: PluginIoLayout) -> PluginDescriptor {
    let mut descriptor = PluginDescriptor::new(
        plugin_type_id.to_string(),
        "Signal",
        lv2_fixture_name(plugin_type_id),
        PluginFormat::Lv2,
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
        exposes_latency: true,
        exposes_tail: true,
    })
    .with_processing_contract(PluginProcessingContract {
        max_block_frames: 4_096,
        sample_accurate_automation: false,
        accepts_midi: io_layout.midi_inputs > 0,
        accepts_note_events: io_layout.midi_inputs > 0,
        supports_note_expression: false,
        produces_midi: false,
        silence_aware: true,
    })
    .with_lifecycle_contract(PluginLifecycleContract {
        requires_main_thread_for_state: false,
        supports_prepare: true,
        supports_activate: true,
        supports_reset_while_active: false,
    });
    for feature in lv2_fixture_features(plugin_type_id) {
        descriptor = descriptor.with_feature(feature);
    }
    descriptor
}

fn lv2_discovered_plugin_type(plugin_type_id: &str) -> Option<Lv2DiscoveredPluginType> {
    match plugin_type_id {
        "plugin:lv2:linux-synth"
        | "plugin:lv2:multiout-instrument"
        | "plugin:lv2:utility"
        | "plugin:lv2:bus-fx" => {
            let default_io_layout = lv2_default_io_layout(plugin_type_id);
            Some(Lv2DiscoveredPluginType {
                plugin_type_id: PluginTypeId(plugin_type_id.to_string()),
                plugin_uri: lv2_fixture_uri(plugin_type_id).into(),
                bundle_root: format!("fixture://{}", lv2_fixture_bundle_name(plugin_type_id)),
                manifest_path: format!(
                    "fixture://{}/manifest.ttl",
                    lv2_fixture_bundle_name(plugin_type_id)
                ),
                required_features: lv2_fixture_required_features(plugin_type_id),
                descriptor: lv2_fixture_descriptor(plugin_type_id, default_io_layout),
                default_io_layout,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Lv2HostAdapter, Lv2HostPlatform};
    use signal_plugin::PluginFormat;

    #[test]
    fn lv2_adapter_reports_supported_format_and_capabilities() {
        let adapter = Lv2HostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Lv2));
        assert!(!adapter.supports_format(PluginFormat::Clap));
        assert!(adapter.strict_sandbox_default());
        assert!(adapter.advertised_capabilities(2048).supports_state);
    }

    #[test]
    fn lv2_adapter_discovers_linux_scan_roots_and_plugin_types() {
        let adapter = Lv2HostAdapter::default();
        let linux_roots = adapter
            .default_scan_roots(Lv2HostPlatform::Linux)
            .into_iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        assert!(linux_roots.iter().any(|root| root == "~/.lv2"));
        assert!(linux_roots.iter().any(|root| root == "/usr/lib/lv2"));

        let discovered = adapter.discover_plugins_for_roots(
            Lv2HostPlatform::Linux,
            &[String::from("~/.lv2"), String::from("/usr/lib/lv2")],
        );
        assert_eq!(discovered.len(), 4);
        assert_eq!(discovered[0].descriptor.format, PluginFormat::Lv2);
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:lv2:linux-synth"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:lv2:multiout-instrument"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:lv2:bus-fx"));
        assert!(discovered
            .iter()
            .all(|plugin| plugin.bundle_root.starts_with("~/.lv2/")));
        assert!(discovered
            .iter()
            .all(|plugin| plugin.manifest_path.ends_with("/manifest.ttl")));
    }

    #[test]
    fn lv2_session_plan_preserves_uri_manifest_and_transport() {
        let adapter = Lv2HostAdapter::default();
        let discovered = adapter
            .discover_plugin_type("plugin:lv2:linux-synth")
            .expect("discovered lv2 synth");
        let instance = adapter.instantiate_plugin(&discovered, "instance:lv2:test");
        let session = adapter.prepare_session(&instance, 48_000, 512);

        assert_eq!(session.plugin_type_id.0, "plugin:lv2:linux-synth");
        assert_eq!(
            session.plugin_uri,
            "https://signal.dev/plugins/lv2/linux-synth"
        );
        assert_eq!(session.io_layout.audio_outputs, 2);
        assert_eq!(
            session.transport,
            signal_plugin::SandboxTransport::SharedMemory
        );
        assert!(session.summary.contains("manifest.ttl"));
    }
}
