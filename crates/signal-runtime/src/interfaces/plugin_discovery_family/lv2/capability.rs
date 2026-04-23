use super::*;
#[path = "capability/pin_matrix.rs"]
mod pin_matrix;

/// LV2 `worker:schedule` extension status: absent, supported, or required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2WorkerCapability {
    /// Worker scheduling extension is not present.
    Absent,
    /// Worker scheduling extension is present and supported.
    Supported,
    /// Worker scheduling extension is required for operation.
    Required,
}

/// LV2 `urid:map` extension status: not required, supported, or required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2UridCapability {
    /// URID map extension is not required.
    NotRequired,
    /// URID map extension is present and supported.
    Supported,
    /// URID map extension is required for operation.
    Required,
}

/// LV2 `patch` extension status: absent or supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2PatchCapability {
    /// Patch message extension is not present.
    Absent,
    /// Patch message extension is present and supported.
    Supported,
}

/// Declared LV2 extension capabilities for a plugin: worker, URID, patch, and total negotiated extension count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionCapabilitySummary {
    /// Worker thread scheduling capability.
    pub worker_capability: RuntimeLv2WorkerCapability,
    /// URID map capability.
    pub urid_capability: RuntimeLv2UridCapability,
    /// Patch message exchange capability.
    pub patch_capability: RuntimeLv2PatchCapability,
    /// Total number of distinct extensions negotiated.
    pub negotiated_extension_count: usize,
    /// Human-readable one-line summary.
    pub summary: String,
}

impl RuntimeLv2ExtensionCapabilitySummary {
    /// Returns a summary with all extensions absent/not-required and zero negotiated count.
    pub fn absent() -> Self {
        Self {
            worker_capability: RuntimeLv2WorkerCapability::Absent,
            urid_capability: RuntimeLv2UridCapability::NotRequired,
            patch_capability: RuntimeLv2PatchCapability::Absent,
            negotiated_extension_count: 0,
            summary: "worker=Absent urid=NotRequired patch=Absent extensions=0".into(),
        }
    }

    /// Derives capability classification from the declared required and supported feature URI lists.
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

/// Runtime posture for LV2 worker thread scheduling: absent, available, required-and-available, guarded, or unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2WorkerPosture {
    /// Plugin does not use the worker extension.
    WorkerAbsent,
    /// Worker extension is available and ready.
    WorkerAvailable,
    /// Plugin requires the worker extension and it is available.
    WorkerRequiredAvailable,
    /// Worker extension is present but operating under runtime guard conditions.
    WorkerGuarded,
    /// Worker extension is required but could not be satisfied.
    WorkerUnavailable,
}

/// Runtime URID map negotiation posture for an LV2 plugin instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2UridNegotiationPosture {
    /// Plugin does not require URID mapping.
    NotRequired,
    /// URID map negotiation completed successfully.
    Negotiated,
    /// URID map is present but operating under guard conditions.
    Guarded,
    /// URID map negotiation failed or is unavailable.
    Unavailable,
}

/// Runtime patch message exchange posture for an LV2 plugin instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2PatchExchangePosture {
    /// Plugin does not use the patch extension.
    Absent,
    /// Patch message exchange is supported and active.
    Supported,
    /// Patch exchange is present but operating under guard conditions.
    Guarded,
    /// Patch exchange is required but could not be satisfied.
    Unavailable,
}

/// Overall LV2 extension negotiation state for a plugin instance: not required, fully negotiated, partially satisfied, or unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLv2ExtensionNegotiationState {
    /// No extensions require negotiation for this plugin.
    NotRequired,
    /// All required extensions were successfully negotiated.
    Negotiated,
    /// Some extensions were negotiated but optional ones were not.
    PartiallySatisfied,
    /// One or more required extensions could not be negotiated.
    Unavailable,
}

