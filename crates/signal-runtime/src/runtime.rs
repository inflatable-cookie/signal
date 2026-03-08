//! Runtime configuration and shell implementation for Signal.

use signal_graph::GraphConfig;
use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{
    AutomationContinuityReport, BlockSequenceContinuityReport, ParameterAutomationSummary,
};
use signal_primitives::SampleRate;

use crate::interfaces::{
    DegradedReason, EffectiveRuntimeConfig, GraphProjection, HandshakeRequest, HandshakeResponse,
    ParameterBatch, PluginFaultKind, ProjectionReceipt, RestartRequest, RuntimeAutomationSnapshot,
    RuntimeConfigRequest, RuntimeControlSnapshot, RuntimeDiagnosticsSnapshot, RuntimeError,
    RuntimeErrorKind, RuntimeEvent, RuntimeEventSink, RuntimeLifecycleApi, RuntimeObservationApi,
    RuntimeProjectionApi, RuntimeReadiness, RuntimeSupervisionSnapshot, RuntimeTimelineSnapshot,
    RuntimeWatchdogTrigger, SafeModeRequest, ScheduleProjection, StopReason, SubscriptionHandle,
    TransportProjection, WatchdogRestartRecord,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub sample_rate: SampleRate,
    pub graph: GraphConfig,
    pub profile: RuntimeProfile,
}

impl RuntimeConfig {
    pub fn local(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            graph: GraphConfig { block_size },
            profile: RuntimeProfile::Local,
        }
    }

    pub fn server(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            graph: GraphConfig { block_size },
            profile: RuntimeProfile::Server,
        }
    }
}

