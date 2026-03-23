use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginRecallState {
    #[default]
    Unbound,
    Cold,
    Warm,
    Recovered,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginRecallPortabilityClass {
    Portable,
    Guarded,
    NativeOnly,
    ContextOnly,
    #[default]
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginPresetOrigin {
    Factory,
    User,
    Embedded,
    Document,
    #[default]
    Transient,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginPresetDescriptor {
    pub preset_id: Option<String>,
    pub label: Option<String>,
    pub origin: RuntimePluginPresetOrigin,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginInterchangeSnapshot {
    pub portability_class: RuntimePluginRecallPortabilityClass,
    pub shared_payload_available: bool,
    pub native_supplement_required: bool,
    pub preset_descriptor: Option<RuntimePluginPresetDescriptor>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAraDocumentContext {
    pub document_id: String,
    pub display_label: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAraSourceContext {
    pub source_id: String,
    pub display_label: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAraRegionContext {
    pub region_id: String,
    pub display_label: Option<String>,
    pub timeline_start_samples: Option<i64>,
    pub duration_samples: Option<u32>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAraContextSnapshot {
    pub portability_class: RuntimePluginRecallPortabilityClass,
    pub document_context: Option<RuntimePluginAraDocumentContext>,
    pub source_context: Option<RuntimePluginAraSourceContext>,
    pub region_context: Option<RuntimePluginAraRegionContext>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallPayload {
    pub sandbox_id: Option<String>,
    pub plugin_type_id: Option<String>,
    pub plugin_format: Option<PluginFormat>,
    pub lifecycle_state: Option<RuntimePluginLifecycleState>,
    pub lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    pub transport_stage: Option<PluginSandboxTransportStage>,
    pub readiness_state: Option<String>,
    pub recovery_count: u32,
    pub restart_count: u32,
    pub fault_count: u32,
    pub last_restart_intent: Option<RecoveryRestartIntent>,
    pub last_stop_reason: Option<StopReason>,
    pub last_fault_kind: Option<PluginFaultKind>,
    pub last_fault_detail: Option<String>,
    pub degraded_reasons: Vec<String>,
    pub interchange: RuntimePluginInterchangeSnapshot,
    pub ara_context: Option<RuntimePluginAraContextSnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginRecallSnapshot {
    pub state: RuntimePluginRecallState,
    pub payload: RuntimePluginRecallPayload,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginCompensationState {
    #[default]
    MissingBinding,
    PendingRender,
    Settling,
    Compensated,
    Bypassed,
    Degraded,
}
