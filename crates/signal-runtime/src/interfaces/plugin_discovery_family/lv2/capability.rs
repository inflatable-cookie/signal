use super::*;
#[path = "capability/pin_matrix.rs"]
mod pin_matrix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2WorkerCapability {
    Absent,
    Supported,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2UridCapability {
    NotRequired,
    Supported,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2PatchCapability {
    Absent,
    Supported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionCapabilitySummary {
    pub worker_capability: RuntimeLv2WorkerCapability,
    pub urid_capability: RuntimeLv2UridCapability,
    pub patch_capability: RuntimeLv2PatchCapability,
    pub negotiated_extension_count: usize,
    pub summary: String,
}

impl RuntimeLv2ExtensionCapabilitySummary {
    pub fn absent() -> Self {
        Self {
            worker_capability: RuntimeLv2WorkerCapability::Absent,
            urid_capability: RuntimeLv2UridCapability::NotRequired,
            patch_capability: RuntimeLv2PatchCapability::Absent,
            negotiated_extension_count: 0,
            summary: "worker=Absent urid=NotRequired patch=Absent extensions=0".into(),
        }
    }

    pub fn from_lv2_feature_uris(
        required_features: &[String],
        supported_extensions: &[String],
    ) -> Self {
        let supports_worker = required_features
            .iter()
            .chain(supported_extensions.iter())
            .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/worker#"));
        let requires_worker = required_features
            .iter()
            .any(|feature| feature == "http://lv2plug.in/ns/ext/worker#schedule");
        let supports_urid = required_features
            .iter()
            .chain(supported_extensions.iter())
            .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/urid#"));
        let requires_urid = required_features
            .iter()
            .any(|feature| feature == "http://lv2plug.in/ns/ext/urid#map");
        let supports_patch = required_features
            .iter()
            .chain(supported_extensions.iter())
            .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/patch#"));
        let negotiated_extension_count = required_features
            .iter()
            .chain(supported_extensions.iter())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let worker_capability = if requires_worker {
            RuntimeLv2WorkerCapability::Required
        } else if supports_worker {
            RuntimeLv2WorkerCapability::Supported
        } else {
            RuntimeLv2WorkerCapability::Absent
        };
        let urid_capability = if requires_urid {
            RuntimeLv2UridCapability::Required
        } else if supports_urid {
            RuntimeLv2UridCapability::Supported
        } else {
            RuntimeLv2UridCapability::NotRequired
        };
        let patch_capability = if supports_patch {
            RuntimeLv2PatchCapability::Supported
        } else {
            RuntimeLv2PatchCapability::Absent
        };
        let mut summary = Self {
            worker_capability,
            urid_capability,
            patch_capability,
            negotiated_extension_count,
            summary: String::new(),
        };
        summary.summary = format!(
            "worker={:?} urid={:?} patch={:?} extensions={}",
            summary.worker_capability,
            summary.urid_capability,
            summary.patch_capability,
            summary.negotiated_extension_count,
        );
        summary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2WorkerPosture {
    WorkerAbsent,
    WorkerAvailable,
    WorkerRequiredAvailable,
    WorkerGuarded,
    WorkerUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2UridNegotiationPosture {
    NotRequired,
    Negotiated,
    Guarded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2PatchExchangePosture {
    Absent,
    Supported,
    Guarded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2ExtensionNegotiationState {
    NotRequired,
    Negotiated,
    PartiallySatisfied,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionRecord {
    pub plugin_type_id: String,
    pub plugin_id: String,
    pub worker_posture: RuntimeLv2WorkerPosture,
    pub urid_negotiation_posture: RuntimeLv2UridNegotiationPosture,
    pub patch_exchange_posture: RuntimeLv2PatchExchangePosture,
    pub extension_negotiation_state: RuntimeLv2ExtensionNegotiationState,
    pub strongest_lifecycle_state: Option<RuntimePluginLifecycleState>,
    pub sandbox_count: usize,
    pub active_sandbox_count: usize,
    pub faulted_sandbox_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionSnapshot {
    pub plugin_type_count: usize,
    pub sandbox_count: usize,
    pub worker_required_type_count: usize,
    pub worker_guarded_type_count: usize,
    pub urid_negotiated_type_count: usize,
    pub patch_supported_type_count: usize,
    pub negotiated_type_count: usize,
    pub guarded_type_count: usize,
    pub unavailable_type_count: usize,
    pub records: Vec<RuntimeLv2ExtensionRecord>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginCapabilityCoverageSummary {
    pub discovered_format_count: usize,
    pub multi_format_catalog: bool,
    pub complex_io_type_count: usize,
    pub multi_output_instrument_count: usize,
    pub bus_capable_fx_count: usize,
    pub sidechain_capable_fx_count: usize,
    pub instrument_count: usize,
    pub audio_effect_count: usize,
    pub analyzer_count: usize,
    pub utility_count: usize,
    pub note_effect_count: usize,
    pub supports_snapshot_count: usize,
    pub supports_reset_count: usize,
    pub supports_bypass_count: usize,
    pub exposes_latency_count: usize,
    pub exposes_tail_count: usize,
    pub sample_accurate_automation_count: usize,
    pub accepts_midi_count: usize,
    pub accepts_note_events_count: usize,
    pub supports_note_expression_count: usize,
    pub produces_midi_count: usize,
    pub silence_aware_count: usize,
    pub requires_main_thread_for_state_count: usize,
    pub supports_prepare_count: usize,
    pub supports_activate_count: usize,
    pub supports_reset_while_active_count: usize,
    pub max_complex_io_port_group_count: usize,
    pub max_audio_bus_count: usize,
    pub max_parameter_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginDiscoverySnapshot {
    pub scan_count: usize,
    pub format_filtered_scan_count: usize,
    pub discovered_type_count: usize,
    pub discovered_format_count: usize,
    pub last_scan: Option<RuntimePluginScanReceipt>,
    pub format_coverage: Vec<RuntimePluginFormatCoverageRecord>,
    pub parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    pub capability_coverage: RuntimePluginCapabilityCoverageSummary,
    pub discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
    pub summary: String,
}