pub struct SignalRuntime {
    config: RuntimeConfig,
    readiness: RuntimeReadiness,
    safe_mode_enabled: bool,
    anticipative_enabled: bool,
    active_output_device: Option<String>,
    applied_graph: Option<GraphProjection>,
    applied_schedule: Option<ScheduleProjection>,
    applied_transport: Option<TransportProjection>,
    latest_parameter_epoch: u64,
    projection_epoch: u64,
    control: RuntimeControlSnapshot,
    timeline: RuntimeTimelineState,
    automation: RuntimeAutomationState,
    diagnostics: RuntimeDiagnosticsSnapshot,
    supervision: RuntimeSupervisionState,
    next_subscription: u64,
    sinks: Vec<Box<dyn RuntimeEventSink>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeSupervisionPolicy {
    safe_mode_restart_threshold: u32,
}

impl Default for RuntimeSupervisionPolicy {
    fn default() -> Self {
        Self {
            safe_mode_restart_threshold: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeSupervisionState {
    policy: RuntimeSupervisionPolicy,
    watchdog_restart_count: u32,
    last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    last_sandbox_id: Option<String>,
    last_processing_epoch: Option<u64>,
}

impl RuntimeSupervisionState {
    fn snapshot(&self, safe_mode_enabled: bool) -> RuntimeSupervisionSnapshot {
        RuntimeSupervisionSnapshot {
            watchdog_restart_count: self.watchdog_restart_count,
            safe_mode_enabled,
            last_watchdog_trigger: self.last_watchdog_trigger,
            last_sandbox_id: self.last_sandbox_id.clone(),
            last_processing_epoch: self.last_processing_epoch,
        }
    }

    fn record_watchdog_restart(&mut self, record: WatchdogRestartRecord) -> bool {
        self.watchdog_restart_count = self.watchdog_restart_count.saturating_add(1);
        self.last_watchdog_trigger = Some(record.trigger);
        self.last_sandbox_id = Some(record.sandbox_id);
        self.last_processing_epoch = Some(record.processing_epoch);
        self.watchdog_restart_count >= self.policy.safe_mode_restart_threshold
    }
}

impl Default for RuntimeSupervisionState {
    fn default() -> Self {
        Self {
            policy: RuntimeSupervisionPolicy::default(),
            watchdog_restart_count: 0,
            last_watchdog_trigger: None,
            last_sandbox_id: None,
            last_processing_epoch: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RuntimeTimelineState {
    next_block_sequence: u64,
    continuity: BlockSequenceContinuityReport,
}

impl RuntimeTimelineState {
    fn allocate_block_sequence(&mut self) -> u64 {
        let block_sequence = self.next_block_sequence;
        self.next_block_sequence = self.next_block_sequence.saturating_add(1);
        block_sequence
    }

    fn record_block_sequence(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) {
        self.continuity
            .record(processing_epoch, lease_id, block_sequence);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn snapshot(&self) -> RuntimeTimelineSnapshot {
        RuntimeTimelineSnapshot {
            next_block_sequence: self.next_block_sequence,
            block_sequence_continuity: self.continuity.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuntimeAutomationState {
    continuity: AutomationContinuityReport,
}

impl RuntimeAutomationState {
    fn record_summary(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        summary: ParameterAutomationSummary,
    ) {
        self.continuity.record(processing_epoch, lease_id, summary);
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn snapshot(&self) -> RuntimeAutomationSnapshot {
        let aggregate = self.continuity.aggregate();
        RuntimeAutomationSnapshot {
            parameter_id: aggregate.parameter_id,
            value_events: aggregate.value_events,
            modulation_events: aggregate.modulation_events,
            gesture_begin_events: aggregate.gesture_begin_events,
            gesture_end_events: aggregate.gesture_end_events,
            first_value: aggregate.first_value,
            last_value: aggregate.last_value,
            last_modulation: aggregate.last_modulation,
            first_epoch: self.continuity.first_epoch(),
            last_epoch: self.continuity.last_epoch(),
            segment_count: self.continuity.segment_count(),
            segment_epochs: self.continuity.segment_epochs(),
            lease_rollovers: self.continuity.lease_rollovers,
        }
    }
}

impl core::fmt::Debug for SignalRuntime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SignalRuntime")
            .field("config", &self.config)
            .field("readiness", &self.readiness)
            .field("safe_mode_enabled", &self.safe_mode_enabled)
            .field("anticipative_enabled", &self.anticipative_enabled)
            .field("active_output_device", &self.active_output_device)
            .field("applied_graph", &self.applied_graph)
            .field("applied_schedule", &self.applied_schedule)
            .field("applied_transport", &self.applied_transport)
            .field("latest_parameter_epoch", &self.latest_parameter_epoch)
            .field("projection_epoch", &self.projection_epoch)
            .field("control", &self.control)
            .field("timeline", &self.timeline)
            .field("automation", &self.automation)
            .field("diagnostics", &self.diagnostics)
            .field("supervision", &self.supervision)
            .finish()
    }
}

impl SignalRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            readiness: RuntimeReadiness::Stopped,
            safe_mode_enabled: false,
            anticipative_enabled: true,
            active_output_device: None,
            applied_graph: None,
            applied_schedule: None,
            applied_transport: None,
            latest_parameter_epoch: 0,
            projection_epoch: 0,
            control: RuntimeControlSnapshot::default(),
            timeline: RuntimeTimelineState::default(),
            automation: RuntimeAutomationState::default(),
            diagnostics: RuntimeDiagnosticsSnapshot {
                cpu_load_percent: 0.0,
                xruns: 0,
                graph_latency_ms: 0.0,
                active_plugin_sandboxes: 0,
                backend_policy_tier: BackendPolicyTier::Tier0InHost,
            },
            supervision: RuntimeSupervisionState::default(),
            next_subscription: 1,
            sinks: Vec::new(),
        }
    }

    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    pub fn set_active_output_device(&mut self, device_id: impl Into<String>) {
        self.active_output_device = Some(device_id.into());
        self.emit(RuntimeEvent::HardwareDeviceChanged {
            device_id: self.active_output_device.clone(),
        });
    }

    pub fn set_active_plugin_sandboxes(&mut self, count: u32) {
        self.diagnostics.active_plugin_sandboxes = count;
        self.emit(RuntimeEvent::PluginSandboxChanged {
            active_sandboxes: self.diagnostics.active_plugin_sandboxes,
        });
    }

    pub fn set_backend_policy_tier(&mut self, tier: BackendPolicyTier) {
        self.diagnostics.backend_policy_tier = tier;
    }

    pub fn set_cpu_load_percent(&mut self, cpu_load_percent: f32) {
        self.diagnostics.cpu_load_percent = cpu_load_percent.max(0.0);
    }

    pub fn set_graph_latency_ms(&mut self, graph_latency_ms: f32) {
        self.diagnostics.graph_latency_ms = graph_latency_ms.max(0.0);
    }

    pub fn increment_xruns(&mut self) {
        self.diagnostics.xruns = self.diagnostics.xruns.saturating_add(1);
    }

    pub fn record_plugin_sandbox_fault(
        &mut self,
        sandbox_id: impl Into<String>,
        kind: PluginFaultKind,
        detail: impl Into<String>,
        processing_epoch: Option<u64>,
    ) {
        self.emit(RuntimeEvent::PluginSandboxFault {
            sandbox_id: sandbox_id.into(),
            kind,
            detail: detail.into(),
            processing_epoch,
        });
    }

    pub fn record_watchdog_restart(
        &mut self,
        record: WatchdogRestartRecord,
    ) -> RuntimeSupervisionSnapshot {
        if self.supervision.record_watchdog_restart(record) {
            self.safe_mode_enabled = true;
        }
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        self.emit(RuntimeEvent::SupervisionChanged(
            self.get_supervision_snapshot(),
        ));
        self.get_supervision_snapshot()
    }

    pub fn projection_epoch(&self) -> u64 {
        self.projection_epoch
    }

    pub fn reset_block_timeline(&mut self) {
        self.timeline.reset();
    }

    pub fn reset_automation_tracking(&mut self) {
        self.automation.reset();
    }

    pub fn allocate_block_sequence(&mut self) -> u64 {
        self.timeline.allocate_block_sequence()
    }

    pub fn record_block_sequence(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) {
        self.timeline
            .record_block_sequence(processing_epoch, lease_id, block_sequence);
    }

    pub fn record_automation_summary(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        summary: ParameterAutomationSummary,
    ) {
        self.automation
            .record_summary(processing_epoch, lease_id, summary);
    }

    fn require_handshake(&self) -> Result<(), RuntimeError> {
        if self.control.handshaken {
            Ok(())
        } else {
            Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be handshaken before control requests",
            ))
        }
    }

    fn require_configured(&self) -> Result<(), RuntimeError> {
        if self.control.configured {
            Ok(())
        } else {
            Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before this request",
            ))
        }
    }

    fn refresh_runtime_state(&mut self) {
        match self.readiness {
            RuntimeReadiness::Failed { .. } | RuntimeReadiness::Stopped => {}
            RuntimeReadiness::Starting => {}
            RuntimeReadiness::Ready | RuntimeReadiness::Degraded { .. } => {
                self.readiness = if self.safe_mode_enabled {
                    RuntimeReadiness::Degraded {
                        reasons: vec![
                            DegradedReason("safe-mode-enabled"),
                            DegradedReason("watchdog-restart-threshold-exceeded"),
                        ],
                    }
                } else {
                    RuntimeReadiness::Ready
                };
            }
        }
    }

    fn emit(&mut self, event: RuntimeEvent) {
        for sink in &mut self.sinks {
            sink.push(event.clone());
        }
    }
}

impl RuntimeLifecycleApi for SignalRuntime {
    fn handshake(&mut self, request: HandshakeRequest) -> Result<HandshakeResponse, RuntimeError> {
        if request.client_version.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "client_version must not be empty",
            ));
        }
        if matches!(request.max_sample_rate_hint, Some(0)) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "max_sample_rate_hint must be positive when provided",
            ));
        }

        self.control.handshaken = true;
        self.control.handshake_count = self.control.handshake_count.saturating_add(1);
        self.control.last_client_version = Some(request.client_version.clone());

        Ok(HandshakeResponse {
            runtime_version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: 1,
            supports_anticipative: true,
            supports_dynamic_reconfigure: true,
            max_channels: 2048,
            max_sample_rate: request.max_sample_rate_hint.unwrap_or(192_000),
        })
    }

    fn configure(&mut self, request: RuntimeConfigRequest) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        if request.block_size == 0 || request.sample_rate.0 == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "sample_rate and block_size must be non-zero",
            ));
        }

        self.config.sample_rate = request.sample_rate;
        self.config.graph.block_size = request.block_size;
        self.anticipative_enabled = request.anticipative_enabled;
        self.safe_mode_enabled = request.realtime_safe_mode;
        self.control.configured = true;
        self.control.running = false;
        self.control.configure_count = self.control.configure_count.saturating_add(1);
        self.control.last_reconfigure = Some(request);
        self.timeline.reset();
        self.automation.reset();
        self.readiness = RuntimeReadiness::Starting;
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }

    fn start(&mut self) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        self.require_configured()?;
        if self.control.running {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime is already running",
            ));
        }

        self.readiness = RuntimeReadiness::Ready;
        self.control.running = true;
        self.control.start_count = self.control.start_count.saturating_add(1);
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        Ok(())
    }

    fn stop(&mut self, reason: StopReason) -> Result<(), RuntimeError> {
        if !self.control.running {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime is not running",
            ));
        }

        self.readiness = RuntimeReadiness::Stopped;
        self.control.running = false;
        self.control.stop_count = self.control.stop_count.saturating_add(1);
        self.control.last_stop_reason = Some(reason);
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        Ok(())
    }

    fn restart(&mut self, request: RestartRequest) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        if request.reconfigure.is_none() {
            self.require_configured()?;
        }
        if self.control.running {
            self.stop(StopReason::DeviceReconfigure)?;
        }
        if let Some(config) = request.reconfigure {
            self.configure(config)?;
        }
        self.control.restart_count = self.control.restart_count.saturating_add(1);
        self.start()
    }

    fn set_safe_mode(&mut self, request: SafeModeRequest) -> Result<(), RuntimeError> {
        self.safe_mode_enabled = request.enabled;
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }
}

