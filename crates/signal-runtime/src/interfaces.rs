//! Typed runtime-host interfaces for embedded Signal assemblies.

use std::sync::{Arc, Mutex};

use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::BlockSequenceContinuityReport;
use signal_primitives::SampleRate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    InvalidRequest,
    UnsupportedCapability,
    InvalidState,
    ResourceUnavailable,
    PluginFailure,
    HardwareFailure,
    Timeout,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub message: String,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeRequest {
    pub client_version: String,
    pub anticipative_preferred: bool,
    pub max_sample_rate_hint: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeResponse {
    pub runtime_version: String,
    pub protocol_version: u32,
    pub supports_anticipative: bool,
    pub supports_dynamic_reconfigure: bool,
    pub max_channels: u32,
    pub max_sample_rate: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfigRequest {
    pub sample_rate: SampleRate,
    pub block_size: usize,
    pub anticipative_enabled: bool,
    pub realtime_safe_mode: bool,
    pub max_graph_latency_ms: Option<u32>,
    pub max_background_load_percent: Option<u8>,
}

impl RuntimeConfigRequest {
    pub fn new(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            block_size,
            anticipative_enabled: true,
            realtime_safe_mode: false,
            max_graph_latency_ms: None,
            max_background_load_percent: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    UserRequested,
    DeviceReconfigure,
    DegradedModeRecovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartRequest {
    pub reconfigure: Option<RuntimeConfigRequest>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SafeModeRequest {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphProjection {
    pub graph_id: String,
    pub node_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleProjection {
    pub schedule_id: String,
    pub stream_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopRegion {
    pub start_samples: i64,
    pub end_samples: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportProjection {
    pub playing: bool,
    pub timeline_position_samples: i64,
    pub tempo_bpm: f64,
    pub loop_state: Option<LoopRegion>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterEvent {
    pub target: String,
    pub normalized_value: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterBatch {
    pub epoch: u64,
    pub events: Vec<ParameterEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionReceipt {
    pub accepted_epoch: u64,
    pub applied_at_block_boundary: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DegradedReason(pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultKind {
    Timeout,
    Crash,
    ProtocolViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWatchdogTrigger {
    DeadlineMisses,
    HeartbeatMisses,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchdogRestartRecord {
    pub sandbox_id: String,
    pub trigger: RuntimeWatchdogTrigger,
    pub processing_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSupervisionSnapshot {
    pub watchdog_restart_count: u32,
    pub safe_mode_enabled: bool,
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    pub last_sandbox_id: Option<String>,
    pub last_processing_epoch: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeTimelineSnapshot {
    pub next_block_sequence: u64,
    pub block_sequence_continuity: BlockSequenceContinuityReport,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationSnapshot {
    pub parameter_id: u32,
    pub value_events: usize,
    pub modulation_events: usize,
    pub gesture_begin_events: usize,
    pub gesture_end_events: usize,
    pub first_value: Option<f32>,
    pub last_value: Option<f32>,
    pub last_modulation: Option<f32>,
    pub first_epoch: Option<u64>,
    pub last_epoch: Option<u64>,
    pub segment_count: usize,
    pub segment_epochs: Vec<u64>,
    pub lease_rollovers: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeControlSnapshot {
    pub handshaken: bool,
    pub configured: bool,
    pub running: bool,
    pub handshake_count: u64,
    pub configure_count: u64,
    pub start_count: u64,
    pub stop_count: u64,
    pub restart_count: u64,
    pub last_client_version: Option<String>,
    pub last_stop_reason: Option<StopReason>,
    pub last_reconfigure: Option<RuntimeConfigRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeReadiness {
    Starting,
    Ready,
    Degraded { reasons: Vec<DegradedReason> },
    Stopped,
    Failed { fatal: RuntimeError },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveRuntimeConfig {
    pub sample_rate: SampleRate,
    pub block_size: usize,
    pub anticipative_enabled: bool,
    pub safe_mode_enabled: bool,
    pub active_output_device: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub cpu_load_percent: f32,
    pub xruns: u64,
    pub graph_latency_ms: f32,
    pub active_plugin_sandboxes: u32,
    pub backend_policy_tier: BackendPolicyTier,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEvent {
    ReadinessChanged(RuntimeReadiness),
    EffectiveConfigChanged(EffectiveRuntimeConfig),
    SupervisionChanged(RuntimeSupervisionSnapshot),
    PluginSandboxChanged {
        active_sandboxes: u32,
    },
    PluginSandboxFault {
        sandbox_id: String,
        kind: PluginFaultKind,
        detail: String,
        processing_epoch: Option<u64>,
    },
    HardwareDeviceChanged {
        device_id: Option<String>,
    },
}

pub trait RuntimeEventSink: Send {
    fn push(&mut self, event: RuntimeEvent);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFaultRecord {
    pub sandbox_id: String,
    pub kind: PluginFaultKind,
    pub detail: String,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeObservationDiagnostics {
    pub total_events: usize,
    pub supervision_updates: Vec<RuntimeSupervisionSnapshot>,
    pub plugin_faults: Vec<PluginFaultRecord>,
}

impl RuntimeObservationDiagnostics {
    pub fn supervision_update_count(&self) -> usize {
        self.supervision_updates.len()
    }

    pub fn plugin_fault_count(&self) -> usize {
        self.plugin_faults.len()
    }

    pub fn fault_detail_count_containing(&self, needle: &str) -> usize {
        self.plugin_faults
            .iter()
            .filter(|fault| fault.detail.contains(needle))
            .count()
    }

    pub fn last_supervision_update(&self) -> Option<&RuntimeSupervisionSnapshot> {
        self.supervision_updates.last()
    }

    pub fn render_compact(&self) -> String {
        let last_trigger = self
            .last_supervision_update()
            .and_then(|snapshot| snapshot.last_watchdog_trigger)
            .map(|trigger| format!("{trigger:?}"))
            .unwrap_or_else(|| "none".into());
        let last_fault = self
            .plugin_faults
            .last()
            .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
            .unwrap_or_else(|| "none".into());

        format!(
            "events={} supervision_updates={} plugin_faults={} last_watchdog={} last_fault={}",
            self.total_events,
            self.supervision_update_count(),
            self.plugin_fault_count(),
            last_trigger,
            last_fault
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeObservationReport {
    pub readiness: RuntimeReadiness,
    pub effective_config: EffectiveRuntimeConfig,
    pub control_snapshot: RuntimeControlSnapshot,
    pub diagnostics_snapshot: RuntimeDiagnosticsSnapshot,
    pub supervision_snapshot: RuntimeSupervisionSnapshot,
    pub timeline_snapshot: RuntimeTimelineSnapshot,
    pub automation_snapshot: RuntimeAutomationSnapshot,
    pub observation: RuntimeObservationDiagnostics,
}

impl RuntimeObservationReport {
    pub fn capture(runtime: &impl RuntimeObservationApi, recorder: &RuntimeEventRecorder) -> Self {
        Self {
            readiness: runtime.get_readiness(),
            effective_config: runtime.get_effective_config(),
            control_snapshot: runtime.get_control_snapshot(),
            diagnostics_snapshot: runtime.get_diagnostics_snapshot(),
            supervision_snapshot: runtime.get_supervision_snapshot(),
            timeline_snapshot: runtime.get_timeline_snapshot(),
            automation_snapshot: runtime.get_automation_snapshot(),
            observation: recorder.diagnostics(),
        }
    }

    pub fn render_compact(&self) -> String {
        let automation = (self.automation_snapshot.parameter_id != 0)
            .then(|| {
                let snapshot = &self.automation_snapshot;
                format!(
                    " automation_param={} automation_segments={} automation_first_epoch={:?} automation_last_epoch={:?} automation_lease_rollovers={}",
                    snapshot.parameter_id,
                    snapshot.segment_count,
                    snapshot.first_epoch,
                    snapshot.last_epoch,
                    snapshot.lease_rollovers
                )
            })
            .unwrap_or_default();
        format!(
            "readiness={:?} sample_rate={} block_size={} handshaken={} configured={} running={} handshakes={} configures={} starts={} stops={} restarts={} xruns={} active_sandboxes={} safe_mode={} next_block_sequence={} sequence_segments={} sequence_first_block={:?} sequence_last_block={:?}{} {}",
            self.readiness,
            self.effective_config.sample_rate.0,
            self.effective_config.block_size,
            self.control_snapshot.handshaken,
            self.control_snapshot.configured,
            self.control_snapshot.running,
            self.control_snapshot.handshake_count,
            self.control_snapshot.configure_count,
            self.control_snapshot.start_count,
            self.control_snapshot.stop_count,
            self.control_snapshot.restart_count,
            self.diagnostics_snapshot.xruns,
            self.diagnostics_snapshot.active_plugin_sandboxes,
            self.supervision_snapshot.safe_mode_enabled,
            self.timeline_snapshot.next_block_sequence,
            self.timeline_snapshot.block_sequence_continuity.segment_count(),
            self.timeline_snapshot
                .block_sequence_continuity
                .first_block_sequence(),
            self.timeline_snapshot
                .block_sequence_continuity
                .last_block_sequence(),
            automation,
            self.observation.render_compact()
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSupervisorReport {
    pub observation: RuntimeObservationReport,
    pub events: Vec<RuntimeEvent>,
}

impl RuntimeSupervisorReport {
    pub fn capture(runtime: &impl RuntimeObservationApi, recorder: &RuntimeEventRecorder) -> Self {
        Self {
            observation: RuntimeObservationReport::capture(runtime, recorder),
            events: recorder.snapshot(),
        }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn supervision_update_count(&self) -> usize {
        self.observation.observation.supervision_update_count()
    }

    pub fn plugin_fault_count(&self) -> usize {
        self.observation.observation.plugin_fault_count()
    }

    pub fn last_watchdog_trigger(&self) -> Option<RuntimeWatchdogTrigger> {
        self.observation
            .observation
            .last_supervision_update()
            .and_then(|snapshot| snapshot.last_watchdog_trigger)
    }

    pub fn render_compact(&self) -> String {
        format!(
            "{} event_stream={}",
            self.observation.render_compact(),
            self.event_count()
        )
    }

    pub fn render_multiline(&self) -> String {
        let automation = (self.observation.automation_snapshot.parameter_id != 0)
            .then(|| {
                let snapshot = &self.observation.automation_snapshot;
                format!(
                    "\nautomation_param={}\nautomation_value_events={}\nautomation_modulation_events={}\nautomation_gesture_begin_events={}\nautomation_gesture_end_events={}\nautomation_first_value={:?}\nautomation_last_value={:?}\nautomation_last_modulation={:?}\nautomation_first_epoch={:?}\nautomation_last_epoch={:?}\nautomation_segments={}\nautomation_segment_epochs={:?}\nautomation_lease_rollovers={}",
                    snapshot.parameter_id,
                    snapshot.value_events,
                    snapshot.modulation_events,
                    snapshot.gesture_begin_events,
                    snapshot.gesture_end_events,
                    snapshot.first_value,
                    snapshot.last_value,
                    snapshot.last_modulation,
                    snapshot.first_epoch,
                    snapshot.last_epoch,
                    snapshot.segment_count,
                    snapshot.segment_epochs,
                    snapshot.lease_rollovers,
                )
            })
            .unwrap_or_default();
        format!(
            "readiness={:?}\nsample_rate={}\nblock_size={}\nhandshaken={}\nconfigured={}\nrunning={}\nhandshake_count={}\nconfigure_count={}\nstart_count={}\nstop_count={}\nrestart_count={}\nlast_client_version={:?}\nlast_stop_reason={:?}\nlast_reconfigure={:?}\nxruns={}\nactive_sandboxes={}\nsafe_mode={}\nnext_block_sequence={}\nsequence_segments={}\nsequence_segment_epochs={:?}\nsequence_first_block={:?}\nsequence_last_block={:?}\nsequence_gaps={}\nsequence_lease_rollovers={}{}\nevent_stream={}\nsupervision_updates={}\nplugin_faults={}\nlast_watchdog={}\nlast_fault={}",
            self.observation.readiness,
            self.observation.effective_config.sample_rate.0,
            self.observation.effective_config.block_size,
            self.observation.control_snapshot.handshaken,
            self.observation.control_snapshot.configured,
            self.observation.control_snapshot.running,
            self.observation.control_snapshot.handshake_count,
            self.observation.control_snapshot.configure_count,
            self.observation.control_snapshot.start_count,
            self.observation.control_snapshot.stop_count,
            self.observation.control_snapshot.restart_count,
            self.observation.control_snapshot.last_client_version,
            self.observation.control_snapshot.last_stop_reason,
            self.observation.control_snapshot.last_reconfigure,
            self.observation.diagnostics_snapshot.xruns,
            self.observation.diagnostics_snapshot.active_plugin_sandboxes,
            self.observation.supervision_snapshot.safe_mode_enabled,
            self.observation.timeline_snapshot.next_block_sequence,
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .segment_count(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .segment_epochs(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .first_block_sequence(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .last_block_sequence(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .sequence_gaps,
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .lease_rollovers,
            automation,
            self.event_count(),
            self.supervision_update_count(),
            self.plugin_fault_count(),
            self.last_watchdog_trigger()
                .map(|trigger| format!("{trigger:?}"))
                .unwrap_or_else(|| "none".into()),
            self.observation
                .observation
                .plugin_faults
                .last()
                .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
                .unwrap_or_else(|| "none".into())
        )
    }

    pub fn render_json(&self) -> String {
        let timeline = &self.observation.timeline_snapshot.block_sequence_continuity;
        let last_fault = self.observation.observation.plugin_faults.last();
        let automation = &self.observation.automation_snapshot;
        let automation = if automation.parameter_id == 0 {
            "null".into()
        } else {
            json_runtime_automation_snapshot(automation)
        };
        format!(
            concat!(
                "{{",
                "\"readiness\":{},",
                "\"sample_rate\":{},",
                "\"block_size\":{},",
                "\"control\":{},",
                "\"xruns\":{},",
                "\"active_sandboxes\":{},",
                "\"safe_mode\":{},",
                "\"next_block_sequence\":{},",
                "\"sequence_segments\":{},",
                "\"sequence_segment_epochs\":{},",
                "\"sequence_first_block\":{},",
                "\"sequence_last_block\":{},",
                "\"sequence_gaps\":{},",
                "\"sequence_lease_rollovers\":{},",
                "\"event_stream\":{},",
                "\"supervision_updates\":{},",
                "\"plugin_faults\":{},",
                "\"last_watchdog\":{},",
                "\"last_fault\":{},",
                "\"automation\":{}",
                "}}"
            ),
            json_escape_string(&format!("{:?}", self.observation.readiness)),
            self.observation.effective_config.sample_rate.0,
            self.observation.effective_config.block_size,
            json_runtime_control_snapshot(&self.observation.control_snapshot),
            self.observation.diagnostics_snapshot.xruns,
            self.observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            self.observation.supervision_snapshot.safe_mode_enabled,
            self.observation.timeline_snapshot.next_block_sequence,
            timeline.segment_count(),
            json_u64_vec(&timeline.segment_epochs()),
            json_option_u64(timeline.first_block_sequence()),
            json_option_u64(timeline.last_block_sequence()),
            timeline.sequence_gaps,
            timeline.lease_rollovers,
            self.event_count(),
            self.supervision_update_count(),
            self.plugin_fault_count(),
            json_option_string(
                self.last_watchdog_trigger()
                    .map(|trigger| format!("{trigger:?}"))
                    .as_deref(),
            ),
            json_option_string(
                last_fault
                    .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
                    .as_deref(),
            ),
            automation,
        )
    }
}

fn json_escape_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn json_option_string(value: Option<&str>) -> String {
    match value {
        Some(value) => json_escape_string(value),
        None => "null".into(),
    }
}

fn json_option_u64(value: Option<u64>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn json_u64_vec(values: &[u64]) -> String {
    let joined = values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_option_f32(value: Option<f32>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn json_runtime_automation_snapshot(snapshot: &RuntimeAutomationSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"parameter_id\":{},",
            "\"value_events\":{},",
            "\"modulation_events\":{},",
            "\"gesture_begin_events\":{},",
            "\"gesture_end_events\":{},",
            "\"first_value\":{},",
            "\"last_value\":{},",
            "\"last_modulation\":{},",
            "\"first_epoch\":{},",
            "\"last_epoch\":{},",
            "\"segment_count\":{},",
            "\"segment_epochs\":{},",
            "\"lease_rollovers\":{}",
            "}}"
        ),
        snapshot.parameter_id,
        snapshot.value_events,
        snapshot.modulation_events,
        snapshot.gesture_begin_events,
        snapshot.gesture_end_events,
        json_option_f32(snapshot.first_value),
        json_option_f32(snapshot.last_value),
        json_option_f32(snapshot.last_modulation),
        json_option_u64(snapshot.first_epoch),
        json_option_u64(snapshot.last_epoch),
        snapshot.segment_count,
        json_u64_vec(&snapshot.segment_epochs),
        snapshot.lease_rollovers,
    )
}

fn json_runtime_control_snapshot(snapshot: &RuntimeControlSnapshot) -> String {
    let last_stop_reason = snapshot
        .last_stop_reason
        .map(|reason| format!("{reason:?}"));
    let last_reconfigure = snapshot.last_reconfigure.map(|request| {
        format!(
            "sample_rate={} block_size={} anticipative={} realtime_safe={}",
            request.sample_rate.0,
            request.block_size,
            request.anticipative_enabled,
            request.realtime_safe_mode
        )
    });
    format!(
        concat!(
            "{{",
            "\"handshaken\":{},",
            "\"configured\":{},",
            "\"running\":{},",
            "\"handshake_count\":{},",
            "\"configure_count\":{},",
            "\"start_count\":{},",
            "\"stop_count\":{},",
            "\"restart_count\":{},",
            "\"last_client_version\":{},",
            "\"last_stop_reason\":{},",
            "\"last_reconfigure\":{}",
            "}}"
        ),
        snapshot.handshaken,
        snapshot.configured,
        snapshot.running,
        snapshot.handshake_count,
        snapshot.configure_count,
        snapshot.start_count,
        snapshot.stop_count,
        snapshot.restart_count,
        json_option_string(snapshot.last_client_version.as_deref()),
        json_option_string(last_stop_reason.as_deref()),
        json_option_string(last_reconfigure.as_deref()),
    )
}

#[derive(Clone, Default)]
pub struct RuntimeEventRecorder {
    events: Arc<Mutex<Vec<RuntimeEvent>>>,
}

impl RuntimeEventRecorder {
    pub fn count(&self) -> usize {
        self.events
            .lock()
            .map(|events| events.len())
            .unwrap_or_default()
    }

    pub fn snapshot(&self) -> Vec<RuntimeEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    pub fn supervision_updates(&self) -> Vec<RuntimeSupervisionSnapshot> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::SupervisionChanged(snapshot) => Some(snapshot),
                _ => None,
            })
            .collect()
    }

    pub fn plugin_faults(&self) -> Vec<PluginFaultRecord> {
        self.snapshot()
            .into_iter()
            .filter_map(|event| match event {
                RuntimeEvent::PluginSandboxFault {
                    sandbox_id,
                    kind,
                    detail,
                    processing_epoch,
                } => Some(PluginFaultRecord {
                    sandbox_id,
                    kind,
                    detail,
                    processing_epoch,
                }),
                _ => None,
            })
            .collect()
    }

    pub fn diagnostics(&self) -> RuntimeObservationDiagnostics {
        RuntimeObservationDiagnostics {
            total_events: self.count(),
            supervision_updates: self.supervision_updates(),
            plugin_faults: self.plugin_faults(),
        }
    }
}

impl RuntimeEventSink for RuntimeEventRecorder {
    fn push(&mut self, event: RuntimeEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubscriptionHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginScanRequest {
    pub roots: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanHandle(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxSpec {
    pub sandbox_id: String,
    pub plugin_format: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxHandle(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendPolicyOverride {
    pub tier: BackendPolicyTier,
}

pub trait RuntimeLifecycleApi {
    fn handshake(&mut self, request: HandshakeRequest) -> Result<HandshakeResponse, RuntimeError>;
    fn configure(&mut self, request: RuntimeConfigRequest) -> Result<(), RuntimeError>;
    fn start(&mut self) -> Result<(), RuntimeError>;
    fn stop(&mut self, reason: StopReason) -> Result<(), RuntimeError>;
    fn restart(&mut self, request: RestartRequest) -> Result<(), RuntimeError>;
    fn set_safe_mode(&mut self, request: SafeModeRequest) -> Result<(), RuntimeError>;
}

pub trait RuntimeProjectionApi {
    fn apply_graph_projection(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_schedule_projection(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError>;
    fn apply_parameter_batch(&mut self, batch: ParameterBatch) -> Result<(), RuntimeError>;
    fn apply_hardware_config(&mut self, request: HardwareConfigRequest)
        -> Result<(), RuntimeError>;
}

pub trait RuntimeObservationApi {
    fn subscribe(&mut self, sink: Box<dyn RuntimeEventSink>) -> SubscriptionHandle;
    fn get_readiness(&self) -> RuntimeReadiness;
    fn get_effective_config(&self) -> EffectiveRuntimeConfig;
    fn get_control_snapshot(&self) -> RuntimeControlSnapshot;
    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot;
    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot;
    fn get_timeline_snapshot(&self) -> RuntimeTimelineSnapshot;
    fn get_automation_snapshot(&self) -> RuntimeAutomationSnapshot;
}

pub trait RuntimeSupervisorApi {
    fn start_plugin_scan(&mut self, request: PluginScanRequest)
        -> Result<ScanHandle, RuntimeError>;
    fn ensure_plugin_sandbox(
        &mut self,
        request: PluginSandboxSpec,
    ) -> Result<SandboxHandle, RuntimeError>;
    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError>;
}
