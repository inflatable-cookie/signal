use super::*;

/// Recall readiness of a plugin chain stage.
///
/// - `Unbound`: no sandbox assigned; preset cannot be recalled.
/// - `Cold`: sandbox exists but state has not been transferred.
/// - `Warm`: state is present and ready to use.
/// - `Recovered`: state was recovered from a prior session.
/// - `Unavailable`: sandbox is faulted or quarantined; recall blocked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginRecallState {
    #[default]
    /// No sandbox is assigned; preset cannot be recalled.
    Unbound,
    /// Sandbox exists but state has not been transferred.
    Cold,
    /// State is present and ready to use.
    Warm,
    /// State was recovered from a prior session.
    Recovered,
    /// Sandbox is faulted or quarantined; recall is blocked.
    Unavailable,
}

/// Cross-platform portability of a plugin's recalled state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginRecallPortabilityClass {
    /// State can be transferred across platforms without modification.
    Portable,
    /// State can be transferred with additional platform-specific guards.
    Guarded,
    /// State is tied to the native platform and cannot be ported.
    NativeOnly,
    /// State is tied to the current execution context only.
    ContextOnly,
    #[default]
    /// Portability is not supported or undetermined.
    Unsupported,
}

/// Origin of a plugin preset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginPresetOrigin {
    /// Preset shipped with the plugin.
    Factory,
    /// Preset created by the user.
    User,
    /// Preset embedded in a project or clip file.
    Embedded,
    /// Preset stored within the document.
    Document,
    #[default]
    /// Preset exists only for the current session.
    Transient,
}

/// Identifies a plugin preset by optional ID, display label, and origin.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginPresetDescriptor {
    /// Stable preset identifier, if the format provides one.
    pub preset_id: Option<String>,
    /// Human-readable label for this preset.
    pub label: Option<String>,
    /// Where this preset originated.
    pub origin: RuntimePluginPresetOrigin,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Portability and state-transfer snapshot for a plugin sandbox.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginInterchangeSnapshot {
    /// Cross-platform portability classification for the recalled state.
    pub portability_class: RuntimePluginRecallPortabilityClass,
    /// Whether a shared (portable) state payload is available.
    pub shared_payload_available: bool,
    /// Whether a native platform supplement is required for full state restoration.
    pub native_supplement_required: bool,
    /// Preset descriptor for the currently loaded preset, if any.
    pub preset_descriptor: Option<RuntimePluginPresetDescriptor>,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// ARA document context associated with a plugin sandbox.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAraDocumentContext {
    /// Stable ARA document identifier.
    pub document_id: String,
    /// Human-readable display label for the document.
    pub display_label: Option<String>,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// ARA audio source context associated with a plugin sandbox.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAraSourceContext {
    /// Stable ARA audio source identifier.
    pub source_id: String,
    /// Human-readable display label for the audio source.
    pub display_label: Option<String>,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// ARA region context (timeline placement) associated with a plugin sandbox.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAraRegionContext {
    /// Stable ARA region identifier.
    pub region_id: String,
    /// Human-readable display label for the region.
    pub display_label: Option<String>,
    /// Region start position in samples on the timeline.
    pub timeline_start_samples: Option<i64>,
    /// Region duration in samples.
    pub duration_samples: Option<u32>,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Combined ARA context snapshot (portability, document, source, region).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAraContextSnapshot {
    /// Cross-platform portability classification for the ARA context.
    pub portability_class: RuntimePluginRecallPortabilityClass,
    /// ARA document context, if available.
    pub document_context: Option<RuntimePluginAraDocumentContext>,
    /// ARA audio source context, if available.
    pub source_context: Option<RuntimePluginAraSourceContext>,
    /// ARA region context, if available.
    pub region_context: Option<RuntimePluginAraRegionContext>,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Full recall payload for a plugin chain stage: sandbox identity, lifecycle
/// state, fault history, and interchange/ARA context.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallPayload {
    /// Sandbox ID bound to this stage, if any.
    pub sandbox_id: Option<String>,
    /// Plugin type identifier for the bound sandbox.
    pub plugin_type_id: Option<String>,
    /// Plugin format of the bound sandbox.
    pub plugin_format: Option<PluginFormat>,
    /// Lifecycle state of the bound sandbox.
    pub lifecycle_state: Option<RuntimePluginLifecycleState>,
    /// Lifecycle stage of the bound sandbox.
    pub lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    /// Transport stage of the bound sandbox.
    pub transport_stage: Option<PluginSandboxTransportStage>,
    /// Readiness state string from the most recent sandbox heartbeat.
    pub readiness_state: Option<String>,
    /// Number of recovery cycles for this sandbox.
    pub recovery_count: u32,
    /// Number of restart cycles for this sandbox.
    pub restart_count: u32,
    /// Number of fault events for this sandbox.
    pub fault_count: u32,
    /// Intent of the most recent recovery restart.
    pub last_restart_intent: Option<RecoveryRestartIntent>,
    /// Reason for the most recent sandbox stop.
    pub last_stop_reason: Option<StopReason>,
    /// Kind of the most recent plugin fault.
    pub last_fault_kind: Option<PluginFaultKind>,
    /// Detail message from the most recent plugin fault.
    pub last_fault_detail: Option<String>,
    /// Reasons this sandbox is currently degraded.
    pub degraded_reasons: Vec<String>,
    /// State portability and interchange snapshot.
    pub interchange: RuntimePluginInterchangeSnapshot,
    /// ARA document context, if applicable.
    pub ara_context: Option<RuntimePluginAraContextSnapshot>,
}

/// Per-stage recall snapshot combining state and full payload.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallSnapshot {
    /// Recall readiness state for this stage.
    pub state: RuntimePluginRecallState,
    /// Full recall payload for this stage.
    pub payload: RuntimePluginRecallPayload,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Latency compensation state of a plugin chain stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginCompensationState {
    #[default]
    /// No sandbox binding is present; compensation cannot proceed.
    MissingBinding,
    /// Waiting for the first render pass before applying compensation.
    PendingRender,
    /// Compensation delay is being applied incrementally.
    Settling,
    /// Latency compensation is fully active.
    Compensated,
    /// Stage is bypassed; compensation is not applied.
    Bypassed,
    /// Stage is degraded; compensation state is unreliable.
    Degraded,
}
