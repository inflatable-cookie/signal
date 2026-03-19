//! VST3 plugin adapter surfaces for Signal.

use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginInstanceId, PluginIoLayout,
    PluginLifecycleContract, PluginParameterDescriptor, PluginParameterDomain,
    PluginParameterFlags, PluginProcessingContract, PluginSandboxCapabilities, PluginStateContract,
    PluginTypeId, SandboxTransport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3HostPlatform {
    MacOs,
    Linux,
    Windows,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3ScanRootKind {
    UserBundleRoot,
    SystemBundleRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ScanRoot {
    pub root: String,
    pub platform: Vst3HostPlatform,
    pub kind: Vst3ScanRootKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vst3DiscoveredPluginType {
    pub plugin_type_id: PluginTypeId,
    pub class_id: String,
    pub controller_class_id: Option<String>,
    pub category: String,
    pub module_root: String,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vst3InstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub class_id: String,
    pub controller_class_id: Option<String>,
    pub module_root: String,
    pub default_io_layout: PluginIoLayout,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ProcessSessionPlan {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub class_id: String,
    pub controller_class_id: Option<String>,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub transport: SandboxTransport,
    pub module_root: String,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vst3HostAdapter {
    strict_sandbox_default: bool,
}

impl Default for Vst3HostAdapter {
    fn default() -> Self {
        Self {
            strict_sandbox_default: true,
        }
    }
}

impl Vst3HostAdapter {
    pub fn strict_sandbox_default(&self) -> bool {
        self.strict_sandbox_default
    }

    pub fn supports_format(&self, format: PluginFormat) -> bool {
        matches!(format, PluginFormat::Vst3)
    }

    pub fn advertised_capabilities(&self, max_block_frames: u32) -> PluginSandboxCapabilities {
        PluginSandboxCapabilities {
            transport: SandboxTransport::SharedMemory,
            supports_state: true,
            supports_midi: true,
            max_block_frames,
        }
    }

    pub fn default_scan_roots(&self, platform: Vst3HostPlatform) -> Vec<Vst3ScanRoot> {
        match platform {
            Vst3HostPlatform::MacOs => vec![
                Vst3ScanRoot {
                    root: "~/Library/Audio/Plug-Ins/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/Library/Audio/Plug-Ins/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
            Vst3HostPlatform::Linux => vec![
                Vst3ScanRoot {
                    root: "~/.vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/usr/lib/vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
                Vst3ScanRoot {
                    root: "/usr/local/lib/vst3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
            Vst3HostPlatform::Windows => vec![
                Vst3ScanRoot {
                    root: "%LOCALAPPDATA%/Programs/Common/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::UserBundleRoot,
                },
                Vst3ScanRoot {
                    root: "%COMMONPROGRAMFILES%/VST3".into(),
                    platform,
                    kind: Vst3ScanRootKind::SystemBundleRoot,
                },
            ],
        }
    }

    pub fn discover_plugin_type(&self, plugin_type_id: &str) -> Option<Vst3DiscoveredPluginType> {
        vst3_discovered_plugin_type(plugin_type_id)
    }

    pub fn discover_plugins_for_roots(
        &self,
        platform: Vst3HostPlatform,
        roots: &[String],
    ) -> Vec<Vst3DiscoveredPluginType> {
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

        let fixture_ids = match platform {
            Vst3HostPlatform::MacOs => vec![
                "plugin:vst3:instrument",
                "plugin:vst3:multiout-instrument",
                "plugin:vst3:utility",
                "plugin:vst3:bus-fx",
            ],
            Vst3HostPlatform::Linux => vec![
                "plugin:vst3:linux-synth",
                "plugin:vst3:multiout-instrument",
                "plugin:vst3:utility",
                "plugin:vst3:bus-fx",
            ],
            Vst3HostPlatform::Windows => vec![
                "plugin:vst3:instrument",
                "plugin:vst3:multiout-instrument",
                "plugin:vst3:utility",
                "plugin:vst3:bus-fx",
            ],
        };

        fixture_ids
            .into_iter()
            .filter_map(|plugin_type_id| {
                let mut discovered = self.discover_plugin_type(plugin_type_id)?;
                discovered.module_root = format!(
                    "{}/{}",
                    matched_roots[0],
                    vst3_fixture_bundle_name(plugin_type_id)
                );
                Some(discovered)
            })
            .collect()
    }

    pub fn instantiate_plugin(
        &self,
        discovered: &Vst3DiscoveredPluginType,
        instance_id: &str,
    ) -> Vst3InstanceControlSurface {
        Vst3InstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            class_id: discovered.class_id.clone(),
            controller_class_id: discovered.controller_class_id.clone(),
            module_root: discovered.module_root.clone(),
            default_io_layout: discovered.default_io_layout,
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        }
    }

    pub fn prepare_session(
        &self,
        instance: &Vst3InstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> Vst3ProcessSessionPlan {
        Vst3ProcessSessionPlan {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            class_id: instance.class_id.clone(),
            controller_class_id: instance.controller_class_id.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: instance.default_io_layout,
            transport: SandboxTransport::SharedMemory,
            module_root: instance.module_root.clone(),
            summary: format!(
                "plugin_type={} class={} controller={} sample_rate={} max_block_frames={} module_root={}",
                instance.plugin_type_id.0,
                instance.class_id,
                instance
                    .controller_class_id
                    .as_deref()
                    .unwrap_or("none"),
                sample_rate_hz,
                max_block_frames,
                instance.module_root,
            ),
        }
    }
}

fn known_root_matches(known_roots: &[String], root: &str) -> bool {
    known_roots.iter().any(|known| known == root)
}

fn vst3_fixture_bundle_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:instrument" => "Signal Instrument.vst3",
        "plugin:vst3:multiout-instrument" => "Signal Multi Output Instrument.vst3",
        "plugin:vst3:linux-synth" => "Signal Linux Synth.vst3",
        "plugin:vst3:utility" => "Signal Utility.vst3",
        "plugin:vst3:bus-fx" => "Signal Bus FX.vst3",
        _ => "Signal Unknown.vst3",
    }
}

fn vst3_default_io_layout(plugin_type_id: &str) -> PluginIoLayout {
    match plugin_type_id {
        "plugin:vst3:multiout-instrument" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:vst3:instrument" | "plugin:vst3:linux-synth" => PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        "plugin:vst3:bus-fx" => PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        "plugin:vst3:utility" => PluginIoLayout {
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

fn vst3_fixture_name(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:instrument" => "Signal Instrument VST3 Plugin",
        "plugin:vst3:multiout-instrument" => "Signal Multi Output Instrument VST3 Plugin",
        "plugin:vst3:linux-synth" => "Signal Linux Synth VST3 Plugin",
        "plugin:vst3:utility" => "Signal Utility VST3 Plugin",
        "plugin:vst3:bus-fx" => "Signal Bus FX VST3 Plugin",
        _ => "Signal Generic VST3 Plugin",
    }
}

fn vst3_fixture_features(plugin_type_id: &str) -> Vec<PluginFeature> {
    match plugin_type_id {
        "plugin:vst3:instrument"
        | "plugin:vst3:multiout-instrument"
        | "plugin:vst3:linux-synth" => {
            vec![PluginFeature::Instrument, PluginFeature::Analyzer]
        }
        "plugin:vst3:bus-fx" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        "plugin:vst3:utility" => vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        _ => vec![PluginFeature::Utility],
    }
}

fn vst3_fixture_descriptor(plugin_type_id: &str, io_layout: PluginIoLayout) -> PluginDescriptor {
    let mut descriptor = PluginDescriptor::new(
        plugin_type_id.to_string(),
        "Signal",
        vst3_fixture_name(plugin_type_id),
        PluginFormat::Vst3,
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
    for feature in vst3_fixture_features(plugin_type_id) {
        descriptor = descriptor.with_feature(feature);
    }
    descriptor
}

fn vst3_discovered_plugin_type(plugin_type_id: &str) -> Option<Vst3DiscoveredPluginType> {
    let (class_id, controller_class_id, category) = match plugin_type_id {
        "plugin:vst3:instrument" => (
            "7E1D8F8A4D874D56A2C44DE250100001",
            Some("7E1D8F8A4D874D56A2C44DE250100002"),
            "Instrument",
        ),
        "plugin:vst3:multiout-instrument" => (
            "7E1D8F8A4D874D56A2C44DE250100011",
            Some("7E1D8F8A4D874D56A2C44DE250100012"),
            "Instrument",
        ),
        "plugin:vst3:linux-synth" => (
            "7E1D8F8A4D874D56A2C44DE250100101",
            Some("7E1D8F8A4D874D56A2C44DE250100102"),
            "Instrument",
        ),
        "plugin:vst3:utility" => (
            "7E1D8F8A4D874D56A2C44DE250100201",
            Some("7E1D8F8A4D874D56A2C44DE250100202"),
            "Fx",
        ),
        "plugin:vst3:bus-fx" => (
            "7E1D8F8A4D874D56A2C44DE250100211",
            Some("7E1D8F8A4D874D56A2C44DE250100212"),
            "Fx",
        ),
        _ => return None,
    };
    let default_io_layout = vst3_default_io_layout(plugin_type_id);
    Some(Vst3DiscoveredPluginType {
        plugin_type_id: PluginTypeId(plugin_type_id.to_string()),
        class_id: class_id.into(),
        controller_class_id: controller_class_id.map(str::to_string),
        category: category.into(),
        module_root: format!("fixture://{}", vst3_fixture_bundle_name(plugin_type_id)),
        descriptor: vst3_fixture_descriptor(plugin_type_id, default_io_layout),
        default_io_layout,
    })
}

#[cfg(test)]
mod tests {
    use super::{Vst3HostAdapter, Vst3HostPlatform};
    use signal_plugin::PluginFormat;

    #[test]
    fn vst3_adapter_reports_supported_format_and_capabilities() {
        let adapter = Vst3HostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Vst3));
        assert!(!adapter.supports_format(PluginFormat::Clap));
        assert!(adapter.strict_sandbox_default());
        assert!(adapter.advertised_capabilities(2048).supports_state);
    }

    #[test]
    fn vst3_adapter_discovers_linux_scan_roots_and_plugin_types() {
        let adapter = Vst3HostAdapter::default();
        let linux_roots = adapter
            .default_scan_roots(Vst3HostPlatform::Linux)
            .into_iter()
            .map(|root| root.root)
            .collect::<Vec<_>>();
        assert!(linux_roots.iter().any(|root| root == "~/.vst3"));
        assert!(linux_roots.iter().any(|root| root == "/usr/lib/vst3"));

        let discovered = adapter.discover_plugins_for_roots(
            Vst3HostPlatform::Linux,
            &[String::from("~/.vst3"), String::from("/usr/lib/vst3")],
        );
        assert_eq!(discovered.len(), 4);
        assert_eq!(discovered[0].descriptor.format, PluginFormat::Vst3);
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:vst3:linux-synth"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:vst3:multiout-instrument"));
        assert!(discovered
            .iter()
            .any(|plugin| plugin.plugin_type_id.0 == "plugin:vst3:bus-fx"));
        assert!(discovered
            .iter()
            .all(|plugin| plugin.module_root.starts_with("~/.vst3/")));
    }

    #[test]
    fn vst3_session_plan_preserves_controller_pairing_and_transport() {
        let adapter = Vst3HostAdapter::default();
        let discovered = adapter
            .discover_plugin_type("plugin:vst3:instrument")
            .expect("discovered vst3 instrument");
        let instance = adapter.instantiate_plugin(&discovered, "instance:vst3:test");
        let session = adapter.prepare_session(&instance, 48_000, 512);

        assert_eq!(session.plugin_type_id.0, "plugin:vst3:instrument");
        assert_eq!(
            session.controller_class_id.as_deref(),
            Some("7E1D8F8A4D874D56A2C44DE250100002")
        );
        assert_eq!(session.io_layout.audio_outputs, 2);
        assert_eq!(
            session.transport,
            signal_plugin::SandboxTransport::SharedMemory
        );
        assert!(session.summary.contains("plugin:vst3:instrument"));
    }
}