impl RuntimeProjectionApi for SignalRuntime {
    fn apply_graph_projection(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        if projection.graph_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph_id must not be empty",
            ));
        }

        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.applied_graph = Some(projection);
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_schedule_projection(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        if projection.schedule_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "schedule_id must not be empty",
            ));
        }

        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.applied_schedule = Some(projection);
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError> {
        if projection.tempo_bpm <= 0.0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "tempo_bpm must be positive",
            ));
        }

        self.applied_transport = Some(projection);
        Ok(())
    }

    fn apply_parameter_batch(&mut self, batch: ParameterBatch) -> Result<(), RuntimeError> {
        if batch.epoch < self.projection_epoch {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "parameter batch epoch is stale",
            ));
        }
        self.latest_parameter_epoch = batch.epoch;
        Ok(())
    }

    fn apply_hardware_config(
        &mut self,
        request: HardwareConfigRequest,
    ) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        self.require_configured()?;
        if request.buffer_size == 0 || request.sample_rate.0 == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "hardware config sample_rate and buffer_size must be non-zero",
            ));
        }

        self.config.sample_rate = request.sample_rate;
        self.config.graph.block_size = request.buffer_size;
        self.diagnostics.backend_policy_tier = request.backend_policy;
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }
}

impl RuntimeObservationApi for SignalRuntime {
    fn subscribe(&mut self, sink: Box<dyn RuntimeEventSink>) -> SubscriptionHandle {
        let handle = SubscriptionHandle(self.next_subscription);
        self.next_subscription = self.next_subscription.saturating_add(1);
        self.sinks.push(sink);
        handle
    }