/// LV2 extension negotiation record for one plugin type: per-extension postures, sandbox counts, and overall negotiation state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionRecord {
    /// Stable unique identifier for this plugin type.
    pub plugin_type_id: String,
    /// Shorter plugin identifier.
    pub plugin_id: String,
    /// Runtime worker thread scheduling posture.
    pub worker_posture: RuntimeLv2WorkerPosture,
    /// Runtime URID map negotiation posture.
    pub urid_negotiation_posture: RuntimeLv2UridNegotiationPosture,
    /// Runtime patch message exchange posture.
    pub patch_exchange_posture: RuntimeLv2PatchExchangePosture,
    /// Overall extension negotiation state.
    pub extension_negotiation_state: RuntimeLv2ExtensionNegotiationState,
    /// Highest-severity lifecycle state across all sandboxes for this type.
    pub strongest_lifecycle_state: Option<RuntimePluginLifecycleState>,
    /// Total sandboxes for this plugin type.
    pub sandbox_count: usize,
    /// Number of sandboxes currently active or transport-active.
    pub active_sandbox_count: usize,
    /// Number of sandboxes in a faulted or quarantined state.
    pub faulted_sandbox_count: usize,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Aggregate LV2 extension snapshot across all plugin types: counts by extension posture and the full record list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2ExtensionSnapshot {
    /// Total LV2 plugin types in this snapshot.
    pub plugin_type_count: usize,
    /// Total sandbox instances across all LV2 types.
    pub sandbox_count: usize,
    /// Number of types that require the worker extension.
    pub worker_required_type_count: usize,
    /// Number of types with a guarded worker posture.
    pub worker_guarded_type_count: usize,
    /// Number of types with a negotiated URID map posture.
    pub urid_negotiated_type_count: usize,
    /// Number of types with patch extension support.
    pub patch_supported_type_count: usize,
    /// Number of types fully negotiated across all extensions.
    pub negotiated_type_count: usize,
    /// Number of types with at least one guarded extension.
    pub guarded_type_count: usize,
    /// Number of types where extension negotiation is unavailable.
    pub unavailable_type_count: usize,
    /// Per-type extension negotiation records.
    pub records: Vec<RuntimeLv2ExtensionRecord>,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Aggregate plugin capability coverage across the full catalog: format counts, I/O topology counts, and feature support counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginCapabilityCoverageSummary {
    /// Number of distinct plugin formats with discovered types.
    pub discovered_format_count: usize,
    /// Whether the catalog contains more than one plugin format.
    pub multi_format_catalog: bool,
    /// Number of types with complex I/O topology.
    pub complex_io_type_count: usize,
    /// Number of instruments with multiple output buses.
    pub multi_output_instrument_count: usize,
    /// Number of FX types with bus routing capability.
    pub bus_capable_fx_count: usize,
    /// Number of FX types with sidechain input capability.
    pub sidechain_capable_fx_count: usize,
    /// Number of instrument types.
    pub instrument_count: usize,
    /// Number of audio effect types.
    pub audio_effect_count: usize,
    /// Number of analyzer types.
    pub analyzer_count: usize,
    /// Number of utility types.
    pub utility_count: usize,
    /// Number of note effect types.
    pub note_effect_count: usize,
    /// Number of types supporting snapshot state.
    pub supports_snapshot_count: usize,
    /// Number of types supporting runtime reset.
    pub supports_reset_count: usize,
    /// Number of types supporting bypass.
    pub supports_bypass_count: usize,
    /// Number of types that expose latency information.
    pub exposes_latency_count: usize,
    /// Number of types that expose tail length information.
    pub exposes_tail_count: usize,
    /// Number of types supporting sample-accurate automation.
    pub sample_accurate_automation_count: usize,
    /// Number of types that accept MIDI input.
    pub accepts_midi_count: usize,
    /// Number of types that accept note events.
    pub accepts_note_events_count: usize,
    /// Number of types that support note expression.
    pub supports_note_expression_count: usize,
    /// Number of types that produce MIDI output.
    pub produces_midi_count: usize,
    /// Number of types that are silence-aware.
    pub silence_aware_count: usize,
    /// Number of types that require the main thread for state operations.
    pub requires_main_thread_for_state_count: usize,
    /// Number of types supporting the prepare lifecycle step.
    pub supports_prepare_count: usize,
    /// Number of types supporting the activate lifecycle step.
    pub supports_activate_count: usize,
    /// Number of types supporting reset while active.
    pub supports_reset_while_active_count: usize,
    /// Maximum complex I/O port group count across all catalog types.
    pub max_complex_io_port_group_count: usize,
    /// Maximum audio bus count across all catalog types.
    pub max_audio_bus_count: usize,
    /// Maximum parameter count across all catalog types.
    pub max_parameter_count: usize,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Full plugin discovery snapshot: scan history, discovered types, format/parity coverage, and aggregate capability coverage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginDiscoverySnapshot {
    /// Total number of scans performed.
    pub scan_count: usize,
    /// Number of scans that targeted a specific format subset.
    pub format_filtered_scan_count: usize,
    /// Total number of plugin types currently discovered.
    pub discovered_type_count: usize,
    /// Number of distinct plugin formats with discovered types.
    pub discovered_format_count: usize,
    /// Receipt from the most recent scan, if any.
    pub last_scan: Option<RuntimePluginScanReceipt>,
    /// Per-format type and capability coverage records.
    pub format_coverage: Vec<RuntimePluginFormatCoverageRecord>,
    /// Per-format platform parity and sandbox lifecycle records.
    pub parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    /// Aggregate capability coverage across all discovered types.
    pub capability_coverage: RuntimePluginCapabilityCoverageSummary,
    /// Full list of discovered plugin type records.
    pub discovered_types: Vec<RuntimePluginDiscoveredTypeRecord>,
    /// Human-readable one-line summary.
    pub summary: String,
}