    fn get_readiness(&self) -> RuntimeReadiness {
        self.readiness.clone()
    }

    fn get_effective_config(&self) -> EffectiveRuntimeConfig {
        EffectiveRuntimeConfig {
            sample_rate: self.config.sample_rate,
            block_size: self.config.graph.block_size,
            anticipative_enabled: self.anticipative_enabled,
            safe_mode_enabled: self.safe_mode_enabled,
            active_output_device: self.active_output_device.clone(),
        }
    }

    fn get_control_snapshot(&self) -> RuntimeControlSnapshot {
        self.control.clone()
    }

    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot {
        self.diagnostics
    }

    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot {
        self.supervision.snapshot(self.safe_mode_enabled)
    }

    fn get_timeline_snapshot(&self) -> RuntimeTimelineSnapshot {
        self.timeline.snapshot()
    }

    fn get_automation_snapshot(&self) -> RuntimeAutomationSnapshot {
        self.automation.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimeConfig, RuntimeProfile, SignalRuntime};
    use crate::interfaces::{
        HandshakeRequest, RestartRequest, RuntimeConfigRequest, RuntimeEvent, RuntimeEventRecorder,
        RuntimeEventSink, RuntimeLifecycleApi, RuntimeObservationApi, RuntimeObservationReport,
        RuntimeProjectionApi, RuntimeReadiness, RuntimeSupervisorReport, RuntimeWatchdogTrigger,
        SafeModeRequest, ScheduleProjection, StopReason, TransportProjection,
        WatchdogRestartRecord,
    };
    use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
    use signal_plugin::ParameterAutomationSummary;

    #[derive(Default)]
    struct TestSink {
        events: Vec<RuntimeEvent>,
    }

    impl RuntimeEventSink for TestSink {
        fn push(&mut self, event: RuntimeEvent) {
            self.events.push(event);
        }
    }

    fn handshake_and_configure(runtime: &mut SignalRuntime) {
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();
    }

    #[test]
    fn runtime_starts_and_reports_ready() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();

        assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
        assert_eq!(runtime.config().profile, RuntimeProfile::Local);
    }

    #[test]
    fn configure_updates_effective_config() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(96_000, 256))
            .unwrap();

        let config = runtime.get_effective_config();
        assert_eq!(config.sample_rate.0, 96_000);
        assert_eq!(config.block_size, 256);
    }

    #[test]
    fn configure_resets_runtime_block_timeline() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        let first_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence(1, "lease-a", first_sequence);

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.next_block_sequence, 0);
        assert_eq!(timeline.block_sequence_continuity.segment_count(), 0);
    }

    #[test]
    fn runtime_timeline_tracks_sequences_across_leases() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let first = runtime.allocate_block_sequence();
        runtime.record_block_sequence(1, "lease-a", first);
        let second = runtime.allocate_block_sequence();
        runtime.record_block_sequence(1, "lease-a", second);
        let third = runtime.allocate_block_sequence();
        runtime.record_block_sequence(2, "lease-b", third);

        let timeline = runtime.get_timeline_snapshot();
        assert_eq!(timeline.next_block_sequence, 3);
        assert_eq!(timeline.block_sequence_continuity.segment_count(), 2);
        assert_eq!(timeline.block_sequence_continuity.lease_rollovers, 1);
        assert_eq!(
            timeline.block_sequence_continuity.first_block_sequence(),
            Some(0)
        );
        assert_eq!(
            timeline.block_sequence_continuity.last_block_sequence(),
            Some(2)
        );
    }

    #[test]
    fn configure_resets_runtime_automation_tracking() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.parameter_id, 0);
        assert_eq!(automation.segment_count, 0);
        assert_eq!(automation.first_epoch, None);
    }

    #[test]
    fn runtime_automation_tracking_rolls_across_leases() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );
        runtime.record_automation_summary(
            2,
            "lease-b",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 0,
                gesture_end_events: 1,
                first_value: Some(0.5),
                last_value: Some(0.7),
                last_modulation: Some(0.12),
            },
        );

        let automation = runtime.get_automation_snapshot();
        assert_eq!(automation.parameter_id, 4096);
        assert_eq!(automation.value_events, 4);
        assert_eq!(automation.segment_count, 2);
        assert_eq!(automation.segment_epochs, vec![1, 2]);
        assert_eq!(automation.lease_rollovers, 1);
        assert_eq!(automation.first_epoch, Some(1));
        assert_eq!(automation.last_epoch, Some(2));
    }

    #[test]
    fn handshake_requires_client_version() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let error = runtime
            .handshake(HandshakeRequest {
                client_version: String::new(),
                anticipative_preferred: true,
                max_sample_rate_hint: None,
            })
            .unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidRequest
        );
    }

    #[test]
    fn schedule_projection_advances_epoch() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let receipt = runtime
            .apply_schedule_projection(ScheduleProjection {
                schedule_id: "sched-1".into(),
                stream_count: 2,
            })
            .unwrap();

        assert_eq!(receipt.accepted_epoch, 1);
        assert!(receipt.applied_at_block_boundary);
    }

    #[test]
    fn hardware_config_updates_runtime_and_backend_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime
            .apply_hardware_config(HardwareConfigRequest::new(
                96_000,
                256,
                BackendPolicyTier::Tier1Brokered,
            ))
            .unwrap();

        let config = runtime.get_effective_config();
        assert_eq!(config.sample_rate.0, 96_000);
        assert_eq!(config.block_size, 256);
        assert_eq!(
            runtime.get_diagnostics_snapshot().backend_policy_tier,
            BackendPolicyTier::Tier1Brokered
        );
    }

    #[test]
    fn safe_mode_sets_degraded_readiness() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .unwrap();

        assert!(matches!(
            runtime.get_readiness(),
            RuntimeReadiness::Degraded { .. }
        ));
    }

    #[test]
    fn restart_reconfigures_runtime() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .restart(RestartRequest {
                reconfigure: Some(RuntimeConfigRequest::new(44_100, 128)),
            })
            .unwrap();

        assert_eq!(runtime.get_effective_config().sample_rate.0, 44_100);
        assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
    }

    #[test]
    fn transport_projection_rejects_non_positive_tempo() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let error = runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 0,
                tempo_bpm: 0.0,
                loop_state: None,
            })
            .unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidRequest
        );
    }

    #[test]
    fn runtime_emits_events_to_subscribers() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let sink = Box::new(TestSink::default());
        runtime.subscribe(sink);

        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();
        runtime.start().unwrap();
        runtime.set_active_output_device("coreaudio:default");
        runtime.set_active_plugin_sandboxes(2);

        let readiness = runtime.get_readiness();
        assert_eq!(readiness, RuntimeReadiness::Ready);
        assert_eq!(
            runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
            2
        );
    }

    #[test]
    fn runtime_records_plugin_fault_events() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime.record_plugin_sandbox_fault(
            "sandbox-a",
            crate::interfaces::PluginFaultKind::ProtocolViolation,
            "epoch mismatch",
            Some(3),
        );

        assert_eq!(
            runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
            0
        );
    }

    #[test]
    fn runtime_owns_watchdog_restart_escalation() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();

        let first = runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
            processing_epoch: 1,
        });
        assert_eq!(first.watchdog_restart_count, 1);
        assert!(!first.safe_mode_enabled);

        let second = runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "sandbox-a".into(),
            trigger: RuntimeWatchdogTrigger::DeadlineMisses,
            processing_epoch: 2,
        });
        assert_eq!(second.watchdog_restart_count, 2);
        assert!(second.safe_mode_enabled);
        assert_eq!(
            second.last_watchdog_trigger,
            Some(RuntimeWatchdogTrigger::DeadlineMisses)
        );
        assert_eq!(second.last_processing_epoch, Some(2));
        assert!(matches!(
            runtime.get_readiness(),
            RuntimeReadiness::Degraded { .. }
        ));
    }

    #[test]
    fn runtime_event_recorder_builds_reusable_observation_diagnostics() {
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SupervisionChanged(crate::interfaces::RuntimeSupervisionSnapshot {
                watchdog_restart_count: 2,
                safe_mode_enabled: true,
                last_watchdog_trigger: Some(RuntimeWatchdogTrigger::HeartbeatMisses),
                last_sandbox_id: Some("sandbox-a".into()),
                last_processing_epoch: Some(4),
            }),
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxFault {
                sandbox_id: "sandbox-a".into(),
                kind: crate::interfaces::PluginFaultKind::Timeout,
                detail: "heartbeat watchdog missed twice".into(),
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxFault {
                sandbox_id: "sandbox-a".into(),
                kind: crate::interfaces::PluginFaultKind::Timeout,
                detail: "block deadline missed twice".into(),
                processing_epoch: Some(3),
            },
        );

        let diagnostics = recorder.diagnostics();
        assert_eq!(diagnostics.total_events, 3);
        assert_eq!(diagnostics.supervision_update_count(), 1);
        assert_eq!(diagnostics.plugin_fault_count(), 2);
        assert_eq!(diagnostics.fault_detail_count_containing("watchdog"), 1);
        assert_eq!(
            diagnostics.fault_detail_count_containing("block deadline"),
            1
        );
        assert_eq!(
            diagnostics
                .last_supervision_update()
                .and_then(|snapshot| snapshot.last_processing_epoch),
            Some(4)
        );
        assert!(diagnostics.render_compact().contains("plugin_faults=2"));

        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();
        let first_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence(1, "lease-a", first_sequence);
        let second_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence(1, "lease-a", second_sequence);
        let report = RuntimeObservationReport::capture(&runtime, &recorder);
        assert!(report.render_compact().contains("readiness=Ready"));
        assert!(report.render_compact().contains("handshaken=true"));
        assert!(report.render_compact().contains("configures=1"));
        assert!(report.render_compact().contains("plugin_faults=2"));
        assert!(report.render_compact().contains("next_block_sequence=2"));
        runtime.record_automation_summary(
            1,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.4),
                last_modulation: Some(0.08),
            },
        );
        runtime.record_automation_summary(
            2,
            "lease-b",
            ParameterAutomationSummary {
                parameter_id: 4096,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 0,
                gesture_end_events: 1,
                first_value: Some(0.5),
                last_value: Some(0.7),
                last_modulation: Some(0.12),
            },
        );

        let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
        assert_eq!(supervisor.event_count(), 3);
        assert_eq!(supervisor.supervision_update_count(), 1);
        assert_eq!(supervisor.plugin_fault_count(), 2);
        assert_eq!(
            supervisor.last_watchdog_trigger(),
            Some(RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert!(supervisor.render_compact().contains("event_stream=3"));
        assert!(supervisor.render_multiline().contains("plugin_faults=2"));
        assert!(supervisor
            .render_multiline()
            .contains("sequence_segments=1"));
        assert!(supervisor
            .render_multiline()
            .contains("automation_param=4096"));
        let json = supervisor.render_json();
        assert!(json.contains("\"readiness\":\"Ready\""));
        assert!(json.contains("\"control\":{\"handshaken\":true"));
        assert!(json.contains("\"next_block_sequence\":2"));
        assert!(json.contains("\"sequence_segments\":1"));
        assert!(json.contains("\"plugin_faults\":2"));
        assert!(json.contains("\"automation\":{\"parameter_id\":4096"));
    }

    #[test]
    fn configure_requires_prior_handshake() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let error = runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidState
        );
    }

    #[test]
    fn start_requires_prior_configuration() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        let error = runtime.start().unwrap_err();

        assert_eq!(
            error.kind,
            crate::interfaces::RuntimeErrorKind::InvalidState
        );
    }

    #[test]
    fn control_snapshot_tracks_handshake_configure_and_restart_history() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .unwrap();
        runtime.start().unwrap();
        runtime
            .restart(RestartRequest {
                reconfigure: Some(RuntimeConfigRequest::new(44_100, 128)),
            })
            .unwrap();

        let control = runtime.get_control_snapshot();
        assert!(control.handshaken);
        assert!(control.configured);
        assert!(control.running);
        assert_eq!(control.handshake_count, 1);
        assert_eq!(control.configure_count, 2);
        assert_eq!(control.start_count, 2);
        assert_eq!(control.stop_count, 1);
        assert_eq!(control.restart_count, 1);
        assert_eq!(control.last_client_version.as_deref(), Some("runtime-test"));
        assert_eq!(
            control.last_stop_reason,
            Some(StopReason::DeviceReconfigure)
        );
        assert_eq!(
            control
                .last_reconfigure
                .map(|request| request.sample_rate.0),
            Some(44_100)
        );
    }
}
