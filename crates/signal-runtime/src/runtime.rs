//! Runtime configuration and shell implementation for Signal.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use signal_graph::{
    synthetic_stereo_block, ExecutableGraph, GraphBlockReport, GraphConfig, GraphExecutionContext,
    GraphNodeBufferContract, GraphNodeExecutionClass, GraphNodeSpec, GraphNodeTopologyMetadata,
    GraphPreparedDispatch,
};
use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{
    AutomationContinuityReport, BlockSequenceContinuityReport, CompletionState,
    ParameterAutomationSummary,
};
use signal_primitives::{AudioBuffer, FrameCount, SampleRate};

use crate::interfaces::{
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    DegradedReason, EffectiveRuntimeConfig, GraphProjection, HandshakeRequest, HandshakeResponse,
    HeartbeatCycleStage, LeaseRolloverRecord, LingeringCleanupMode, LingeringCleanupQueueReceipt,
    LingeringCleanupTrigger, ParameterBatch, PluginBackedNodeBindingProjection, PluginFaultKind,
    PluginSandboxLifecycleStage, PluginSandboxTransportStage, ProjectionReceipt,
    RecoveryRestartIntent, RestartRequest, RuntimeAutomationSnapshot, RuntimeConfigRequest,
    RuntimeControlSnapshot, RuntimeDiagnosticsSnapshot, RuntimeEngineBlockResult,
    RuntimeEngineBlockSnapshot, RuntimeError, RuntimeErrorKind, RuntimeEvent, RuntimeEventSink,
    RuntimeLifecycleApi, RuntimeObservationApi, RuntimePreworkBacklogClass,
    RuntimePreworkCacheState, RuntimePreworkForecastMode, RuntimePreworkForecastPolicy,
    RuntimePreworkForecastProfile, RuntimePreworkForecastProfileSelection,
    RuntimePreworkForecastProfileSource, RuntimePreworkFreshnessState,
    RuntimePreworkInvalidationReason, RuntimePreworkRetirementReason,
    RuntimePreworkServicePressure, RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
    RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness, RuntimeSupervisionSnapshot,
    RuntimeTimelineSnapshot, RuntimeTransportConcurrencySnapshot, RuntimeWatchdogTrigger,
    SafeModeRequest, SandboxOperationFailureStage, ScheduleProjection, StopReason,
    SubscriptionHandle, TransportAttachIntent, TransportProjection, TransportSessionProvenance,
    TransportSessionState, WatchdogRestartRecord,
};

const PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES: u32 = 64;

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
    prework_forecast_requested_mode: RuntimePreworkForecastMode,
    prework_forecast_mode: RuntimePreworkForecastMode,
    prework_forecast_policy: Option<RuntimePreworkForecastPolicy>,
    prework_forecast_profile: Option<RuntimePreworkForecastProfileSelection>,
    prework_forecast_profile_source: Option<RuntimePreworkForecastProfileSource>,
    latest_parameter_epoch: u64,
    projection_epoch: u64,
    control: RuntimeControlSnapshot,
    timeline: RuntimeTimelineState,
    automation: RuntimeAutomationState,
    engine: RuntimeEngineState,
    transport_concurrency: RuntimeTransportConcurrencyState,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeTransportConcurrencyPolicy {
    steady_session_limit: usize,
    recovery_session_limit: usize,
}

impl Default for RuntimeTransportConcurrencyPolicy {
    fn default() -> Self {
        Self {
            steady_session_limit: 1,
            recovery_session_limit: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeTransportConcurrencySession {
    sandbox_id: String,
    lease_id: String,
    region_id: String,
    intent: TransportAttachIntent,
    provenance: TransportSessionProvenance,
    attach_sequence: u64,
    attach_processing_epoch: Option<u64>,
    state: TransportSessionState,
    backing_path: Option<String>,
    total_bytes: Option<u32>,
    cleanup_attempt_count: u32,
    last_cleanup_mode: Option<LingeringCleanupMode>,
    last_cleanup_wave: Option<u64>,
    cleanup_in_progress: bool,
    last_cleanup_epoch: Option<u64>,
    last_cleanup_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeTransportConcurrencyState {
    policy: RuntimeTransportConcurrencyPolicy,
    active_sessions: BTreeMap<(String, String, String), RuntimeTransportConcurrencySession>,
    pending_cleanup_work: VecDeque<RuntimeLingeringCleanupWorkItem>,
    peak_attached_sessions: usize,
    peak_recovery_overlap_sessions: usize,
    peak_lingering_sessions: usize,
    next_attach_sequence: u64,
    next_cleanup_work_id: u64,
    next_cleanup_epoch: u64,
    next_cleanup_wave_by_sandbox: BTreeMap<String, u64>,
    last_admitted_sandbox_id: Option<String>,
    last_rejected_sandbox_id: Option<String>,
    last_rejection_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeLingeringCleanupWorkItem {
    work_id: u64,
    cleanup_epoch: u64,
    cleanup_wave: u64,
    sandbox_id: String,
    mode: LingeringCleanupMode,
    trigger: LingeringCleanupTrigger,
    retry_count: u32,
    processing_epoch: u64,
    ready_at_processing_epoch: u64,
    exclude_lease_id: Option<String>,
    exclude_region_id: Option<String>,
}

impl RuntimeTransportConcurrencyState {
    fn active_session_view(
        session: &RuntimeTransportConcurrencySession,
    ) -> crate::interfaces::ActiveTransportConcurrencySession {
        crate::interfaces::ActiveTransportConcurrencySession {
            sandbox_id: session.sandbox_id.clone(),
            lease_id: session.lease_id.clone(),
            region_id: session.region_id.clone(),
            intent: session.intent,
            provenance: session.provenance,
            attach_sequence: session.attach_sequence,
            attach_processing_epoch: session.attach_processing_epoch,
            state: session.state,
            backing_path: session.backing_path.clone(),
            total_bytes: session.total_bytes,
            cleanup_attempt_count: session.cleanup_attempt_count,
            last_cleanup_mode: session.last_cleanup_mode,
            last_cleanup_wave: session.last_cleanup_wave,
            cleanup_in_progress: session.cleanup_in_progress,
            last_cleanup_epoch: session.last_cleanup_epoch,
            last_cleanup_error: session.last_cleanup_error.clone(),
        }
    }

    fn next_cleanup_wave_for_sandbox(&mut self, sandbox_id: &str) -> u64 {
        let next = self
            .next_cleanup_wave_by_sandbox
            .entry(sandbox_id.to_string())
            .or_insert(1);
        let cleanup_wave = *next;
        *next = next.saturating_add(1);
        cleanup_wave
    }

    fn has_lingering_candidates(
        &self,
        sandbox_id: &str,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> bool {
        self.active_sessions.values().any(|session| {
            session.sandbox_id == sandbox_id
                && matches!(
                    session.state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
                && !matches!(
                    (exclude_lease_id, exclude_region_id),
                    (Some(exclude_lease_id), Some(exclude_region_id))
                        if session.lease_id == exclude_lease_id
                            && session.region_id == exclude_region_id
                )
        })
    }

    fn enqueue_cleanup_work(
        &mut self,
        sandbox_id: &str,
        mode: LingeringCleanupMode,
        trigger: LingeringCleanupTrigger,
        retry_count: u32,
        processing_epoch: u64,
        cleanup_wave: Option<u64>,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> Option<LingeringCleanupQueueReceipt> {
        if !self.has_lingering_candidates(sandbox_id, exclude_lease_id, exclude_region_id) {
            return None;
        }

        let work_id = self.next_cleanup_work_id;
        self.next_cleanup_work_id = self.next_cleanup_work_id.saturating_add(1);
        let cleanup_epoch = self.next_cleanup_epoch;
        self.next_cleanup_epoch = self.next_cleanup_epoch.saturating_add(1);
        let cleanup_wave =
            cleanup_wave.unwrap_or_else(|| self.next_cleanup_wave_for_sandbox(sandbox_id));
        let backoff = match trigger {
            LingeringCleanupTrigger::DeferredRetry => retry_count.max(1) as u64,
            LingeringCleanupTrigger::RecoveryPreAttach
            | LingeringCleanupTrigger::PostStartReconciliation => 0,
        };
        self.pending_cleanup_work
            .push_back(RuntimeLingeringCleanupWorkItem {
                work_id,
                cleanup_epoch,
                cleanup_wave,
                sandbox_id: sandbox_id.to_string(),
                mode,
                trigger,
                retry_count,
                processing_epoch,
                ready_at_processing_epoch: processing_epoch.saturating_add(backoff),
                exclude_lease_id: exclude_lease_id.map(ToOwned::to_owned),
                exclude_region_id: exclude_region_id.map(ToOwned::to_owned),
            });
        Some(LingeringCleanupQueueReceipt {
            work_id,
            cleanup_epoch,
            cleanup_wave,
        })
    }

    fn cleanup_attempt_count(&self, sandbox_id: &str, lease_id: &str, region_id: &str) -> u32 {
        self.active_sessions
            .get(&(
                sandbox_id.to_string(),
                lease_id.to_string(),
                region_id.to_string(),
            ))
            .map(|session| session.cleanup_attempt_count)
            .unwrap_or(0)
    }

    fn cleanup_wave_for_session(
        &self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> Option<u64> {
        self.active_sessions
            .get(&(
                sandbox_id.to_string(),
                lease_id.to_string(),
                region_id.to_string(),
            ))
            .and_then(|session| session.last_cleanup_wave)
    }

    fn pending_deferred_retry_work_count(&self) -> usize {
        self.pending_cleanup_work
            .iter()
            .filter(|item| item.trigger == LingeringCleanupTrigger::DeferredRetry)
            .count()
    }

    fn oldest_pending_cleanup_ready_epoch(&self) -> Option<u64> {
        self.pending_cleanup_work
            .iter()
            .map(|item| item.ready_at_processing_epoch)
            .min()
    }

    fn pending_cleanup_waves(&self) -> Vec<crate::interfaces::PendingLingeringCleanupWaveSummary> {
        let mut by_wave: BTreeMap<
            (String, u64),
            crate::interfaces::PendingLingeringCleanupWaveSummary,
        > = BTreeMap::new();
        for item in &self.pending_cleanup_work {
            let key = (item.sandbox_id.clone(), item.cleanup_wave);
            let entry = by_wave.entry(key).or_insert_with(|| {
                crate::interfaces::PendingLingeringCleanupWaveSummary {
                    sandbox_id: item.sandbox_id.clone(),
                    cleanup_wave: item.cleanup_wave,
                    mode: item.mode,
                    first_trigger: item.trigger,
                    latest_trigger: item.trigger,
                    pending_work_items: 0,
                    deferred_retry_work_items: 0,
                    first_cleanup_epoch: item.cleanup_epoch,
                    latest_cleanup_epoch: item.cleanup_epoch,
                    first_processing_epoch: item.processing_epoch,
                    latest_processing_epoch: item.processing_epoch,
                    oldest_ready_at_processing_epoch: item.ready_at_processing_epoch,
                    newest_ready_at_processing_epoch: item.ready_at_processing_epoch,
                }
            });
            entry.latest_trigger = item.trigger;
            entry.pending_work_items = entry.pending_work_items.saturating_add(1);
            if item.trigger == LingeringCleanupTrigger::DeferredRetry {
                entry.deferred_retry_work_items = entry.deferred_retry_work_items.saturating_add(1);
            }
            entry.first_cleanup_epoch = entry.first_cleanup_epoch.min(item.cleanup_epoch);
            entry.latest_cleanup_epoch = entry.latest_cleanup_epoch.max(item.cleanup_epoch);
            entry.first_processing_epoch = entry.first_processing_epoch.min(item.processing_epoch);
            entry.latest_processing_epoch =
                entry.latest_processing_epoch.max(item.processing_epoch);
            entry.oldest_ready_at_processing_epoch = entry
                .oldest_ready_at_processing_epoch
                .min(item.ready_at_processing_epoch);
            entry.newest_ready_at_processing_epoch = entry
                .newest_ready_at_processing_epoch
                .max(item.ready_at_processing_epoch);
        }
        by_wave.into_values().collect()
    }

    fn dequeue_cleanup_work_for_sandbox(
        &mut self,
        sandbox_id: &str,
        current_processing_epoch: u64,
    ) -> Option<crate::interfaces::LingeringCleanupPlan> {
        let position = self.pending_cleanup_work.iter().position(|item| {
            item.sandbox_id == sandbox_id
                && item.ready_at_processing_epoch <= current_processing_epoch
        })?;
        let work = self.pending_cleanup_work.remove(position)?;
        let candidates = self.lingering_cleanup_candidates(
            work.sandbox_id.as_str(),
            work.exclude_lease_id.as_deref(),
            work.exclude_region_id.as_deref(),
            work.mode,
            work.processing_epoch,
            work.cleanup_wave,
        );
        if candidates.is_empty() {
            return None;
        }
        Some(crate::interfaces::LingeringCleanupPlan {
            work_id: work.work_id,
            cleanup_epoch: work.cleanup_epoch,
            cleanup_wave: work.cleanup_wave,
            sandbox_id: work.sandbox_id,
            mode: work.mode,
            trigger: work.trigger,
            retry_count: work.retry_count,
            processing_epoch: work.processing_epoch,
            ready_at_processing_epoch: work.ready_at_processing_epoch,
            exclude_lease_id: work.exclude_lease_id,
            exclude_region_id: work.exclude_region_id,
            candidates,
        })
    }

    fn lingering_cleanup_candidates(
        &mut self,
        sandbox_id: &str,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        cleanup_wave: u64,
    ) -> Vec<crate::interfaces::ActiveTransportConcurrencySession> {
        let mut session_keys: Vec<_> = self
            .active_sessions
            .iter()
            .filter(|(_, session)| {
                session.sandbox_id == sandbox_id
                    && matches!(
                        session.state,
                        TransportSessionState::DetachRequested
                            | TransportSessionState::DetachFaulted
                    )
                    && !matches!(
                        (exclude_lease_id, exclude_region_id),
                        (Some(exclude_lease_id), Some(exclude_region_id))
                            if session.lease_id == exclude_lease_id
                                && session.region_id == exclude_region_id
                    )
            })
            .map(|(key, _)| key.clone())
            .collect();

        session_keys.sort_by(|left, right| {
            let left = self
                .active_sessions
                .get(left)
                .expect("missing left session");
            let right = self
                .active_sessions
                .get(right)
                .expect("missing right session");
            let left_key = (
                match left.provenance {
                    TransportSessionProvenance::SteadyOrigin => 0_u8,
                    TransportSessionProvenance::RecoveryReplacement => 1_u8,
                },
                left.attach_sequence,
                match left.state {
                    TransportSessionState::DetachRequested => 0_u8,
                    TransportSessionState::DetachFaulted => 1_u8,
                    _ => 2_u8,
                },
                left.lease_id.as_str(),
                left.region_id.as_str(),
            );
            let right_key = (
                match right.provenance {
                    TransportSessionProvenance::SteadyOrigin => 0_u8,
                    TransportSessionProvenance::RecoveryReplacement => 1_u8,
                },
                right.attach_sequence,
                match right.state {
                    TransportSessionState::DetachRequested => 0_u8,
                    TransportSessionState::DetachFaulted => 1_u8,
                    _ => 2_u8,
                },
                right.lease_id.as_str(),
                right.region_id.as_str(),
            );
            left_key.cmp(&right_key)
        });

        let mut sessions = Vec::with_capacity(session_keys.len());
        for key in session_keys {
            if let Some(session) = self.active_sessions.get_mut(&key) {
                session.cleanup_attempt_count = session.cleanup_attempt_count.saturating_add(1);
                session.last_cleanup_mode = Some(mode);
                session.last_cleanup_wave = Some(cleanup_wave);
                session.cleanup_in_progress = true;
                session.last_cleanup_epoch = Some(processing_epoch);
                session.last_cleanup_error = None;
                sessions.push(Self::active_session_view(session));
            }
        }

        sessions
    }

    fn steady_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.intent == TransportAttachIntent::SteadyState)
            .count()
    }

    fn recovery_overlap_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.intent == TransportAttachIntent::RecoveryOverlap)
            .count()
    }

    fn lingering_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| {
                matches!(
                    session.state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
            })
            .count()
    }

    fn detach_requested_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.state == TransportSessionState::DetachRequested)
            .count()
    }

    fn detach_faulted_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.state == TransportSessionState::DetachFaulted)
            .count()
    }

    fn recovery_overlap_limit(&self) -> usize {
        self.policy
            .recovery_session_limit
            .saturating_sub(self.policy.steady_session_limit)
            .max(1)
    }

    fn lingering_reason_suffix(&self, intent: TransportAttachIntent) -> String {
        let lingering = self
            .active_sessions
            .values()
            .filter(|session| {
                session.intent == intent
                    && matches!(
                        session.state,
                        TransportSessionState::DetachRequested
                            | TransportSessionState::DetachFaulted
                    )
            })
            .count();
        if lingering == 0 {
            String::new()
        } else {
            format!(" ({lingering} lingering session(s) pending detach)")
        }
    }

    fn snapshot(&self) -> RuntimeTransportConcurrencySnapshot {
        RuntimeTransportConcurrencySnapshot {
            steady_session_limit: self.policy.steady_session_limit,
            recovery_session_limit: self.policy.recovery_session_limit,
            current_attached_sessions: self.active_sessions.len(),
            peak_attached_sessions: self.peak_attached_sessions,
            current_recovery_overlap_sessions: self.recovery_overlap_session_count(),
            peak_recovery_overlap_sessions: self.peak_recovery_overlap_sessions,
            current_lingering_sessions: self.lingering_session_count(),
            peak_lingering_sessions: self.peak_lingering_sessions,
            current_detach_requested_sessions: self.detach_requested_session_count(),
            current_detach_faulted_sessions: self.detach_faulted_session_count(),
            pending_cleanup_work_items: self.pending_cleanup_work.len(),
            pending_deferred_retry_work_items: self.pending_deferred_retry_work_count(),
            next_cleanup_epoch: self.next_cleanup_epoch,
            oldest_pending_cleanup_ready_epoch: self.oldest_pending_cleanup_ready_epoch(),
            pending_cleanup_waves: self.pending_cleanup_waves(),
            active_sessions: self
                .active_sessions
                .values()
                .map(Self::active_session_view)
                .collect(),
            last_admitted_sandbox_id: self.last_admitted_sandbox_id.clone(),
            last_rejected_sandbox_id: self.last_rejected_sandbox_id.clone(),
            last_rejection_reason: self.last_rejection_reason.clone(),
        }
    }

    fn begin_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        provenance: TransportSessionProvenance,
        attach_processing_epoch: Option<u64>,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let key = (
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        );
        if self.active_sessions.contains_key(&key) {
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some("transport session is already attached".to_string());
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "transport session is already attached",
            ));
        }

        let steady_sessions = self.steady_session_count();
        let recovery_sessions = self.recovery_overlap_session_count();

        if matches!(intent, TransportAttachIntent::SteadyState)
            && steady_sessions >= self.policy.steady_session_limit
        {
            let reason = format!(
                "steady-state transport session limit {} is already attached{}",
                self.policy.steady_session_limit,
                self.lingering_reason_suffix(TransportAttachIntent::SteadyState)
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        if matches!(intent, TransportAttachIntent::RecoveryOverlap)
            && recovery_sessions >= self.recovery_overlap_limit()
        {
            let reason = format!(
                "recovery overlap session limit {} is already attached{}",
                self.recovery_overlap_limit(),
                self.lingering_reason_suffix(TransportAttachIntent::RecoveryOverlap)
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        let limit = match intent {
            TransportAttachIntent::SteadyState => self.policy.steady_session_limit,
            TransportAttachIntent::RecoveryOverlap => self.policy.recovery_session_limit,
        };
        if self.active_sessions.len() >= limit {
            let reason = format!(
                "transport session admission exceeds {:?} limit {}",
                intent, limit
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        self.active_sessions.insert(
            key,
            RuntimeTransportConcurrencySession {
                sandbox_id: sandbox_id.to_string(),
                lease_id: lease_id.to_string(),
                region_id: region_id.to_string(),
                intent,
                provenance,
                attach_sequence: self.next_attach_sequence,
                attach_processing_epoch,
                state: TransportSessionState::AttachActive,
                backing_path,
                total_bytes,
                cleanup_attempt_count: 0,
                last_cleanup_mode: None,
                last_cleanup_wave: None,
                cleanup_in_progress: false,
                last_cleanup_epoch: None,
                last_cleanup_error: None,
            },
        );
        self.next_attach_sequence = self.next_attach_sequence.saturating_add(1);
        self.peak_attached_sessions = self.peak_attached_sessions.max(self.active_sessions.len());
        let recovery_sessions = self.recovery_overlap_session_count();
        self.peak_recovery_overlap_sessions =
            self.peak_recovery_overlap_sessions.max(recovery_sessions);
        let lingering_sessions = self.lingering_session_count();
        self.peak_lingering_sessions = self.peak_lingering_sessions.max(lingering_sessions);
        self.last_admitted_sandbox_id = Some(sandbox_id.to_string());
        self.last_rejected_sandbox_id = None;
        self.last_rejection_reason = None;
        Ok(self.snapshot())
    }

    fn mark_session_state(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        state: TransportSessionState,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.state = state;
        }
        let lingering_sessions = self.lingering_session_count();
        self.peak_lingering_sessions = self.peak_lingering_sessions.max(lingering_sessions);
        self.snapshot()
    }

    fn record_cleanup_failure(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        error: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.last_cleanup_mode = Some(mode);
            session.cleanup_in_progress = false;
            session.last_cleanup_epoch = Some(processing_epoch);
            session.last_cleanup_error = Some(error.to_string());
        }
        self.snapshot()
    }

    fn clear_cleanup_in_progress(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.cleanup_in_progress = false;
            session.last_cleanup_error = None;
        }
        self.snapshot()
    }

    fn end_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.active_sessions.remove(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        ));
        self.snapshot()
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for RuntimeTransportConcurrencyState {
    fn default() -> Self {
        Self {
            policy: RuntimeTransportConcurrencyPolicy::default(),
            active_sessions: BTreeMap::new(),
            pending_cleanup_work: VecDeque::new(),
            peak_attached_sessions: 0,
            peak_recovery_overlap_sessions: 0,
            peak_lingering_sessions: 0,
            next_attach_sequence: 1,
            next_cleanup_work_id: 1,
            next_cleanup_epoch: 1,
            next_cleanup_wave_by_sandbox: BTreeMap::new(),
            last_admitted_sandbox_id: None,
            last_rejected_sandbox_id: None,
            last_rejection_reason: None,
        }
    }
}

fn transport_session_provenance(intent: TransportAttachIntent) -> TransportSessionProvenance {
    match intent {
        TransportAttachIntent::SteadyState => TransportSessionProvenance::SteadyOrigin,
        TransportAttachIntent::RecoveryOverlap => TransportSessionProvenance::RecoveryReplacement,
    }
}

fn hash_audio_buffer(buffer: &AudioBuffer) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for sample in buffer.samples() {
        hash ^= u64::from(sample.to_bits());
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= buffer.frames().0 as u64;
    hash = hash.wrapping_mul(1099511628211);
    hash ^= buffer.channel_count().0 as u64;
    hash
}

fn peak_abs(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
}

const PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW: u64 = 2;
const PREWORK_QUEUE_CAPACITY: usize = 3;

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
        sandbox_id: &str,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) -> Option<LeaseRolloverRecord> {
        let lease_id = lease_id.into();
        let previous = self.continuity.segments.last().cloned();
        self.continuity
            .record(processing_epoch, lease_id.clone(), block_sequence);
        previous.and_then(|segment| {
            (segment.lease_id != lease_id).then(|| LeaseRolloverRecord {
                sandbox_id: sandbox_id.to_string(),
                previous_lease_id: segment.lease_id,
                lease_id,
                processing_epoch,
                first_block_sequence: block_sequence,
            })
        })
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

#[derive(Clone, Debug, Default, PartialEq)]
struct RuntimeEngineState {
    graph: Option<ExecutableGraph>,
    snapshot: RuntimeEngineBlockSnapshot,
    plugin_node_bindings: HashMap<String, String>,
    prework_queue: VecDeque<RuntimeEnginePreworkCache>,
    pending_prework_targets: VecDeque<RuntimePendingPreworkTarget>,
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimeEnginePreworkCache {
    graph_id: String,
    projection_epoch: u64,
    parameter_epoch: u64,
    transport: TransportProjection,
    block_size: usize,
    frame_count: usize,
    channel_count: usize,
    input_signature: u64,
    prepared: GraphPreparedDispatch,
    valid_until_processing_epoch: u64,
    valid_until_block_sequence: u64,
    source_processing_epoch: u64,
    source_block_sequence: u64,
    admitted_from_block_sequence: u64,
    consumption_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimePendingPreworkTarget {
    target_block_sequence: u64,
    admitted_from_block_sequence: u64,
    buffer: AudioBuffer,
    input_signature: u64,
    backlog_class: RuntimePreworkBacklogClass,
    parameter_epoch_override: Option<u64>,
    transport_override: Option<TransportProjection>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RuntimePluginBackedBindingSummary {
    bound_sandbox_ids: Vec<String>,
    active_bound_sandboxes: usize,
    degraded_bound_sandboxes: usize,
    missing_bound_sandboxes: usize,
}

impl RuntimeEngineState {
    fn classify_prework_service_semantic_policy(
        graph: &ExecutableGraph,
        anticipative_enabled: bool,
        active_plugin_sandboxes: u32,
    ) -> RuntimePreworkServiceSemanticPolicy {
        if !anticipative_enabled {
            return RuntimePreworkServiceSemanticPolicy::Balanced;
        }

        let planning = graph.planning_summary(anticipative_enabled);
        if planning.anticipative_eligible_node_count == 0 {
            return RuntimePreworkServiceSemanticPolicy::Balanced;
        }

        if graph.plugin_backed_node_count() > 0 && active_plugin_sandboxes > 0 {
            return RuntimePreworkServiceSemanticPolicy::PluginConstrained;
        }

        if graph.total_latency_samples() >= PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES
            || graph.max_node_latency_samples() >= PREWORK_LATENCY_FOCUSED_THRESHOLD_SAMPLES
        {
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        } else {
            RuntimePreworkServiceSemanticPolicy::Balanced
        }
    }

    fn classify_prework_backlog_class(
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
    ) -> RuntimePreworkBacklogClass {
        match target_block_sequence.saturating_sub(admitted_from_block_sequence) {
            0 | 1 => RuntimePreworkBacklogClass::Immediate,
            2 => RuntimePreworkBacklogClass::NearTerm,
            _ => RuntimePreworkBacklogClass::Deferred,
        }
    }

    fn set_prework_service_pressure(&mut self, pressure: RuntimePreworkServicePressure) {
        self.snapshot.prework_service_pressure = pressure;
    }

    fn set_prework_service_plugin_state(
        &mut self,
        active_plugin_sandboxes: u32,
        bound_plugin_sandboxes: usize,
        active_bound_plugin_sandboxes: usize,
        degraded_bound_plugin_sandboxes: usize,
        missing_bound_plugin_sandboxes: usize,
        plugin_gate_active: bool,
    ) {
        self.snapshot.prework_service_active_plugin_sandboxes = active_plugin_sandboxes;
        self.snapshot.prework_service_bound_plugin_sandboxes = bound_plugin_sandboxes;
        self.snapshot.prework_service_active_bound_plugin_sandboxes = active_bound_plugin_sandboxes;
        self.snapshot
            .prework_service_degraded_bound_plugin_sandboxes = degraded_bound_plugin_sandboxes;
        self.snapshot.prework_service_missing_bound_plugin_sandboxes =
            missing_bound_plugin_sandboxes;
        self.snapshot.prework_service_plugin_gate_active = plugin_gate_active;
    }

    fn transition_prework_service_state(
        &mut self,
        state: RuntimePreworkServiceState,
        processing_epoch: Option<u64>,
    ) {
        let previous = self.snapshot.prework_service_state;
        if previous == state {
            return;
        }
        if state == RuntimePreworkServiceState::Paused {
            self.snapshot.prework_service_pause_count =
                self.snapshot.prework_service_pause_count.saturating_add(1);
        }
        if previous == RuntimePreworkServiceState::Paused
            && state == RuntimePreworkServiceState::Servicing
        {
            self.snapshot.prework_service_resume_count =
                self.snapshot.prework_service_resume_count.saturating_add(1);
        }
        if state == RuntimePreworkServiceState::Starved {
            self.snapshot.prework_service_starvation_count = self
                .snapshot
                .prework_service_starvation_count
                .saturating_add(1);
        }
        self.snapshot.prework_service_state = state;
        if let Some(processing_epoch) = processing_epoch {
            self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        }
    }

    fn update_prework_queue_snapshot(
        &mut self,
        current_block_sequence: Option<u64>,
        preserve_invalidated: bool,
    ) {
        self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
        self.snapshot.prework_cache_queue_depth = self.prework_queue.len();
        self.snapshot.prework_cache_peak_queue_depth = self
            .snapshot
            .prework_cache_peak_queue_depth
            .max(self.prework_queue.len());
        self.snapshot.prework_pending_target_count = self.pending_prework_targets.len();
        self.snapshot.prework_pending_immediate_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::Immediate)
            .count();
        self.snapshot.prework_pending_near_term_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::NearTerm)
            .count();
        self.snapshot.prework_pending_deferred_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::Deferred)
            .count();
        self.snapshot.prework_next_pending_target_block_sequence = self
            .pending_prework_targets
            .iter()
            .map(|target| target.target_block_sequence)
            .min();
        let mut target_block_sequences = self
            .prework_queue
            .iter()
            .map(|cache| cache.source_block_sequence)
            .chain(
                self.pending_prework_targets
                    .iter()
                    .map(|target| target.target_block_sequence),
            )
            .collect::<Vec<_>>();
        target_block_sequences.sort_unstable();
        target_block_sequences.dedup();
        self.snapshot.prework_cache_window_target_count = target_block_sequences.len();
        self.snapshot.prework_cache_window_target_block_sequences = target_block_sequences;

        let latest = self.prework_queue.back();
        self.snapshot.prework_cache_freshness_state =
            self.prework_freshness_state(latest, current_block_sequence);
        self.snapshot.prework_cache_remaining_valid_blocks = latest.map(|cache| {
            cache
                .valid_until_block_sequence
                .saturating_sub(current_block_sequence.unwrap_or(cache.source_block_sequence))
        });
        self.snapshot.prework_cache_valid_until_processing_epoch =
            latest.map(|cache| cache.valid_until_processing_epoch);
        self.snapshot.prework_cache_valid_until_block_sequence =
            latest.map(|cache| cache.valid_until_block_sequence);

        if latest.is_none() && !preserve_invalidated {
            self.snapshot.prework_cache_state = if self.snapshot.prework_cache_enabled {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
        }
    }

    fn record_prework_service_cycle(
        &mut self,
        processing_epoch: u64,
        cycle_count: usize,
        budget_per_cycle: usize,
        prepared_targets: usize,
    ) {
        self.snapshot.prework_service_cycle_count = self
            .snapshot
            .prework_service_cycle_count
            .saturating_add(cycle_count as u64);
        self.snapshot.prework_service_prepared_targets = self
            .snapshot
            .prework_service_prepared_targets
            .saturating_add(prepared_targets as u64);
        self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        self.snapshot.last_prework_service_cycle_count = cycle_count;
        self.snapshot.last_prework_service_budget_per_cycle = Some(budget_per_cycle);
        self.snapshot.last_prework_service_prepared_targets = prepared_targets;
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    fn record_prework_service_request(
        &mut self,
        requested_cycles: usize,
        effective_cycles: usize,
        requested_budget_per_cycle: usize,
        effective_budget_per_cycle: usize,
    ) {
        self.snapshot.last_prework_service_requested_cycles = requested_cycles;
        self.snapshot.last_prework_service_effective_cycles = effective_cycles;
        self.snapshot.last_prework_service_budget_per_cycle = Some(requested_budget_per_cycle);
        self.snapshot
            .last_prework_service_effective_budget_per_cycle = Some(effective_budget_per_cycle);
        if effective_cycles < requested_cycles
            || effective_budget_per_cycle < requested_budget_per_cycle
        {
            self.snapshot.prework_service_throttle_count = self
                .snapshot
                .prework_service_throttle_count
                .saturating_add(1);
        }
    }

    fn record_prework_service_yield(
        &mut self,
        processing_epoch: u64,
        requested_cycles: usize,
        requested_budget_per_cycle: usize,
    ) {
        self.snapshot.prework_service_yield_count =
            self.snapshot.prework_service_yield_count.saturating_add(1);
        self.snapshot.last_prework_service_processing_epoch = Some(processing_epoch);
        self.snapshot.last_prework_service_requested_cycles = requested_cycles;
        self.snapshot.last_prework_service_effective_cycles = 0;
        self.snapshot.last_prework_service_cycle_count = 0;
        self.snapshot.last_prework_service_budget_per_cycle = Some(requested_budget_per_cycle);
        self.snapshot
            .last_prework_service_effective_budget_per_cycle = Some(0);
        self.snapshot.last_prework_service_prepared_targets = 0;
    }

    fn record_last_serviced_pending_target(&mut self, target: &RuntimePendingPreworkTarget) {
        self.snapshot.last_prework_serviced_target_block_sequence =
            Some(target.target_block_sequence);
        self.snapshot.last_prework_serviced_backlog_class = Some(target.backlog_class);
    }

    fn retire_prework_entry(
        &mut self,
        cache: RuntimeEnginePreworkCache,
        reason: RuntimePreworkInvalidationReason,
    ) {
        self.snapshot.prework_cache_invalidation_count = self
            .snapshot
            .prework_cache_invalidation_count
            .saturating_add(1);
        self.snapshot.last_prework_invalidation_reason = Some(reason);
        let retirement_reason = self.retirement_reason_from_invalidation(reason);
        let retired_unconsumed = cache.consumption_count == 0;
        self.snapshot.prework_cache_retirement_count = self
            .snapshot
            .prework_cache_retirement_count
            .saturating_add(1);
        if retired_unconsumed {
            self.snapshot.prework_cache_unconsumed_retirement_count = self
                .snapshot
                .prework_cache_unconsumed_retirement_count
                .saturating_add(1);
        } else {
            self.snapshot.prework_cache_consumed_retirement_count = self
                .snapshot
                .prework_cache_consumed_retirement_count
                .saturating_add(1);
        }
        self.snapshot.last_prework_retirement_reason = Some(retirement_reason);
        self.snapshot.last_prework_retired_unconsumed = Some(retired_unconsumed);
        self.snapshot.last_prework_retirement_processing_epoch =
            Some(cache.source_processing_epoch);
        self.snapshot.last_prework_retirement_block_sequence = Some(cache.source_block_sequence);
        self.snapshot.prework_cache_state = RuntimePreworkCacheState::Invalidated;
    }

    fn retire_prework_entries_matching(
        &mut self,
        mut should_retire: impl FnMut(&RuntimeEnginePreworkCache) -> bool,
        reason: RuntimePreworkInvalidationReason,
    ) {
        let mut index = 0;
        while index < self.prework_queue.len() {
            if should_retire(&self.prework_queue[index]) {
                let cache = self.prework_queue.remove(index).expect("queue index valid");
                self.retire_prework_entry(cache, reason);
            } else {
                index += 1;
            }
        }
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    fn pending_target_matches(
        pending: &RuntimePendingPreworkTarget,
        target: &RuntimePreworkWindowTarget,
    ) -> bool {
        pending.target_block_sequence == target.target_block_sequence
            && pending.admitted_from_block_sequence == target.admitted_from_block_sequence
            && pending.parameter_epoch_override == target.parameter_epoch_override
            && pending.transport_override == target.transport_override
            && pending.input_signature == hash_audio_buffer(&target.buffer)
            && pending.buffer.frames() == target.buffer.frames()
            && pending.buffer.channel_count() == target.buffer.channel_count()
    }

    fn prepared_target_matches(
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        target: &RuntimePreworkWindowTarget,
        projection_epoch: u64,
        latest_parameter_epoch: u64,
        applied_transport: Option<TransportProjection>,
        block_size: usize,
    ) -> bool {
        let transport = target.transport_override.or(applied_transport);
        cache.graph_id == graph_id
            && cache.source_block_sequence == target.target_block_sequence
            && cache.admitted_from_block_sequence == target.admitted_from_block_sequence
            && cache.projection_epoch == projection_epoch
            && cache.parameter_epoch
                == target
                    .parameter_epoch_override
                    .unwrap_or(latest_parameter_epoch)
            && cache.transport.playing == transport.map(|t| t.playing).unwrap_or(false)
            && cache.transport.tempo_bpm == transport.map(|t| t.tempo_bpm).unwrap_or(0.0)
            && cache.transport.timeline_position_samples
                == transport.map(|t| t.timeline_position_samples).unwrap_or(0)
            && cache.block_size == block_size
            && cache.frame_count == target.buffer.frames().0
            && cache.channel_count == target.buffer.channel_count().0
            && cache.input_signature == hash_audio_buffer(&target.buffer)
    }

    fn reconcile_pending_prework_targets(
        &mut self,
        targets: &[RuntimePreworkWindowTarget],
        graph_id: Option<&str>,
        projection_epoch: u64,
        latest_parameter_epoch: u64,
        applied_transport: Option<TransportProjection>,
        block_size: usize,
    ) {
        self.pending_prework_targets.retain(|pending| {
            targets
                .iter()
                .any(|target| Self::pending_target_matches(pending, target))
        });

        for target in targets {
            let already_prepared = graph_id.is_some_and(|graph_id| {
                self.prework_queue.iter().any(|cache| {
                    Self::prepared_target_matches(
                        cache,
                        graph_id,
                        target,
                        projection_epoch,
                        latest_parameter_epoch,
                        applied_transport,
                        block_size,
                    )
                })
            });
            let already_pending = self
                .pending_prework_targets
                .iter()
                .any(|pending| Self::pending_target_matches(pending, target));
            if !already_prepared && !already_pending {
                self.pending_prework_targets
                    .push_back(RuntimePendingPreworkTarget {
                        target_block_sequence: target.target_block_sequence,
                        admitted_from_block_sequence: target.admitted_from_block_sequence,
                        input_signature: hash_audio_buffer(&target.buffer),
                        buffer: target.buffer.clone(),
                        backlog_class: Self::classify_prework_backlog_class(
                            target.target_block_sequence,
                            target.admitted_from_block_sequence,
                        ),
                        parameter_epoch_override: target.parameter_epoch_override,
                        transport_override: target.transport_override,
                    });
            }
        }

        self.pending_prework_targets
            .make_contiguous()
            .sort_by_key(|target| (target.backlog_class, target.target_block_sequence));
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    fn take_pending_prework_targets(
        &mut self,
        budget: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> Vec<RuntimePendingPreworkTarget> {
        let mut drained = Vec::with_capacity(budget.min(self.pending_prework_targets.len()));
        let mut retained = VecDeque::with_capacity(self.pending_prework_targets.len());
        while let Some(target) = self.pending_prework_targets.pop_front() {
            if drained.len() < budget && target.backlog_class <= max_backlog_class {
                drained.push(target);
            } else {
                retained.push_back(target);
            }
        }
        self.pending_prework_targets = retained;
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        drained
    }

    fn matching_prework_index(
        &self,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> Option<usize> {
        self.prework_queue
            .iter()
            .enumerate()
            .rev()
            .find(|(_, cache)| {
                self.prework_cache_matches(cache, graph_id, context, buffer, input_signature)
            })
            .map(|(index, _)| index)
    }

    fn retire_unready_or_mismatched_prework_for_current_block(
        &mut self,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) {
        let mut index = 0;
        while index < self.prework_queue.len() {
            let maybe_reason = {
                let cache = &self.prework_queue[index];
                if context.processing_epoch > cache.valid_until_processing_epoch {
                    Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
                } else if cache.source_block_sequence > context.block_sequence {
                    None
                } else {
                    self.prework_cache_mismatch_reason(
                        cache,
                        graph_id,
                        context,
                        buffer,
                        input_signature,
                    )
                }
            };

            if let Some(reason) = maybe_reason {
                let cache = self.prework_queue.remove(index).expect("queue index valid");
                self.retire_prework_entry(cache, reason);
            } else {
                index += 1;
            }
        }
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    fn apply_graph_projection(
        &mut self,
        projection: &GraphProjection,
        anticipative_enabled: bool,
    ) -> Result<(), RuntimeError> {
        if projection.node_count != projection.nodes.len() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph node_count must match node projection count",
            ));
        }
        if projection
            .nodes
            .iter()
            .any(|node| node.node_id.is_empty() || node.stages.is_empty())
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph nodes must have non-empty ids and at least one stage",
            ));
        }
        if projection.nodes.iter().any(|node| {
            matches!(node.execution_class, GraphNodeExecutionClass::PureTransform)
                && node.latency_samples != 0
        }) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "pure-transform graph nodes must report zero latency",
            ));
        }
        if projection.nodes.iter().any(|node| {
            matches!(
                node.execution_class,
                GraphNodeExecutionClass::LatencyBearing
            ) && node.latency_samples == 0
        }) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "latency-bearing graph nodes must report non-zero latency",
            ));
        }

        self.graph = Some(ExecutableGraph::new(
            projection.graph_id.clone(),
            projection
                .nodes
                .iter()
                .map(|node| GraphNodeSpec {
                    node_id: node.node_id.clone(),
                    execution_class: node.execution_class,
                    latency_samples: node.latency_samples,
                    buffer_contract: GraphNodeBufferContract::default(),
                    topology: GraphNodeTopologyMetadata::default(),
                    stages: node.stages.clone(),
                })
                .collect(),
        ));
        self.plugin_node_bindings.clear();
        self.invalidate_prework_cache(RuntimePreworkInvalidationReason::GraphProjectionChanged);
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: &PluginBackedNodeBindingProjection,
        anticipative_enabled: bool,
    ) -> Result<(), RuntimeError> {
        let Some(graph) = self.graph.as_ref() else {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "cannot bind plugin-backed nodes before a graph is applied",
            ));
        };
        if projection.graph_id != graph.graph_id() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "plugin-backed node bindings must target the currently applied graph",
            ));
        }

        let planning = graph.planning_summary(anticipative_enabled);
        let mut bindings = HashMap::new();
        for binding in &projection.bindings {
            if !planning.planned_nodes.iter().any(|node| {
                node.node_id == binding.node_id
                    && matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked)
            }) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "plugin-backed binding node '{}' does not resolve to a plugin-backed node",
                        binding.node_id
                    ),
                ));
            }
            if bindings
                .insert(binding.node_id.clone(), binding.sandbox_id.clone())
                .is_some()
            {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "duplicate plugin-backed binding provided for node '{}'",
                        binding.node_id
                    ),
                ));
            }
        }

        self.plugin_node_bindings = bindings;
        self.refresh_planning(anticipative_enabled);
        Ok(())
    }

    fn refresh_planning(&mut self, anticipative_enabled: bool) {
        if !anticipative_enabled {
            self.invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        }
        if let Some(graph) = self.graph.as_ref() {
            let planning = graph.planning_summary(anticipative_enabled);
            self.snapshot.graph_id = Some(graph.graph_id().to_string());
            self.snapshot.node_count = graph.node_count();
            self.snapshot.stateful_node_count = graph.stateful_node_count();
            self.snapshot.latency_node_count = graph.latency_node_count();
            self.snapshot.plugin_backed_node_count = graph.plugin_backed_node_count();
            self.snapshot.anticipative_planning_enabled = anticipative_enabled;
            self.snapshot.inline_realtime_node_count = planning.inline_realtime_node_count;
            self.snapshot.stateful_realtime_node_count = planning.stateful_realtime_node_count;
            self.snapshot.anticipative_eligible_node_count =
                planning.anticipative_eligible_node_count;
            self.snapshot.phase_count = planning.phase_count;
            self.snapshot.anticipative_phase_count = planning.anticipative_phase_count;
            self.snapshot.phase_order = planning.phase_order.clone();
            self.snapshot.lane_count = planning.lane_count;
            self.snapshot.anticipative_lane_count = planning.anticipative_lane_count;
            self.snapshot.lane_order = planning.lane_order.clone();
            self.snapshot.dispatch_count = planning.dispatch_count;
            self.snapshot.dispatch_boundary_count = planning.dispatch_boundary_count;
            self.snapshot.dispatch_order = planning
                .dispatches
                .iter()
                .map(|dispatch| dispatch.lane)
                .collect();
            self.snapshot.prepared_dispatch_count = planning
                .dispatches
                .iter()
                .filter(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Anticipative)
                .count();
            self.snapshot.realtime_dispatch_count = planning
                .dispatches
                .iter()
                .filter(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Realtime)
                .count();
            self.snapshot.dispatch_handoff_count = usize::from(
                self.snapshot.prepared_dispatch_count > 0
                    && self.snapshot.realtime_dispatch_count > 0,
            );
            self.snapshot.prework_cache_enabled = self.snapshot.prepared_dispatch_count > 0;
            self.snapshot.prework_cache_block_freshness_window =
                PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
            self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
            self.snapshot.prework_pending_target_count = self.pending_prework_targets.len();
            self.snapshot.prework_cache_state = if !self.snapshot.prework_cache_enabled {
                RuntimePreworkCacheState::Disabled
            } else if !self.prework_queue.is_empty() {
                match self.snapshot.prework_cache_state {
                    RuntimePreworkCacheState::Consumed => RuntimePreworkCacheState::Consumed,
                    RuntimePreworkCacheState::Admitted => RuntimePreworkCacheState::Admitted,
                    _ => RuntimePreworkCacheState::Admitted,
                }
            } else if matches!(
                self.snapshot.prework_cache_state,
                RuntimePreworkCacheState::Invalidated
            ) {
                RuntimePreworkCacheState::Invalidated
            } else {
                RuntimePreworkCacheState::Empty
            };
            self.snapshot.last_prework_cache_hit = false;
            let latest = self.prework_queue.back();
            self.snapshot.prework_cache_freshness_state =
                self.prework_freshness_state(latest, None);
            self.snapshot.prework_cache_remaining_valid_blocks = latest.map(|cache| {
                cache
                    .valid_until_block_sequence
                    .saturating_sub(cache.source_block_sequence)
            });
            self.snapshot.prework_cache_valid_until_processing_epoch =
                latest.map(|cache| cache.valid_until_processing_epoch);
            self.snapshot.prework_cache_valid_until_block_sequence =
                latest.map(|cache| cache.valid_until_block_sequence);
            self.snapshot.last_prework_source_processing_epoch =
                latest.map(|cache| cache.source_processing_epoch);
            self.snapshot.last_prework_source_block_sequence =
                latest.map(|cache| cache.source_block_sequence);
            self.snapshot.last_prework_admission_processing_epoch =
                latest.map(|cache| cache.source_processing_epoch);
            self.snapshot.last_prework_admission_block_sequence =
                latest.map(|cache| cache.source_block_sequence);
            self.snapshot.last_prework_admitted_from_block_sequence =
                latest.map(|cache| cache.admitted_from_block_sequence);
            self.snapshot.last_prework_retirement_processing_epoch = None;
            self.snapshot.last_prework_retirement_block_sequence = None;
            self.snapshot.prework_cache_queue_depth = self.prework_queue.len();
            self.snapshot.prework_cache_peak_queue_depth = self
                .snapshot
                .prework_cache_peak_queue_depth
                .max(self.prework_queue.len());
            self.snapshot.planned_nodes = planning
                .planned_nodes
                .into_iter()
                .map(|node| crate::interfaces::RuntimePlannedGraphNode {
                    plugin_sandbox_id: self.plugin_node_bindings.get(&node.node_id).cloned(),
                    node_id: node.node_id,
                    execution_class: node.execution_class,
                    group: node.group,
                    latency_samples: node.latency_samples,
                })
                .collect();
            self.snapshot.stage_count = graph.stage_count();
            self.snapshot.total_latency_samples = graph.total_latency_samples();
            self.snapshot.max_node_latency_samples = graph.max_node_latency_samples();
        } else {
            self.snapshot.anticipative_planning_enabled = anticipative_enabled;
            self.snapshot.inline_realtime_node_count = 0;
            self.snapshot.stateful_realtime_node_count = 0;
            self.snapshot.anticipative_eligible_node_count = 0;
            self.snapshot.plugin_backed_node_count = 0;
            self.snapshot.phase_count = 0;
            self.snapshot.anticipative_phase_count = 0;
            self.snapshot.phase_order.clear();
            self.snapshot.lane_count = 0;
            self.snapshot.anticipative_lane_count = 0;
            self.snapshot.lane_order.clear();
            self.snapshot.dispatch_count = 0;
            self.snapshot.dispatch_boundary_count = 0;
            self.snapshot.dispatch_order.clear();
            self.snapshot.prepared_dispatch_count = 0;
            self.snapshot.realtime_dispatch_count = 0;
            self.snapshot.dispatch_handoff_count = 0;
            self.snapshot.prework_cache_enabled = false;
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Disabled;
            self.snapshot.last_prework_cache_hit = false;
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Disabled;
            self.snapshot.prework_cache_block_freshness_window =
                PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
            self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
            self.snapshot.prework_cache_queue_depth = 0;
            self.snapshot.prework_pending_target_count = 0;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.last_prework_invalidation_reason = None;
            self.snapshot.prework_cache_valid_until_processing_epoch = None;
            self.snapshot.prework_cache_valid_until_block_sequence = None;
            self.snapshot.last_prework_source_processing_epoch = None;
            self.snapshot.last_prework_source_block_sequence = None;
            self.snapshot.last_prework_admission_processing_epoch = None;
            self.snapshot.last_prework_admission_block_sequence = None;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            self.snapshot.last_prework_consumption_processing_epoch = None;
            self.snapshot.last_prework_consumption_block_sequence = None;
            self.snapshot.last_prework_consumed_from_block_sequence = None;
            self.snapshot.last_prework_retirement_processing_epoch = None;
            self.snapshot.last_prework_retirement_block_sequence = None;
            self.snapshot.planned_nodes.clear();
            self.plugin_node_bindings.clear();
            self.prework_queue.clear();
            self.pending_prework_targets.clear();
        }
    }

    fn process_block(
        &mut self,
        context: GraphExecutionContext,
        buffer: AudioBuffer,
    ) -> Result<RuntimeEngineBlockResult, RuntimeError> {
        let graph_id = self
            .graph
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidState,
                    "no executable graph has been applied",
                )
            })?
            .graph_id()
            .to_string();

        let input_signature = hash_audio_buffer(&buffer);
        self.retire_unready_or_mismatched_prework_for_current_block(
            graph_id.as_str(),
            &context,
            &buffer,
            input_signature,
        );

        let cache_hit_index =
            self.matching_prework_index(graph_id.as_str(), &context, &buffer, input_signature);
        let cache_hit = cache_hit_index.is_some();

        let prepared = if cache_hit {
            self.snapshot.prework_cache_hits = self.snapshot.prework_cache_hits.saturating_add(1);
            self.snapshot.prework_cache_consumptions =
                self.snapshot.prework_cache_consumptions.saturating_add(1);
            if self.prework_queue[cache_hit_index.expect("cache hit index present")]
                .admitted_from_block_sequence
                < context.block_sequence
            {
                self.snapshot.prework_cache_queued_consumptions = self
                    .snapshot
                    .prework_cache_queued_consumptions
                    .saturating_add(1);
            }
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Consumed;
            self.snapshot.last_prework_consumption_processing_epoch =
                Some(context.processing_epoch);
            self.snapshot.last_prework_consumption_block_sequence = Some(context.block_sequence);
            let cache = &mut self.prework_queue[cache_hit_index.expect("cache hit index present")];
            self.snapshot.last_prework_source_processing_epoch =
                Some(cache.source_processing_epoch);
            self.snapshot.last_prework_source_block_sequence = Some(cache.source_block_sequence);
            self.snapshot.last_prework_admission_processing_epoch =
                Some(cache.source_processing_epoch);
            self.snapshot.last_prework_admission_block_sequence = Some(cache.source_block_sequence);
            self.snapshot.last_prework_admitted_from_block_sequence =
                Some(cache.admitted_from_block_sequence);
            self.snapshot.last_prework_consumed_from_block_sequence =
                Some(cache.admitted_from_block_sequence);
            cache.consumption_count = cache.consumption_count.saturating_add(1);
            Some(cache.prepared.clone())
        } else {
            let anticipative_dispatches_present = self
                .graph
                .as_ref()
                .map(|graph| {
                    graph
                        .planning_summary(context.anticipative_enabled)
                        .dispatches
                        .iter()
                        .any(|dispatch| {
                            dispatch.lane == signal_graph::GraphExecutionLane::Anticipative
                        })
                })
                .unwrap_or(false);
            let planning = self
                .graph
                .as_ref()
                .map(|graph| graph.planning_summary(context.anticipative_enabled))
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "no executable graph has been applied",
                    )
                })?;
            if anticipative_dispatches_present {
                self.snapshot.prework_cache_misses =
                    self.snapshot.prework_cache_misses.saturating_add(1);
            }
            let admitted = self.admit_prework_for_block(
                context.clone(),
                context.block_sequence,
                buffer.clone(),
            )?;
            let prepared = if admitted {
                self.prework_queue
                    .iter()
                    .rev()
                    .find(|cache| cache.source_block_sequence == context.block_sequence)
                    .map(|cache| cache.prepared.clone())
            } else {
                None
            };
            self.snapshot.prework_cache_state = if !self.prework_queue.is_empty() {
                RuntimePreworkCacheState::Admitted
            } else if planning
                .dispatches
                .iter()
                .any(|dispatch| dispatch.lane == signal_graph::GraphExecutionLane::Anticipative)
            {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
            prepared
        };
        let prepared_was_used = prepared.is_some();
        let graph = self.graph.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "no executable graph has been applied",
            )
        })?;
        let planning = graph.planning_summary(context.anticipative_enabled);
        let contract = graph.contract_summary();

        let (
            output,
            GraphBlockReport {
                graph_id,
                context,
                node_count,
                stateful_node_count,
                latency_node_count,
                plugin_backed_node_count,
                phase_count,
                anticipative_phase_count,
                phase_order,
                lane_count,
                anticipative_lane_count,
                lane_order,
                dispatch_count,
                dispatch_boundary_count,
                dispatch_order,
                prepared_dispatch_count,
                realtime_dispatch_count,
                dispatch_handoff_count,
                stage_count,
                total_latency_samples,
                max_node_latency_samples,
                frame_count,
                channel_count,
                input_peak,
                prework_output_peak,
                realtime_input_peak,
                output_peak,
                output_rms,
                first_output_sample,
                ..
            },
        ) = graph.execute_realtime_from_prepared(
            &buffer,
            peak_abs(buffer.samples()),
            prepared,
            context,
            &planning,
            &contract,
        );

        self.snapshot.graph_id = Some(graph_id);
        self.snapshot.node_count = node_count;
        self.snapshot.stateful_node_count = stateful_node_count;
        self.snapshot.latency_node_count = latency_node_count;
        self.snapshot.plugin_backed_node_count = plugin_backed_node_count;
        self.snapshot.phase_count = phase_count;
        self.snapshot.anticipative_phase_count = anticipative_phase_count;
        self.snapshot.phase_order = phase_order;
        self.snapshot.lane_count = lane_count;
        self.snapshot.anticipative_lane_count = anticipative_lane_count;
        self.snapshot.lane_order = lane_order;
        self.snapshot.dispatch_count = dispatch_count;
        self.snapshot.dispatch_boundary_count = dispatch_boundary_count;
        self.snapshot.dispatch_order = dispatch_order;
        self.snapshot.prepared_dispatch_count = prepared_dispatch_count;
        self.snapshot.realtime_dispatch_count = realtime_dispatch_count;
        self.snapshot.dispatch_handoff_count = dispatch_handoff_count;
        self.snapshot.prework_cache_enabled = prepared_dispatch_count > 0;
        if !self.snapshot.prework_cache_enabled {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Disabled;
        } else if !self.prework_queue.is_empty() {
            self.snapshot.prework_cache_state = if cache_hit {
                RuntimePreworkCacheState::Consumed
            } else {
                RuntimePreworkCacheState::Admitted
            };
        } else if !matches!(
            self.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        ) {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Empty;
        }
        self.snapshot.last_prework_cache_hit = cache_hit;
        self.snapshot.prework_cache_block_freshness_window = PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW;
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        if !cache_hit && self.prework_queue.is_empty() {
            self.snapshot.last_prework_admission_processing_epoch = None;
            self.snapshot.last_prework_admission_block_sequence = None;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
        }
        if !cache_hit && prepared_was_used {
            self.snapshot.prework_cache_consumptions =
                self.snapshot.prework_cache_consumptions.saturating_add(1);
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Consumed;
            self.snapshot.last_prework_consumption_processing_epoch =
                Some(context.processing_epoch);
            self.snapshot.last_prework_consumption_block_sequence = Some(context.block_sequence);
            if let Some(cache) = self
                .prework_queue
                .iter_mut()
                .rev()
                .find(|cache| cache.source_block_sequence == context.block_sequence)
            {
                self.snapshot.last_prework_source_processing_epoch =
                    Some(cache.source_processing_epoch);
                self.snapshot.last_prework_source_block_sequence =
                    Some(cache.source_block_sequence);
                self.snapshot.last_prework_admission_processing_epoch =
                    Some(cache.source_processing_epoch);
                self.snapshot.last_prework_admission_block_sequence =
                    Some(cache.source_block_sequence);
                self.snapshot.last_prework_admitted_from_block_sequence =
                    Some(cache.admitted_from_block_sequence);
                self.snapshot.last_prework_consumed_from_block_sequence =
                    Some(cache.admitted_from_block_sequence);
                cache.consumption_count = cache.consumption_count.saturating_add(1);
            }
        } else if !cache_hit {
            self.snapshot.last_prework_consumed_from_block_sequence = None;
        }
        self.snapshot.stage_count = stage_count;
        self.snapshot.total_latency_samples = total_latency_samples;
        self.snapshot.max_node_latency_samples = max_node_latency_samples;
        self.snapshot.processed_blocks = self.snapshot.processed_blocks.saturating_add(1);
        self.snapshot.last_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_block_sequence = Some(context.block_sequence);
        self.snapshot.last_frame_count = frame_count;
        self.snapshot.last_channel_count = channel_count;
        self.snapshot.last_input_peak = Some(input_peak);
        self.snapshot.last_prework_output_peak = prework_output_peak;
        self.snapshot.last_realtime_input_peak = realtime_input_peak;
        self.snapshot.last_output_peak = Some(output_peak);
        self.snapshot.last_output_rms = Some(output_rms);
        self.snapshot.last_first_output_sample = first_output_sample;
        self.snapshot.last_execution_context = Some(context);

        Ok(RuntimeEngineBlockResult {
            snapshot: self.snapshot.clone(),
            output,
        })
    }

    fn admit_prework_for_block(
        &mut self,
        context: GraphExecutionContext,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<bool, RuntimeError> {
        let (graph_id, planning) = {
            let graph = self.graph.as_ref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidState,
                    "no executable graph has been applied",
                )
            })?;
            (
                graph.graph_id().to_string(),
                graph.planning_summary(context.anticipative_enabled),
            )
        };
        if planning
            .dispatches
            .iter()
            .all(|dispatch| dispatch.lane != signal_graph::GraphExecutionLane::Anticipative)
        {
            self.snapshot.prework_cache_state = if context.anticipative_enabled {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            return Ok(false);
        }

        let input_signature = hash_audio_buffer(&buffer);
        let already_matching = self.prework_queue.iter().any(|cache| {
            self.prework_cache_matches(cache, graph_id.as_str(), &context, &buffer, input_signature)
        });
        if already_matching {
            return Ok(true);
        }
        self.retire_prework_entries_matching(
            |cache| cache.source_block_sequence == context.block_sequence,
            RuntimePreworkInvalidationReason::SupersededByAdmission,
        );

        let graph = self.graph.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "no executable graph has been applied",
            )
        })?;
        let Some(prepared) = graph.prepare_anticipative(&buffer, &context) else {
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Empty;
            self.snapshot.last_prework_admitted_from_block_sequence = None;
            return Ok(false);
        };

        self.snapshot.prework_cache_admissions =
            self.snapshot.prework_cache_admissions.saturating_add(1);
        if admitted_from_block_sequence < context.block_sequence {
            self.snapshot.prework_cache_queued_admissions = self
                .snapshot
                .prework_cache_queued_admissions
                .saturating_add(1);
        }
        self.snapshot.last_prework_admission_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_prework_admission_block_sequence = Some(context.block_sequence);
        self.snapshot.last_prework_admitted_from_block_sequence =
            Some(admitted_from_block_sequence);
        self.prework_queue.push_back(RuntimeEnginePreworkCache {
            graph_id,
            projection_epoch: context.projection_epoch,
            parameter_epoch: context.parameter_epoch,
            transport: TransportProjection {
                playing: context.transport_playing,
                timeline_position_samples: context.timeline_position_samples,
                tempo_bpm: context.transport_tempo_bpm,
                loop_state: None,
            },
            block_size: context.configured_block_size,
            frame_count: buffer.frames().0,
            channel_count: buffer.channel_count().0,
            input_signature,
            prepared,
            valid_until_processing_epoch: context.processing_epoch.saturating_add(1),
            valid_until_block_sequence: context
                .block_sequence
                .saturating_add(PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW),
            source_processing_epoch: context.processing_epoch,
            source_block_sequence: context.block_sequence,
            admitted_from_block_sequence,
            consumption_count: 0,
        });
        self.prework_queue
            .make_contiguous()
            .sort_by_key(|cache| cache.source_block_sequence);
        while self.prework_queue.len() > PREWORK_QUEUE_CAPACITY {
            let cache = self.prework_queue.pop_front().expect("queue not empty");
            self.retire_prework_entry(
                cache,
                RuntimePreworkInvalidationReason::QueueCapacityExceeded,
            );
        }
        self.snapshot.prework_cache_state = RuntimePreworkCacheState::Admitted;
        self.snapshot.last_prework_source_processing_epoch = Some(context.processing_epoch);
        self.snapshot.last_prework_source_block_sequence = Some(context.block_sequence);
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        Ok(true)
    }

    fn snapshot(&self) -> RuntimeEngineBlockSnapshot {
        self.snapshot.clone()
    }

    fn prework_cache_matches(
        &self,
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> bool {
        context.anticipative_enabled
            && cache.graph_id == graph_id
            && cache.projection_epoch == context.projection_epoch
            && cache.parameter_epoch == context.parameter_epoch
            && cache.transport.playing == context.transport_playing
            && cache.transport.tempo_bpm == context.transport_tempo_bpm
            && cache.transport.timeline_position_samples == context.timeline_position_samples
            && cache.block_size == context.configured_block_size
            && cache.frame_count == buffer.frames().0
            && cache.channel_count == buffer.channel_count().0
            && cache.input_signature == input_signature
            && context.processing_epoch <= cache.valid_until_processing_epoch
            && context.block_sequence <= cache.valid_until_block_sequence
    }

    fn prework_cache_mismatch_reason(
        &self,
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> Option<RuntimePreworkInvalidationReason> {
        if !context.anticipative_enabled {
            return Some(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        }
        if cache.graph_id != graph_id || cache.projection_epoch != context.projection_epoch {
            return Some(RuntimePreworkInvalidationReason::GraphProjectionChanged);
        }
        if cache.parameter_epoch != context.parameter_epoch {
            return Some(RuntimePreworkInvalidationReason::ParameterBatchApplied);
        }
        if cache.transport.playing != context.transport_playing
            || cache.transport.tempo_bpm != context.transport_tempo_bpm
            || cache.transport.timeline_position_samples != context.timeline_position_samples
        {
            return Some(RuntimePreworkInvalidationReason::TransportChanged);
        }
        if context.processing_epoch > cache.valid_until_processing_epoch {
            return Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired);
        }
        if context.block_sequence > cache.valid_until_block_sequence {
            return Some(RuntimePreworkInvalidationReason::BlockSequenceExpired);
        }
        if cache.block_size != context.configured_block_size
            || cache.frame_count != buffer.frames().0
            || cache.channel_count != buffer.channel_count().0
            || cache.input_signature != input_signature
        {
            return Some(RuntimePreworkInvalidationReason::InputSignatureChanged);
        }
        None
    }

    fn invalidate_prework_cache(&mut self, reason: RuntimePreworkInvalidationReason) {
        if !self.prework_queue.is_empty() {
            let drained = self.prework_queue.drain(..).collect::<Vec<_>>();
            for cache in drained {
                self.retire_prework_entry(cache, reason);
            }
            self.pending_prework_targets.clear();
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Invalidated;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.prework_cache_valid_until_processing_epoch = None;
            self.snapshot.prework_cache_valid_until_block_sequence = None;
            self.snapshot.last_prework_source_processing_epoch = None;
            self.snapshot.last_prework_source_block_sequence = None;
            self.snapshot.prework_cache_queue_depth = 0;
        } else if self.snapshot.prework_cache_enabled {
            self.pending_prework_targets.clear();
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Invalidated;
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Invalidated;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.last_prework_invalidation_reason = Some(reason);
        }
    }

    fn retirement_reason_from_invalidation(
        &self,
        reason: RuntimePreworkInvalidationReason,
    ) -> RuntimePreworkRetirementReason {
        match reason {
            RuntimePreworkInvalidationReason::RuntimeReconfigured => {
                RuntimePreworkRetirementReason::RuntimeReconfigured
            }
            RuntimePreworkInvalidationReason::RuntimeStopped => {
                RuntimePreworkRetirementReason::RuntimeStopped
            }
            RuntimePreworkInvalidationReason::ForecastPlanChanged => {
                RuntimePreworkRetirementReason::ForecastPlanChanged
            }
            RuntimePreworkInvalidationReason::PlanningDisabled => {
                RuntimePreworkRetirementReason::PlanningDisabled
            }
            RuntimePreworkInvalidationReason::GraphProjectionChanged => {
                RuntimePreworkRetirementReason::GraphProjectionChanged
            }
            RuntimePreworkInvalidationReason::TransportChanged => {
                RuntimePreworkRetirementReason::TransportChanged
            }
            RuntimePreworkInvalidationReason::ParameterBatchApplied => {
                RuntimePreworkRetirementReason::ParameterBatchApplied
            }
            RuntimePreworkInvalidationReason::InputSignatureChanged => {
                RuntimePreworkRetirementReason::InputSignatureChanged
            }
            RuntimePreworkInvalidationReason::ProcessingEpochExpired => {
                RuntimePreworkRetirementReason::ProcessingEpochExpired
            }
            RuntimePreworkInvalidationReason::BlockSequenceExpired => {
                RuntimePreworkRetirementReason::BlockSequenceExpired
            }
            RuntimePreworkInvalidationReason::SupersededByAdmission => {
                RuntimePreworkRetirementReason::SupersededByAdmission
            }
            RuntimePreworkInvalidationReason::PlanningWindowRevised => {
                RuntimePreworkRetirementReason::PlanningWindowRevised
            }
            RuntimePreworkInvalidationReason::QueueCapacityExceeded => {
                RuntimePreworkRetirementReason::QueueCapacityExceeded
            }
        }
    }

    fn prework_freshness_state(
        &self,
        cache: Option<&RuntimeEnginePreworkCache>,
        current_block_sequence: Option<u64>,
    ) -> RuntimePreworkFreshnessState {
        if !self.snapshot.prework_cache_enabled {
            return RuntimePreworkFreshnessState::Disabled;
        }
        if matches!(
            self.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        ) {
            return RuntimePreworkFreshnessState::Invalidated;
        }
        let Some(cache) = cache else {
            return RuntimePreworkFreshnessState::Empty;
        };
        let Some(current_block_sequence) = current_block_sequence else {
            return RuntimePreworkFreshnessState::Fresh;
        };
        let remaining = cache
            .valid_until_block_sequence
            .saturating_sub(current_block_sequence);
        match remaining {
            0 => RuntimePreworkFreshnessState::Exhausted,
            1 => RuntimePreworkFreshnessState::Expiring,
            _ => RuntimePreworkFreshnessState::Fresh,
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
            .field("engine", &self.engine)
            .field("diagnostics", &self.diagnostics)
            .field("supervision", &self.supervision)
            .finish()
    }
}

impl SignalRuntime {
    fn summarize_plugin_backed_bindings(&self) -> RuntimePluginBackedBindingSummary {
        let bound_sandbox_ids = self
            .engine
            .snapshot
            .planned_nodes
            .iter()
            .filter(|node| matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked))
            .filter_map(|node| node.plugin_sandbox_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let mut summary = RuntimePluginBackedBindingSummary {
            bound_sandbox_ids,
            ..RuntimePluginBackedBindingSummary::default()
        };
        for sandbox_id in &summary.bound_sandbox_ids {
            let matching_states = self
                .transport_concurrency
                .active_sessions
                .values()
                .filter(|session| session.sandbox_id == *sandbox_id)
                .map(|session| session.state)
                .collect::<Vec<_>>();
            if matching_states
                .iter()
                .any(|state| matches!(state, TransportSessionState::AttachActive))
            {
                summary.active_bound_sandboxes += 1;
            } else if matching_states.iter().any(|state| {
                matches!(
                    state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
            }) {
                summary.degraded_bound_sandboxes += 1;
            } else {
                summary.missing_bound_sandboxes += 1;
            }
        }
        summary
    }

    fn recompute_prework_service_policy_snapshot(&mut self) {
        let binding_summary = self.summarize_plugin_backed_bindings();
        let semantic_policy = self
            .engine
            .graph
            .as_ref()
            .map(|graph| {
                RuntimeEngineState::classify_prework_service_semantic_policy(
                    graph,
                    self.anticipative_enabled,
                    if !binding_summary.bound_sandbox_ids.is_empty() {
                        binding_summary.active_bound_sandboxes as u32
                            + binding_summary.degraded_bound_sandboxes as u32
                            + binding_summary.missing_bound_sandboxes as u32
                    } else {
                        self.diagnostics.active_plugin_sandboxes
                    },
                )
            })
            .unwrap_or(RuntimePreworkServiceSemanticPolicy::Balanced);
        let plugin_gate_active = matches!(
            semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        ) && self.engine.snapshot.prework_service_pressure
            != RuntimePreworkServicePressure::Normal
            && if !binding_summary.bound_sandbox_ids.is_empty() {
                binding_summary.degraded_bound_sandboxes > 0
                    || binding_summary.missing_bound_sandboxes > 0
                    || binding_summary.active_bound_sandboxes > 1
            } else {
                self.diagnostics.active_plugin_sandboxes > 1
            };
        self.engine.snapshot.prework_service_semantic_policy = semantic_policy;
        self.engine.set_prework_service_plugin_state(
            self.diagnostics.active_plugin_sandboxes,
            binding_summary.bound_sandbox_ids.len(),
            binding_summary.active_bound_sandboxes,
            binding_summary.degraded_bound_sandboxes,
            binding_summary.missing_bound_sandboxes,
            plugin_gate_active,
        );
    }

    pub fn new(config: RuntimeConfig) -> Self {
        let mut runtime = Self {
            config,
            readiness: RuntimeReadiness::Stopped,
            safe_mode_enabled: false,
            anticipative_enabled: true,
            active_output_device: None,
            applied_graph: None,
            applied_schedule: None,
            applied_transport: None,
            prework_forecast_requested_mode: RuntimePreworkForecastMode::RuntimeRoleDefault,
            prework_forecast_mode: RuntimePreworkForecastMode::Disabled,
            prework_forecast_policy: None,
            prework_forecast_profile: None,
            prework_forecast_profile_source: None,
            latest_parameter_epoch: 0,
            projection_epoch: 0,
            control: RuntimeControlSnapshot::default(),
            timeline: RuntimeTimelineState::default(),
            automation: RuntimeAutomationState::default(),
            engine: RuntimeEngineState::default(),
            transport_concurrency: RuntimeTransportConcurrencyState::default(),
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
        };
        runtime.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::RuntimeRoleDefault,
        );
        runtime.set_prework_forecast_mode_internal(RuntimePreworkForecastMode::Disabled);
        runtime.recompute_prework_service_policy_snapshot();
        runtime
    }

    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    fn set_prework_forecast_policy_internal(
        &mut self,
        policy: Option<RuntimePreworkForecastPolicy>,
    ) {
        self.prework_forecast_policy = policy.clone();
        self.engine.snapshot.prework_forecast_policy_configured = policy.is_some();
        self.engine
            .snapshot
            .prework_forecast_policy_target_window_blocks =
            policy.as_ref().map(|policy| policy.target_window_blocks);
    }

    fn set_prework_forecast_requested_mode_internal(&mut self, mode: RuntimePreworkForecastMode) {
        self.prework_forecast_requested_mode = mode;
        self.engine.snapshot.prework_forecast_requested_mode = mode;
    }

    fn set_prework_forecast_mode_internal(&mut self, mode: RuntimePreworkForecastMode) {
        self.prework_forecast_mode = mode;
        self.engine.snapshot.prework_forecast_mode = mode;
    }

    fn reconcile_prework_service_state(&mut self, processing_epoch: Option<u64>) {
        let state = if !self.engine.snapshot.prework_cache_enabled
            || self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
        {
            RuntimePreworkServiceState::Disabled
        } else if !self.control.running {
            RuntimePreworkServiceState::Paused
        } else if !self.engine.pending_prework_targets.is_empty() {
            RuntimePreworkServiceState::Pending
        } else {
            RuntimePreworkServiceState::Idle
        };
        self.engine
            .transition_prework_service_state(state, processing_epoch);
    }

    fn set_prework_forecast_profile_internal(
        &mut self,
        selection: Option<RuntimePreworkForecastProfileSelection>,
        source: Option<RuntimePreworkForecastProfileSource>,
    ) {
        self.prework_forecast_profile = selection;
        self.prework_forecast_profile_source = source;
        self.engine.snapshot.prework_forecast_profile =
            selection.map(|selection| selection.profile);
        self.engine.snapshot.prework_forecast_profile_source = source;
        self.engine
            .snapshot
            .prework_forecast_profile_target_window_override =
            selection.and_then(|selection| selection.target_window_blocks_override);
    }

    fn invalidate_prework_for_forecast_plan_change_if_needed(
        &mut self,
        previous_requested_mode: RuntimePreworkForecastMode,
        previous_effective_mode: RuntimePreworkForecastMode,
        previous_profile: Option<RuntimePreworkForecastProfileSelection>,
        previous_profile_source: Option<RuntimePreworkForecastProfileSource>,
        previous_policy: Option<RuntimePreworkForecastPolicy>,
    ) -> Result<(), RuntimeError> {
        let changed = previous_requested_mode != self.prework_forecast_requested_mode
            || previous_effective_mode != self.prework_forecast_mode
            || previous_profile != self.prework_forecast_profile
            || previous_profile_source != self.prework_forecast_profile_source
            || previous_policy != self.prework_forecast_policy;
        if !changed {
            return Ok(());
        }

        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            self.engine
                .invalidate_prework_cache(RuntimePreworkInvalidationReason::ForecastPlanChanged);
            return Ok(());
        }

        let Some(policy) = self.prework_forecast_policy.clone() else {
            self.engine
                .invalidate_prework_cache(RuntimePreworkInvalidationReason::ForecastPlanChanged);
            return Ok(());
        };
        if self.engine.prework_queue.is_empty() {
            let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        } else {
            self.reconcile_prework_queue_with_current_forecast_plan(&policy);
        }
        self.reconcile_prework_service_state(None);
        Ok(())
    }

    fn reconcile_prework_queue_with_current_forecast_plan(
        &mut self,
        policy: &RuntimePreworkForecastPolicy,
    ) {
        let current_block_sequence = self
            .engine
            .prework_queue
            .iter()
            .map(|cache| cache.admitted_from_block_sequence)
            .max()
            .unwrap_or(0);
        let processing_epoch = self
            .engine
            .prework_queue
            .iter()
            .map(|cache| cache.source_processing_epoch)
            .max()
            .unwrap_or_else(|| {
                self.engine
                    .snapshot
                    .last_processing_epoch
                    .unwrap_or(self.projection_epoch)
            });
        let desired_sequences = (1..=policy.target_window_blocks)
            .map(|offset| current_block_sequence.saturating_add(offset as u64))
            .collect::<Vec<_>>();
        let projection_epoch = self.projection_epoch;
        let sample_rate = self.config.sample_rate;
        let block_size = self.config.graph.block_size;
        let retire_sequences = self
            .engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let expected_loop_length_blocks = policy.transport_loop_length_blocks.max(1);
                let loop_end_samples =
                    (block_size.saturating_mul(expected_loop_length_blocks)) as i64;
                let expected_timeline_position_samples = ((cache.source_block_sequence as i64)
                    .saturating_mul(block_size as i64))
                .rem_euclid(loop_end_samples);
                let expected_parameter_epoch = projection_epoch
                    .saturating_add(cache.source_block_sequence)
                    .saturating_add(1);
                let expected_input_signature = hash_audio_buffer(&synthetic_stereo_block(
                    sample_rate,
                    FrameCount(block_size),
                    cache
                        .source_block_sequence
                        .saturating_add(policy.buffer_seed_offset),
                ));
                let compatible = cache.projection_epoch == projection_epoch
                    && cache.parameter_epoch == expected_parameter_epoch
                    && cache.transport.playing == policy.transport_playing
                    && cache.transport.tempo_bpm == policy.transport_tempo_bpm
                    && cache.transport.timeline_position_samples
                        == expected_timeline_position_samples
                    && cache.block_size == block_size
                    && cache.frame_count == block_size
                    && cache.channel_count == 2
                    && cache.input_signature == expected_input_signature
                    && cache.source_block_sequence > cache.admitted_from_block_sequence
                    && cache.source_block_sequence
                        <= cache
                            .admitted_from_block_sequence
                            .saturating_add(policy.target_window_blocks as u64);
                (!desired_sequences.contains(&cache.source_block_sequence) || !compatible)
                    .then_some(cache.source_block_sequence)
            })
            .collect::<Vec<_>>();
        self.engine.retire_prework_entries_matching(
            |cache| retire_sequences.contains(&cache.source_block_sequence),
            RuntimePreworkInvalidationReason::ForecastPlanChanged,
        );

        let targets = desired_sequences
            .into_iter()
            .map(|target_block_sequence| RuntimePreworkWindowTarget {
                target_block_sequence,
                admitted_from_block_sequence: current_block_sequence,
                buffer: synthetic_stereo_block(
                    sample_rate,
                    FrameCount(block_size),
                    target_block_sequence.saturating_add(policy.buffer_seed_offset),
                ),
                parameter_epoch_override: Some(
                    self.forecast_parameter_batch_for_block(target_block_sequence, policy)
                        .epoch,
                ),
                transport_override: Some(
                    self.forecast_transport_projection_for_block(target_block_sequence, policy),
                ),
            })
            .collect::<Vec<_>>();
        let graph_id = self
            .engine
            .graph
            .as_ref()
            .map(|graph| graph.graph_id().to_string());
        self.engine.reconcile_pending_prework_targets(
            &targets,
            graph_id.as_deref(),
            self.projection_epoch,
            self.latest_parameter_epoch,
            self.applied_transport,
            block_size,
        );
        let _ = self.service_pending_prework_cycle(
            processing_epoch,
            policy.prepare_budget_per_cycle,
            RuntimePreworkBacklogClass::Deferred,
        );
    }

    fn maybe_rebuild_prework_window_from_current_forecast_plan(
        &mut self,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured
            || self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
        {
            return Ok(0);
        }
        if self.engine.graph.is_none() || !self.engine.snapshot.prework_cache_enabled {
            return Ok(0);
        }
        let Some(policy) = self.prework_forecast_policy.clone() else {
            return Ok(0);
        };
        let current_block_sequence = self
            .engine
            .snapshot
            .last_block_sequence
            .or_else(|| self.timeline.next_block_sequence.checked_sub(1))
            .unwrap_or(0);
        let processing_epoch = self
            .engine
            .snapshot
            .last_processing_epoch
            .unwrap_or(self.projection_epoch);
        let rebuilt = self.prime_engine_prework_window_with_forecast(
            processing_epoch,
            current_block_sequence,
            &policy,
        )?;
        self.reconcile_prework_service_state(Some(processing_epoch));
        Ok(rebuilt)
    }

    fn default_prework_forecast_profile_selection_for_runtime_profile(
        profile: RuntimeProfile,
    ) -> RuntimePreworkForecastProfileSelection {
        RuntimePreworkForecastProfileSelection {
            profile: match profile {
                RuntimeProfile::Local => RuntimePreworkForecastProfile::Local,
                RuntimeProfile::Server => RuntimePreworkForecastProfile::Server,
            },
            target_window_blocks_override: None,
        }
    }

    fn prework_forecast_policy_for_profile(
        selection: RuntimePreworkForecastProfileSelection,
    ) -> RuntimePreworkForecastPolicy {
        let mut policy = match selection.profile {
            RuntimePreworkForecastProfile::Local => RuntimePreworkForecastPolicy {
                target_window_blocks: 2,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            },
            RuntimePreworkForecastProfile::Server => RuntimePreworkForecastPolicy {
                target_window_blocks: 2,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 17,
                transport_playing: true,
                transport_tempo_bpm: 122.0,
                transport_loop_length_blocks: 24,
                parameter_target: "engine.server.balance".into(),
                parameter_cycle_length: 6,
            },
        };
        if let Some(target_window_blocks) = selection.target_window_blocks_override {
            policy.target_window_blocks = target_window_blocks;
        }
        policy
    }

    fn reconcile_prework_forecast_mode_state(&mut self) -> Result<(), RuntimeError> {
        match self.prework_forecast_requested_mode {
            RuntimePreworkForecastMode::RuntimeRoleDefault => {
                let selection =
                    Self::default_prework_forecast_profile_selection_for_runtime_profile(
                        self.config.profile,
                    );
                let policy = Self::prework_forecast_policy_for_profile(selection);
                self.set_prework_forecast_profile_internal(
                    Some(selection),
                    Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::ExplicitProfile => {
                let selection = self.prework_forecast_profile.ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "explicit forecast mode requires a stored forecast profile selection",
                    )
                })?;
                let policy = Self::prework_forecast_policy_for_profile(selection);
                self.set_prework_forecast_profile_internal(
                    Some(selection),
                    Some(RuntimePreworkForecastProfileSource::ExplicitSelection),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::RawPolicyOverride => {
                let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "raw forecast override mode requires a stored forecast policy",
                    )
                })?;
                self.set_prework_forecast_profile_internal(
                    None,
                    Some(RuntimePreworkForecastProfileSource::RawPolicyOverride),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::Disabled => {}
        }

        let effective_mode = if self.anticipative_enabled {
            self.prework_forecast_requested_mode
        } else {
            RuntimePreworkForecastMode::Disabled
        };
        self.set_prework_forecast_mode_internal(effective_mode);
        Ok(())
    }

    fn set_prework_forecast_mode_state(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        if mode == RuntimePreworkForecastMode::Disabled
            && self.prework_forecast_mode != RuntimePreworkForecastMode::Disabled
        {
            self.engine
                .invalidate_prework_cache(RuntimePreworkInvalidationReason::PlanningDisabled);
        }
        self.set_prework_forecast_requested_mode_internal(mode);
        match mode {
            RuntimePreworkForecastMode::Disabled => {
                self.reconcile_prework_forecast_mode_state()?;
                self.invalidate_prework_for_forecast_plan_change_if_needed(
                    previous_requested_mode,
                    previous_effective_mode,
                    previous_profile,
                    previous_profile_source,
                    previous_policy,
                )?;
                Ok(())
            }
            RuntimePreworkForecastMode::RuntimeRoleDefault => {
                self.reconcile_prework_forecast_mode_state()?;
                self.invalidate_prework_for_forecast_plan_change_if_needed(
                    previous_requested_mode,
                    previous_effective_mode,
                    previous_profile,
                    previous_profile_source,
                    previous_policy,
                )?;
                Ok(())
            }
            RuntimePreworkForecastMode::ExplicitProfile => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "explicit-profile forecast mode requires a profile selection",
            )),
            RuntimePreworkForecastMode::RawPolicyOverride => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "raw-policy forecast mode requires an explicit forecast policy",
            )),
        }
    }

    pub fn set_active_output_device(&mut self, device_id: impl Into<String>) {
        self.active_output_device = Some(device_id.into());
        self.emit(RuntimeEvent::HardwareDeviceChanged {
            device_id: self.active_output_device.clone(),
        });
    }

    pub fn set_active_plugin_sandboxes(&mut self, count: u32) {
        self.diagnostics.active_plugin_sandboxes = count;
        self.recompute_prework_service_policy_snapshot();
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

    pub fn record_recovery_cycle(
        &mut self,
        sandbox_id: impl Into<String>,
        intent: RecoveryRestartIntent,
        stop_reason: StopReason,
        processing_epoch: Option<u64>,
    ) {
        self.emit(RuntimeEvent::RecoveryCycle {
            sandbox_id: sandbox_id.into(),
            intent,
            stop_reason,
            processing_epoch,
        });
    }

    pub fn record_plugin_sandbox_lifecycle(
        &mut self,
        sandbox_id: impl Into<String>,
        stage: PluginSandboxLifecycleStage,
        processing_epoch: Option<u64>,
    ) {
        self.emit(RuntimeEvent::PluginSandboxLifecycle {
            sandbox_id: sandbox_id.into(),
            stage,
            processing_epoch,
        });
    }

    pub fn record_plugin_sandbox_transport(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        region_id: impl Into<String>,
        stage: PluginSandboxTransportStage,
        processing_epoch: Option<u64>,
        detail: Option<String>,
    ) {
        let sandbox_id = sandbox_id.into();
        let lease_id = lease_id.into();
        let region_id = region_id.into();
        match stage {
            PluginSandboxTransportStage::Attached => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::AttachActive,
                );
            }
            PluginSandboxTransportStage::DetachRequested => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::DetachRequested,
                );
            }
            PluginSandboxTransportStage::DetachFault => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::DetachFaulted,
                );
            }
            PluginSandboxTransportStage::Detached => {
                self.transport_concurrency.mark_session_state(
                    sandbox_id.as_str(),
                    lease_id.as_str(),
                    region_id.as_str(),
                    TransportSessionState::Detached,
                );
            }
        }
        self.recompute_prework_service_policy_snapshot();
        self.emit(RuntimeEvent::PluginSandboxTransport {
            sandbox_id,
            lease_id,
            region_id,
            stage,
            processing_epoch,
            detail,
        });
    }

    pub fn record_heartbeat_cycle(
        &mut self,
        sandbox_id: impl Into<String>,
        stage: HeartbeatCycleStage,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
    ) {
        self.emit(RuntimeEvent::HeartbeatCycle {
            sandbox_id: sandbox_id.into(),
            stage,
            processing_epoch,
            block_sequence,
        });
    }

    pub fn record_block_dispatch(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        stage: BlockDispatchStage,
        completion_state: Option<CompletionState>,
    ) {
        self.emit(RuntimeEvent::BlockDispatch {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            frame_count,
            stage,
            completion_state,
        });
    }

    pub fn record_broker_invalidation(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: Option<u64>,
        stage: BrokerInvalidationStage,
        reason: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::BrokerInvalidation {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            stage,
            reason: reason.into(),
        });
    }

    pub fn record_completion_slot_transition(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: u64,
        stage: CompletionSlotStage,
    ) {
        self.emit(RuntimeEvent::CompletionSlotTransition {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            stage,
        });
    }

    pub fn record_broker_failure(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
        stage: BrokerFailureStage,
        detail: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::BrokerFailure {
            sandbox_id: sandbox_id.into(),
            lease_id,
            processing_epoch,
            block_sequence,
            stage,
            detail: detail.into(),
        });
    }

    pub fn record_sandbox_operation_failure(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        operation: impl Into<String>,
        error_kind: impl Into<String>,
        stage: SandboxOperationFailureStage,
        detail: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::SandboxOperationFailure {
            sandbox_id: sandbox_id.into(),
            lease_id,
            processing_epoch,
            operation: operation.into(),
            error_kind: error_kind.into(),
            stage,
            detail: detail.into(),
        });
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

    pub fn process_engine_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<RuntimeEngineBlockResult, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before processing engine blocks",
            ));
        }
        let context = self.build_engine_execution_context(processing_epoch, block_sequence);
        let result = self.engine.process_block(context, buffer)?;
        self.advance_engine_transport(result.output.frames().0 as i64);
        Ok(result)
    }

    pub fn prepare_engine_prework_for_block(
        &mut self,
        processing_epoch: u64,
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<bool, RuntimeError> {
        self.prepare_engine_prework_for_block_with_future_state(
            processing_epoch,
            target_block_sequence,
            admitted_from_block_sequence,
            buffer,
            None,
            None,
        )
    }

    pub fn prepare_engine_prework_for_block_with_future_state(
        &mut self,
        processing_epoch: u64,
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
        parameter_epoch_override: Option<u64>,
        transport_override: Option<TransportProjection>,
    ) -> Result<bool, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before preparing engine prework",
            ));
        }
        let context = self.build_engine_execution_context_with_overrides(
            processing_epoch,
            target_block_sequence,
            parameter_epoch_override,
            transport_override,
        );
        self.engine
            .admit_prework_for_block(context, admitted_from_block_sequence, buffer)
    }

    pub fn prepare_engine_prework_window(
        &mut self,
        processing_epoch: u64,
        targets: Vec<RuntimePreworkWindowTarget>,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before preparing engine prework",
            ));
        }
        if targets.is_empty() {
            return Ok(0);
        }

        let mut sorted_targets = targets;
        sorted_targets.sort_by_key(|target| target.target_block_sequence);
        let current_block_sequence = sorted_targets
            .iter()
            .map(|target| target.admitted_from_block_sequence)
            .min()
            .unwrap_or(0);

        let planned_sequences: Vec<u64> = sorted_targets
            .iter()
            .map(|target| target.target_block_sequence)
            .collect();
        self.engine.retire_prework_entries_matching(
            |cache| {
                cache.source_block_sequence >= current_block_sequence
                    && !planned_sequences.contains(&cache.source_block_sequence)
            },
            RuntimePreworkInvalidationReason::PlanningWindowRevised,
        );
        self.engine.pending_prework_targets.clear();

        let mut admitted = 0usize;
        for target in sorted_targets {
            let context = self.build_engine_execution_context_with_overrides(
                processing_epoch,
                target.target_block_sequence,
                target.parameter_epoch_override,
                target.transport_override,
            );
            if self.engine.admit_prework_for_block(
                context,
                target.admitted_from_block_sequence,
                target.buffer,
            )? {
                admitted = admitted.saturating_add(1);
            }
        }
        self.engine.update_prework_queue_snapshot(
            Some(current_block_sequence),
            self.engine.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        Ok(admitted)
    }

    fn service_pending_prework_cycle(
        &mut self,
        processing_epoch: u64,
        budget: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> Result<usize, RuntimeError> {
        if budget == 0 || !self.control.configured {
            return Ok(0);
        }
        let targets = self
            .engine
            .take_pending_prework_targets(budget, max_backlog_class);
        if targets.is_empty() {
            return Ok(0);
        }

        let mut admitted = 0usize;
        for target in targets {
            self.engine.record_last_serviced_pending_target(&target);
            if self.prepare_engine_prework_for_block_with_future_state(
                processing_epoch,
                target.target_block_sequence,
                target.admitted_from_block_sequence,
                target.buffer,
                target.parameter_epoch_override,
                target.transport_override,
            )? {
                admitted = admitted.saturating_add(1);
            }
        }
        Ok(admitted)
    }

    pub fn prime_engine_prework_window_with_forecast(
        &mut self,
        processing_epoch: u64,
        current_block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> Result<usize, RuntimeError> {
        self.reconcile_prework_window_with_forecast(current_block_sequence, policy);
        self.service_prework_lane_with_policy(processing_epoch, 1, policy.prepare_budget_per_cycle)
    }

    fn reconcile_prework_window_with_forecast(
        &mut self,
        current_block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> usize {
        let desired_count = policy.target_window_blocks;
        let target_block_sequences =
            self.plan_prework_window_block_sequences(current_block_sequence, desired_count);
        if target_block_sequences.is_empty() {
            self.engine.update_prework_queue_snapshot(
                Some(current_block_sequence),
                self.engine.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
            );
            return 0;
        }

        let targets = target_block_sequences
            .into_iter()
            .map(|target_block_sequence| RuntimePreworkWindowTarget {
                target_block_sequence,
                admitted_from_block_sequence: current_block_sequence,
                buffer: synthetic_stereo_block(
                    self.config.sample_rate,
                    FrameCount(self.config.graph.block_size),
                    target_block_sequence.saturating_add(policy.buffer_seed_offset),
                ),
                parameter_epoch_override: Some(
                    self.forecast_parameter_batch_for_block(target_block_sequence, policy)
                        .epoch,
                ),
                transport_override: Some(
                    self.forecast_transport_projection_for_block(target_block_sequence, policy),
                ),
            })
            .collect::<Vec<_>>();
        let graph_id = self
            .engine
            .graph
            .as_ref()
            .map(|graph| graph.graph_id().to_string());
        self.engine.reconcile_pending_prework_targets(
            &targets,
            graph_id.as_deref(),
            self.projection_epoch,
            self.latest_parameter_epoch,
            self.applied_transport,
            self.config.graph.block_size,
        );
        targets.len()
    }

    fn service_prework_lane_with_policy(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
        budget_per_cycle: usize,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured {
            return Ok(0);
        }
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
            || !self.engine.snapshot.prework_cache_enabled
        {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        if !self.control.running {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        if self.engine.pending_prework_targets.is_empty() {
            self.reconcile_prework_service_state(Some(processing_epoch));
            return Ok(0);
        }
        self.recompute_prework_service_policy_snapshot();
        let pressure = self.engine.snapshot.prework_service_pressure;
        let semantic_policy = self.engine.snapshot.prework_service_semantic_policy;
        if self.engine.snapshot.prework_service_plugin_gate_active {
            self.engine
                .record_prework_service_yield(processing_epoch, cycles, budget_per_cycle);
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Yielding,
                Some(processing_epoch),
            );
            return Ok(0);
        }
        let (effective_cycles, effective_budget_per_cycle, max_backlog_class) = match pressure {
            RuntimePreworkServicePressure::Normal => (
                cycles,
                budget_per_cycle,
                RuntimePreworkBacklogClass::Deferred,
            ),
            RuntimePreworkServicePressure::Elevated => match semantic_policy {
                RuntimePreworkServiceSemanticPolicy::Balanced => (
                    cycles.min(1),
                    budget_per_cycle.min(1),
                    RuntimePreworkBacklogClass::NearTerm,
                ),
                RuntimePreworkServiceSemanticPolicy::PluginConstrained => (
                    cycles.min(1),
                    budget_per_cycle.min(1),
                    RuntimePreworkBacklogClass::Immediate,
                ),
                RuntimePreworkServiceSemanticPolicy::LatencyFocused => (
                    cycles.min(1),
                    budget_per_cycle.min(2),
                    RuntimePreworkBacklogClass::Deferred,
                ),
            },
            RuntimePreworkServicePressure::Critical => {
                (0, 0, RuntimePreworkBacklogClass::Immediate)
            }
        };
        if pressure == RuntimePreworkServicePressure::Critical {
            self.engine
                .record_prework_service_yield(processing_epoch, cycles, budget_per_cycle);
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Yielding,
                Some(processing_epoch),
            );
            return Ok(0);
        }
        self.engine.record_prework_service_request(
            cycles,
            effective_cycles,
            budget_per_cycle,
            effective_budget_per_cycle,
        );
        if effective_cycles == 0 || effective_budget_per_cycle == 0 {
            self.engine.transition_prework_service_state(
                RuntimePreworkServiceState::Starved,
                Some(processing_epoch),
            );
            return Ok(0);
        }

        self.engine.transition_prework_service_state(
            RuntimePreworkServiceState::Servicing,
            Some(processing_epoch),
        );
        let mut total_prepared = 0usize;
        let mut executed_cycles = 0usize;
        for _ in 0..effective_cycles {
            executed_cycles = executed_cycles.saturating_add(1);
            total_prepared = total_prepared.saturating_add(self.service_pending_prework_cycle(
                processing_epoch,
                effective_budget_per_cycle,
                max_backlog_class,
            )?);
            if self.engine.pending_prework_targets.is_empty() {
                break;
            }
        }
        self.engine.record_prework_service_cycle(
            processing_epoch,
            executed_cycles,
            budget_per_cycle,
            total_prepared,
        );
        if !self.engine.pending_prework_targets.is_empty() && total_prepared == 0 {
            if pressure == RuntimePreworkServicePressure::Elevated {
                self.engine.record_prework_service_yield(
                    processing_epoch,
                    cycles,
                    budget_per_cycle,
                );
                self.engine.transition_prework_service_state(
                    RuntimePreworkServiceState::Yielding,
                    Some(processing_epoch),
                );
            } else {
                self.engine.transition_prework_service_state(
                    RuntimePreworkServiceState::Starved,
                    Some(processing_epoch),
                );
            }
        } else {
            self.reconcile_prework_service_state(Some(processing_epoch));
        }
        Ok(total_prepared)
    }

    pub fn forecast_transport_projection_for_block(
        &self,
        block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> TransportProjection {
        let loop_length_blocks = policy.transport_loop_length_blocks.max(1);
        let loop_end_samples = (self
            .config
            .graph
            .block_size
            .saturating_mul(loop_length_blocks)) as i64;
        let timeline_position_samples = ((block_sequence as i64)
            .saturating_mul(self.config.graph.block_size as i64))
        .rem_euclid(loop_end_samples);
        TransportProjection {
            playing: policy.transport_playing,
            timeline_position_samples,
            tempo_bpm: policy.transport_tempo_bpm,
            loop_state: Some(crate::interfaces::LoopRegion {
                start_samples: 0,
                end_samples: loop_end_samples,
            }),
        }
    }

    pub fn forecast_parameter_batch_for_block(
        &self,
        block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> ParameterBatch {
        let cycle_length = policy.parameter_cycle_length.max(1);
        let denominator = cycle_length.saturating_sub(1).max(1) as f32;
        ParameterBatch {
            epoch: self
                .projection_epoch
                .saturating_add(block_sequence)
                .saturating_add(1),
            events: vec![crate::interfaces::ParameterEvent {
                target: policy.parameter_target.clone(),
                normalized_value: ((block_sequence % cycle_length) as f32) / denominator,
            }],
        }
    }

    pub fn apply_forecast_state_for_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Result<usize, RuntimeError> {
        let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime prework forecast policy must be set before applying forecast state",
            )
        })?;
        self.apply_transport_projection(
            self.forecast_transport_projection_for_block(block_sequence, &policy),
        )?;
        self.apply_parameter_batch(
            self.forecast_parameter_batch_for_block(block_sequence, &policy),
        )?;
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            return Ok(0);
        }
        let _ = processing_epoch;
        Ok(self.reconcile_prework_window_with_forecast(block_sequence, &policy))
    }

    pub fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError> {
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            return Ok(0);
        }
        let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime prework forecast policy must be set before servicing the prework lane",
            )
        })?;
        self.service_prework_lane_with_policy(
            processing_epoch,
            cycles,
            policy.prepare_budget_per_cycle,
        )
    }

    pub fn next_planned_prework_block_sequence(
        &self,
        after_block_sequence: Option<u64>,
    ) -> Option<u64> {
        self.engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let target = cache.source_block_sequence;
                match after_block_sequence {
                    Some(after) if target <= after => None,
                    _ => Some(target),
                }
            })
            .min()
    }

    pub fn plan_prework_window_block_sequences(
        &mut self,
        current_block_sequence: u64,
        desired_count: usize,
    ) -> Vec<u64> {
        if !self.engine.snapshot.prework_cache_enabled {
            return Vec::new();
        }

        let mut retained_sequences = self
            .engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let target = cache.source_block_sequence;
                (target > current_block_sequence).then_some(target)
            })
            .collect::<Vec<_>>();
        retained_sequences.sort_unstable();
        retained_sequences.dedup();

        if retained_sequences.len() > desired_count {
            retained_sequences.truncate(desired_count);
            self.engine.retire_prework_entries_matching(
                |cache| {
                    cache.source_block_sequence > current_block_sequence
                        && !retained_sequences.contains(&cache.source_block_sequence)
                },
                RuntimePreworkInvalidationReason::PlanningWindowRevised,
            );
        } else if desired_count == 0 {
            self.engine.retire_prework_entries_matching(
                |cache| cache.source_block_sequence > current_block_sequence,
                RuntimePreworkInvalidationReason::PlanningWindowRevised,
            );
            retained_sequences.clear();
        }

        while retained_sequences.len() < desired_count {
            retained_sequences.push(self.allocate_block_sequence());
        }
        retained_sequences
    }

    fn build_engine_execution_context(
        &self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> GraphExecutionContext {
        self.build_engine_execution_context_with_overrides(
            processing_epoch,
            block_sequence,
            None,
            None,
        )
    }

    fn build_engine_execution_context_with_overrides(
        &self,
        processing_epoch: u64,
        block_sequence: u64,
        parameter_epoch_override: Option<u64>,
        transport_override: Option<TransportProjection>,
    ) -> GraphExecutionContext {
        let transport = transport_override.or(self.applied_transport);
        GraphExecutionContext {
            processing_epoch,
            block_sequence,
            projection_epoch: self.projection_epoch,
            parameter_epoch: parameter_epoch_override.unwrap_or(self.latest_parameter_epoch),
            configured_block_size: self.config.graph.block_size,
            anticipative_enabled: self.anticipative_enabled,
            transport_playing: transport.map(|t| t.playing).unwrap_or(false),
            transport_tempo_bpm: transport.map(|t| t.tempo_bpm).unwrap_or(0.0),
            timeline_position_samples: transport.map(|t| t.timeline_position_samples).unwrap_or(0),
        }
    }

    fn advance_engine_transport(&mut self, frame_count: i64) {
        let Some(mut transport) = self.applied_transport else {
            return;
        };
        if !transport.playing || frame_count <= 0 {
            return;
        }

        let advanced = transport
            .timeline_position_samples
            .saturating_add(frame_count);
        transport.timeline_position_samples = if let Some(loop_region) = transport.loop_state {
            let loop_start = loop_region.start_samples;
            let loop_end = loop_region.end_samples;
            if loop_end > loop_start && advanced >= loop_end {
                let loop_len = loop_end.saturating_sub(loop_start);
                loop_start.saturating_add((advanced - loop_start).rem_euclid(loop_len))
            } else {
                advanced
            }
        } else {
            advanced
        };
        self.applied_transport = Some(transport);
    }

    pub fn allocate_block_sequence(&mut self) -> u64 {
        self.timeline.allocate_block_sequence()
    }

    pub fn record_block_sequence(
        &mut self,
        sandbox_id: impl Into<String>,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) {
        let sandbox_id = sandbox_id.into();
        if let Some(rollover) = self.timeline.record_block_sequence(
            &sandbox_id,
            processing_epoch,
            lease_id,
            block_sequence,
        ) {
            self.emit(RuntimeEvent::LeaseRollover {
                sandbox_id: rollover.sandbox_id,
                previous_lease_id: rollover.previous_lease_id,
                lease_id: rollover.lease_id,
                processing_epoch: rollover.processing_epoch,
                first_block_sequence: rollover.first_block_sequence,
            });
        }
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

    pub fn begin_transport_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let snapshot = self.transport_concurrency.begin_session(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            transport_session_provenance(intent),
            None,
            None,
            None,
        )?;
        self.recompute_prework_service_policy_snapshot();
        Ok(snapshot)
    }

    pub fn begin_transport_session_with_metadata(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        self.begin_transport_session_with_metadata_for_epoch(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            None,
            transport_session_provenance(intent),
            backing_path,
            total_bytes,
        )
    }

    pub fn begin_transport_session_with_metadata_for_epoch(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        attach_processing_epoch: Option<u64>,
        provenance: TransportSessionProvenance,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let snapshot = self.transport_concurrency.begin_session(
            sandbox_id,
            lease_id,
            region_id,
            intent,
            provenance,
            attach_processing_epoch,
            backing_path,
            total_bytes,
        )?;
        self.recompute_prework_service_policy_snapshot();
        Ok(snapshot)
    }

    pub fn enqueue_lingering_cleanup_work(
        &mut self,
        sandbox_id: &str,
        mode: LingeringCleanupMode,
        trigger: LingeringCleanupTrigger,
        processing_epoch: u64,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> Option<LingeringCleanupQueueReceipt> {
        self.transport_concurrency.enqueue_cleanup_work(
            sandbox_id,
            mode,
            trigger,
            0,
            processing_epoch,
            None,
            exclude_lease_id,
            exclude_region_id,
        )
    }

    pub fn dequeue_lingering_cleanup_work_for_sandbox(
        &mut self,
        sandbox_id: &str,
        current_processing_epoch: u64,
    ) -> Option<crate::interfaces::LingeringCleanupPlan> {
        self.transport_concurrency
            .dequeue_cleanup_work_for_sandbox(sandbox_id, current_processing_epoch)
    }

    pub fn record_lingering_cleanup_failure(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        error: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency.record_cleanup_failure(
            sandbox_id,
            lease_id,
            region_id,
            mode,
            processing_epoch,
            error,
        );
        if matches!(mode, LingeringCleanupMode::BestEffortPostStart) {
            let retry_count = self
                .transport_concurrency
                .cleanup_attempt_count(sandbox_id, lease_id, region_id);
            let cleanup_wave = self
                .transport_concurrency
                .cleanup_wave_for_session(sandbox_id, lease_id, region_id);
            let _ = self.transport_concurrency.enqueue_cleanup_work(
                sandbox_id,
                mode,
                LingeringCleanupTrigger::DeferredRetry,
                retry_count,
                processing_epoch,
                cleanup_wave,
                Some(lease_id),
                Some(region_id),
            );
        }
        self.transport_concurrency.snapshot()
    }

    pub fn clear_lingering_cleanup_in_progress(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency
            .clear_cleanup_in_progress(sandbox_id, lease_id, region_id)
    }

    pub fn complete_lingering_cleanup_success(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency
            .clear_cleanup_in_progress(sandbox_id, lease_id, region_id);
        let snapshot = self
            .transport_concurrency
            .end_session(sandbox_id, lease_id, region_id);
        self.recompute_prework_service_policy_snapshot();
        snapshot
    }

    pub fn end_transport_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        let snapshot = self
            .transport_concurrency
            .end_session(sandbox_id, lease_id, region_id);
        self.recompute_prework_service_policy_snapshot();
        snapshot
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
        self.engine
            .invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        self.reconcile_prework_forecast_mode_state()?;
        self.engine.refresh_planning(self.anticipative_enabled);
        self.recompute_prework_service_policy_snapshot();
        self.safe_mode_enabled = request.realtime_safe_mode;
        self.control.configured = true;
        self.control.running = false;
        self.engine
            .set_prework_service_pressure(RuntimePreworkServicePressure::Normal);
        self.control.configure_count = self.control.configure_count.saturating_add(1);
        self.control.last_reconfigure = Some(request);
        self.timeline.reset();
        self.automation.reset();
        self.transport_concurrency.reset();
        self.readiness = RuntimeReadiness::Starting;
        self.refresh_runtime_state();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        self.reconcile_prework_service_state(None);
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
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        self.reconcile_prework_service_state(None);
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

        self.engine
            .invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeStopped);
        self.readiness = RuntimeReadiness::Stopped;
        self.control.running = false;
        self.engine
            .set_prework_service_pressure(RuntimePreworkServicePressure::Normal);
        self.control.stop_count = self.control.stop_count.saturating_add(1);
        self.control.last_stop_reason = Some(reason);
        self.reconcile_prework_service_state(None);
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
    fn set_prework_service_pressure(
        &mut self,
        pressure: RuntimePreworkServicePressure,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        self.engine.set_prework_service_pressure(pressure);
        self.recompute_prework_service_policy_snapshot();
        self.reconcile_prework_service_state(None);
        Ok(())
    }

    fn set_prework_forecast_mode(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError> {
        self.set_prework_forecast_mode_state(mode)
    }

    fn set_prework_forecast_profile(
        &mut self,
        selection: RuntimePreworkForecastProfileSelection,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        self.set_prework_forecast_profile_internal(
            Some(selection),
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection),
        );
        self.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::ExplicitProfile,
        );
        self.reconcile_prework_forecast_mode_state()?;
        self.invalidate_prework_for_forecast_plan_change_if_needed(
            previous_requested_mode,
            previous_effective_mode,
            previous_profile,
            previous_profile_source,
            previous_policy,
        )?;
        Ok(())
    }

    fn set_prework_forecast_policy(
        &mut self,
        policy: RuntimePreworkForecastPolicy,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        self.set_prework_forecast_profile_internal(
            None,
            Some(RuntimePreworkForecastProfileSource::RawPolicyOverride),
        );
        self.set_prework_forecast_policy_internal(Some(policy));
        self.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::RawPolicyOverride,
        );
        self.reconcile_prework_forecast_mode_state()?;
        self.invalidate_prework_for_forecast_plan_change_if_needed(
            previous_requested_mode,
            previous_effective_mode,
            previous_profile,
            previous_profile_source,
            previous_policy,
        )?;
        Ok(())
    }

    fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError> {
        SignalRuntime::service_prework_lane(self, processing_epoch, cycles)
    }

    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: PluginBackedNodeBindingProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.require_configured()?;
        self.engine
            .apply_plugin_backed_node_bindings(&projection, self.anticipative_enabled)?;
        self.recompute_prework_service_policy_snapshot();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

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
        self.engine
            .apply_graph_projection(&projection, self.anticipative_enabled)?;
        self.recompute_prework_service_policy_snapshot();
        self.applied_graph = Some(projection);
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
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

        let current_ready_block = self.timeline.next_block_sequence.saturating_sub(1);
        self.engine.retire_prework_entries_matching(
            |cache| {
                cache.source_block_sequence <= current_ready_block
                    && (cache.transport.playing != projection.playing
                        || cache.transport.tempo_bpm != projection.tempo_bpm
                        || cache.transport.timeline_position_samples
                            != projection.timeline_position_samples)
            },
            RuntimePreworkInvalidationReason::TransportChanged,
        );
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
        if !batch.events.is_empty() {
            let current_ready_block = self.timeline.next_block_sequence.saturating_sub(1);
            self.engine.retire_prework_entries_matching(
                |cache| {
                    cache.source_block_sequence <= current_ready_block
                        && cache.parameter_epoch != batch.epoch
                },
                RuntimePreworkInvalidationReason::ParameterBatchApplied,
            );
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

    fn get_engine_block_snapshot(&self) -> RuntimeEngineBlockSnapshot {
        self.engine.snapshot()
    }

    fn get_transport_concurrency_snapshot(&self) -> RuntimeTransportConcurrencySnapshot {
        self.transport_concurrency.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimeConfig, RuntimeProfile, SignalRuntime};
    use crate::interfaces::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        GraphNodeProjection, GraphProjection, HandshakeRequest, HeartbeatCycleStage,
        LingeringCleanupMode, LingeringCleanupTrigger, ParameterBatch, ParameterEvent,
        PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage,
        PluginSandboxTransportStage, RecoveryRestartIntent, RestartRequest, RuntimeConfigRequest,
        RuntimeErrorKind, RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink,
        RuntimeLifecycleApi, RuntimeObservationApi, RuntimeObservationReport,
        RuntimePreworkBacklogClass, RuntimePreworkCacheState, RuntimePreworkForecastMode,
        RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
        RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
        RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason,
        RuntimePreworkRetirementReason, RuntimePreworkServicePressure,
        RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
        RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness,
        RuntimeSupervisorReport, RuntimeWatchdogTrigger, SafeModeRequest,
        SandboxOperationFailureStage, ScheduleProjection, StopReason, TransportAttachIntent,
        TransportProjection, TransportSessionProvenance, WatchdogRestartRecord,
    };
    use signal_graph::{synthetic_stereo_block, GraphNodeExecutionClass, GraphStageSpec};
    use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
    use signal_plugin::{CompletionState, ParameterAutomationSummary};
    use signal_primitives::{FrameCount, SampleRate};

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
        handshake_and_configure_with_anticipative(runtime, true);
    }

    fn handshake_and_configure_with_anticipative(
        runtime: &mut SignalRuntime,
        anticipative_enabled: bool,
    ) {
        runtime
            .handshake(HandshakeRequest {
                client_version: "runtime-test".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .unwrap();
        let mut request = RuntimeConfigRequest::new(48_000, 256);
        request.anticipative_enabled = anticipative_enabled;
        runtime.configure(request).unwrap();
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
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first_sequence);

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
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first);
        let second = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", second);
        let third = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 2, "lease-b", third);

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
    fn runtime_executes_applied_graph_block_and_updates_snapshot() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:test".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "input".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![
                            GraphStageSpec::Gain { linear: 0.5 },
                            GraphStageSpec::Bias { amount: 0.2 },
                        ],
                    },
                    GraphNodeProjection {
                        node_id: "output".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 96,
                tempo_bpm: 120.0,
                loop_state: Some(crate::interfaces::LoopRegion {
                    start_samples: 64,
                    end_samples: 128,
                }),
            })
            .unwrap();
        runtime
            .apply_parameter_batch(ParameterBatch {
                epoch: runtime.projection_epoch(),
                events: vec![ParameterEvent {
                    target: "engine.runtime.test".into(),
                    normalized_value: 0.5,
                }],
            })
            .unwrap();

        let result = runtime
            .process_engine_block(
                1,
                42,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 3),
            )
            .unwrap();

        assert_eq!(
            result.snapshot.graph_id.as_deref(),
            Some("graph:runtime:test")
        );
        assert_eq!(result.snapshot.node_count, 2);
        assert_eq!(result.snapshot.stateful_node_count, 1);
        assert_eq!(result.snapshot.latency_node_count, 1);
        assert!(result.snapshot.anticipative_planning_enabled);
        assert_eq!(result.snapshot.inline_realtime_node_count, 1);
        assert_eq!(result.snapshot.stateful_realtime_node_count, 0);
        assert_eq!(result.snapshot.anticipative_eligible_node_count, 1);
        assert_eq!(result.snapshot.phase_count, 2);
        assert_eq!(result.snapshot.anticipative_phase_count, 1);
        assert_eq!(result.snapshot.lane_count, 2);
        assert_eq!(result.snapshot.anticipative_lane_count, 1);
        assert_eq!(
            result.snapshot.lane_order,
            vec![
                signal_graph::GraphExecutionLane::Anticipative,
                signal_graph::GraphExecutionLane::Realtime,
            ]
        );
        assert_eq!(result.snapshot.dispatch_count, 2);
        assert_eq!(result.snapshot.dispatch_boundary_count, 1);
        assert_eq!(
            result.snapshot.dispatch_order,
            vec![
                signal_graph::GraphExecutionLane::Anticipative,
                signal_graph::GraphExecutionLane::Realtime,
            ]
        );
        assert_eq!(result.snapshot.prepared_dispatch_count, 1);
        assert_eq!(result.snapshot.realtime_dispatch_count, 1);
        assert_eq!(result.snapshot.dispatch_handoff_count, 1);
        assert!(result.snapshot.prework_cache_enabled);
        assert_eq!(
            result.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Admitted
        );
        assert_eq!(result.snapshot.prework_cache_admissions, 1);
        assert_eq!(result.snapshot.prework_cache_consumptions, 0);
        assert_eq!(result.snapshot.prework_cache_hits, 0);
        assert_eq!(result.snapshot.prework_cache_misses, 1);
        assert_eq!(result.snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(result.snapshot.prework_cache_retirement_count, 0);
        assert_eq!(
            result.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(result.snapshot.prework_cache_block_freshness_window, 2);
        assert_eq!(
            result.snapshot.prework_cache_remaining_valid_blocks,
            Some(2)
        );
        assert!(!result.snapshot.last_prework_cache_hit);
        assert_eq!(
            result.snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ParameterBatchApplied)
        );
        assert_eq!(
            result.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            result.snapshot.prework_cache_valid_until_block_sequence,
            Some(44)
        );
        assert_eq!(
            result.snapshot.last_prework_source_processing_epoch,
            Some(1)
        );
        assert_eq!(result.snapshot.last_prework_source_block_sequence, Some(42));
        assert_eq!(
            result.snapshot.last_prework_admission_processing_epoch,
            Some(1)
        );
        assert_eq!(
            result.snapshot.last_prework_admission_block_sequence,
            Some(42)
        );
        assert_eq!(
            result.snapshot.last_prework_consumption_processing_epoch,
            None
        );
        assert_eq!(
            result.snapshot.last_prework_consumption_block_sequence,
            None
        );
        assert_eq!(
            result.snapshot.phase_order,
            vec![
                signal_graph::GraphNodePlanningGroup::InlineRealtime,
                signal_graph::GraphNodePlanningGroup::AnticipativeEligible,
            ]
        );
        assert_eq!(result.snapshot.planned_nodes.len(), 2);
        assert_eq!(result.snapshot.stage_count, 3);
        assert_eq!(result.snapshot.total_latency_samples, 16);
        assert_eq!(result.snapshot.max_node_latency_samples, 16);
        assert_eq!(result.snapshot.processed_blocks, 1);
        assert_eq!(result.snapshot.last_processing_epoch, Some(1));
        assert_eq!(result.snapshot.last_block_sequence, Some(42));
        assert_eq!(result.snapshot.last_frame_count, 8);
        assert_eq!(result.snapshot.last_channel_count, 2);
        assert!(result.snapshot.last_prework_output_peak.is_some());
        assert_eq!(
            result.snapshot.last_prework_output_peak,
            result.snapshot.last_realtime_input_peak
        );
        assert!(result.snapshot.last_output_peak.unwrap_or_default() <= 0.7);
        assert!(result.snapshot.last_output_rms.unwrap_or_default() > 0.0);
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            Some(1)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.parameter_epoch),
            Some(1)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.anticipative_enabled),
            Some(true)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            Some(true)
        );
        assert_eq!(
            result
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(96)
        );
        assert!(result.output.samples().first().is_some());
        assert_eq!(
            runtime
                .applied_transport
                .map(|transport| transport.timeline_position_samples),
            Some(104)
        );

        let observation =
            RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
        assert_eq!(
            observation.engine_block_snapshot.graph_id.as_deref(),
            Some("graph:runtime:test")
        );
        assert_eq!(observation.engine_block_snapshot.node_count, 2);
        assert_eq!(observation.engine_block_snapshot.stateful_node_count, 1);
        assert!(
            observation
                .engine_block_snapshot
                .anticipative_planning_enabled
        );
        assert_eq!(
            observation.engine_block_snapshot.inline_realtime_node_count,
            1
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .stateful_realtime_node_count,
            0
        );
        assert_eq!(observation.engine_block_snapshot.phase_count, 2);
        assert_eq!(
            observation.engine_block_snapshot.anticipative_phase_count,
            1
        );
        assert_eq!(observation.engine_block_snapshot.lane_count, 2);
        assert_eq!(observation.engine_block_snapshot.anticipative_lane_count, 1);
        assert_eq!(observation.engine_block_snapshot.dispatch_count, 2);
        assert_eq!(observation.engine_block_snapshot.dispatch_boundary_count, 1);
        assert_eq!(observation.engine_block_snapshot.prepared_dispatch_count, 1);
        assert_eq!(observation.engine_block_snapshot.realtime_dispatch_count, 1);
        assert_eq!(observation.engine_block_snapshot.dispatch_handoff_count, 1);
        assert!(observation.engine_block_snapshot.prework_cache_enabled);
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_state,
            RuntimePreworkCacheState::Admitted
        );
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_admissions,
            1
        );
        assert_eq!(
            observation.engine_block_snapshot.prework_cache_consumptions,
            0
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(observation.engine_block_snapshot.prework_cache_hits, 0);
        assert_eq!(observation.engine_block_snapshot.prework_cache_misses, 1);
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_retirement_count,
            0
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_invalidation_count,
            0
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .prework_cache_valid_until_block_sequence,
            Some(44)
        );
        assert_eq!(
            observation
                .engine_block_snapshot
                .anticipative_eligible_node_count,
            1
        );
        assert_eq!(observation.engine_block_snapshot.processed_blocks, 1);
        assert_eq!(
            observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            Some(120.0)
        );
    }

    #[test]
    fn runtime_replans_graph_when_anticipative_mode_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:planning".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "input".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "drive".into(),
                        execution_class: GraphNodeExecutionClass::Stateful,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::TanhDrive { drive: 1.4 }],
                    },
                    GraphNodeProjection {
                        node_id: "output".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 32,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.75 }],
                    },
                ],
            })
            .unwrap();

        let initial = runtime.get_engine_block_snapshot();
        assert!(initial.anticipative_planning_enabled);
        assert_eq!(initial.inline_realtime_node_count, 1);
        assert_eq!(initial.stateful_realtime_node_count, 1);
        assert_eq!(initial.anticipative_eligible_node_count, 1);
        assert_eq!(initial.prepared_dispatch_count, 1);
        assert_eq!(initial.realtime_dispatch_count, 1);
        assert_eq!(initial.dispatch_handoff_count, 1);
        assert!(initial.prework_cache_enabled);
        assert_eq!(initial.prework_cache_state, RuntimePreworkCacheState::Empty);
        assert_eq!(
            initial.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Empty
        );
        assert_eq!(initial.prework_cache_admissions, 0);
        assert_eq!(initial.prework_cache_consumptions, 0);
        assert_eq!(initial.prework_cache_hits, 0);
        assert_eq!(initial.prework_cache_misses, 0);
        assert_eq!(initial.prework_cache_invalidation_count, 0);
        assert_eq!(initial.prework_cache_retirement_count, 0);

        let mut request = RuntimeConfigRequest::new(48_000, 256);
        request.anticipative_enabled = false;
        runtime.configure(request).unwrap();

        let replanned = runtime.get_engine_block_snapshot();
        assert!(!replanned.anticipative_planning_enabled);
        assert_eq!(replanned.inline_realtime_node_count, 1);
        assert_eq!(replanned.stateful_realtime_node_count, 2);
        assert_eq!(replanned.anticipative_eligible_node_count, 0);
        assert_eq!(replanned.phase_count, 2);
        assert_eq!(replanned.anticipative_phase_count, 0);
        assert_eq!(replanned.lane_count, 1);
        assert_eq!(replanned.anticipative_lane_count, 0);
        assert_eq!(
            replanned.lane_order,
            vec![signal_graph::GraphExecutionLane::Realtime]
        );
        assert_eq!(replanned.dispatch_count, 1);
        assert_eq!(replanned.dispatch_boundary_count, 0);
        assert_eq!(replanned.prepared_dispatch_count, 0);
        assert_eq!(replanned.realtime_dispatch_count, 1);
        assert_eq!(replanned.dispatch_handoff_count, 0);
        assert!(!replanned.prework_cache_enabled);
        assert_eq!(
            replanned.prework_cache_state,
            RuntimePreworkCacheState::Disabled
        );
        assert_eq!(
            replanned.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Disabled
        );
        assert_eq!(replanned.prework_cache_admissions, 0);
        assert_eq!(replanned.prework_cache_consumptions, 0);
        assert_eq!(replanned.prework_cache_valid_until_processing_epoch, None);
        assert_eq!(
            replanned.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
        );
        assert_eq!(replanned.prework_cache_invalidation_count, 0);
        assert_eq!(replanned.prework_cache_retirement_count, 0);
        assert_eq!(
            replanned.dispatch_order,
            vec![signal_graph::GraphExecutionLane::Realtime]
        );
        assert_eq!(
            replanned.phase_order,
            vec![
                signal_graph::GraphNodePlanningGroup::InlineRealtime,
                signal_graph::GraphNodePlanningGroup::StatefulRealtime,
            ]
        );
        assert_eq!(replanned.planned_nodes.len(), 3);
        assert_eq!(
            replanned
                .planned_nodes
                .iter()
                .map(|node| (node.node_id.as_str(), format!("{:?}", node.group)))
                .collect::<Vec<_>>(),
            vec![
                ("input", "InlineRealtime".into()),
                ("drive", "StatefulRealtime".into()),
                ("output", "StatefulRealtime".into()),
            ]
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
    fn runtime_reuses_prework_cache_for_matching_adjacent_block() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:cache".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 11);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        let second = runtime.process_engine_block(2, 2, block).unwrap();

        assert_eq!(first.snapshot.prework_cache_hits, 0);
        assert_eq!(first.snapshot.prework_cache_misses, 1);
        assert_eq!(
            first.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(first.snapshot.prework_cache_admissions, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(first.snapshot.prework_cache_queued_admissions, 0);
        assert_eq!(first.snapshot.prework_cache_queued_consumptions, 0);
        assert_eq!(
            first.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(first.snapshot.prework_cache_remaining_valid_blocks, Some(2));
        assert!(!first.snapshot.last_prework_cache_hit);
        assert_eq!(
            first.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            first.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            first.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            first.snapshot.prework_cache_valid_until_block_sequence,
            Some(3)
        );
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(second.snapshot.prework_cache_misses, 1);
        assert_eq!(
            second.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(second.snapshot.prework_cache_admissions, 1);
        assert_eq!(second.snapshot.prework_cache_consumptions, 2);
        assert_eq!(second.snapshot.prework_cache_queued_admissions, 0);
        assert_eq!(second.snapshot.prework_cache_queued_consumptions, 1);
        assert_eq!(
            second.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Expiring
        );
        assert_eq!(
            second.snapshot.prework_cache_remaining_valid_blocks,
            Some(1)
        );
        assert!(second.snapshot.last_prework_cache_hit);
        assert_eq!(
            second.snapshot.last_prework_source_processing_epoch,
            Some(1)
        );
        assert_eq!(second.snapshot.last_prework_source_block_sequence, Some(1));
        assert_eq!(
            second.snapshot.last_prework_admission_processing_epoch,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_admission_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_consumption_processing_epoch,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            second.snapshot.prework_cache_valid_until_processing_epoch,
            Some(2)
        );
        assert_eq!(
            second.snapshot.prework_cache_valid_until_block_sequence,
            Some(3)
        );
        assert_eq!(second.snapshot.prepared_dispatch_count, 1);
        assert_eq!(second.snapshot.realtime_dispatch_count, 1);
    }

    #[test]
    fn runtime_consumes_primed_prework_for_the_next_block() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        let next_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 12);
        let next_batch = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(3),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                normalized_value: 0.5,
            }],
        };
        let next_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 72,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                next_block.clone(),
                Some(next_batch.epoch),
                Some(next_transport),
            )
            .unwrap());

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_admissions, 1);
        assert_eq!(primed.prework_cache_queued_admissions, 1);
        assert_eq!(primed.last_prework_admission_block_sequence, Some(2));
        assert_eq!(primed.last_prework_admitted_from_block_sequence, Some(1));

        runtime.apply_parameter_batch(next_batch).unwrap();
        runtime.apply_transport_projection(next_transport).unwrap();
        let consumed = runtime.process_engine_block(1, 2, next_block).unwrap();
        assert_eq!(consumed.snapshot.prework_cache_hits, 1);
        assert_eq!(consumed.snapshot.prework_cache_admissions, 1);
        assert_eq!(consumed.snapshot.prework_cache_consumptions, 1);
        assert_eq!(consumed.snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(consumed.snapshot.prework_cache_queued_consumptions, 1);
        assert!(consumed.snapshot.last_prework_cache_hit);
        assert_eq!(consumed.snapshot.last_prework_invalidation_reason, None);
        assert_eq!(
            consumed.snapshot.last_prework_admitted_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            consumed.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
        assert_eq!(
            consumed.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            consumed
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(72)
        );
        assert_eq!(
            consumed
                .snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            Some(120.0)
        );
    }

    #[test]
    fn runtime_prework_queue_consumes_multiple_future_blocks_in_order() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-pipeline".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        let block2 = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 12);
        let block3 = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 13);
        let batch2 = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(3),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                normalized_value: 0.5,
            }],
        };
        let batch3 = ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(4),
            events: vec![ParameterEvent {
                target: "engine.local.drive".into(),
                normalized_value: 0.65,
            }],
        };
        let transport2 = TransportProjection {
            playing: true,
            timeline_position_samples: 72,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let transport3 = TransportProjection {
            playing: true,
            timeline_position_samples: 80,
            tempo_bpm: 120.0,
            loop_state: None,
        };

        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                block2.clone(),
                Some(batch2.epoch),
                Some(transport2),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                3,
                1,
                block3.clone(),
                Some(batch3.epoch),
                Some(transport3),
            )
            .unwrap());

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_queue_capacity, 3);
        assert_eq!(primed.prework_cache_queue_depth, 2);
        assert_eq!(primed.prework_cache_peak_queue_depth, 2);
        assert_eq!(primed.prework_cache_queued_admissions, 2);
        assert_eq!(primed.last_prework_admission_block_sequence, Some(3));

        runtime.apply_parameter_batch(batch2).unwrap();
        runtime.apply_transport_projection(transport2).unwrap();
        let second = runtime.process_engine_block(1, 2, block2).unwrap();
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(second.snapshot.prework_cache_queued_consumptions, 1);
        assert_eq!(second.snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            second.snapshot.last_prework_consumption_block_sequence,
            Some(2)
        );
        assert_eq!(
            second.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );

        runtime.apply_parameter_batch(batch3).unwrap();
        runtime.apply_transport_projection(transport3).unwrap();
        let third = runtime.process_engine_block(1, 3, block3).unwrap();
        assert_eq!(third.snapshot.prework_cache_hits, 2);
        assert_eq!(third.snapshot.prework_cache_queued_consumptions, 2);
        assert_eq!(third.snapshot.prework_cache_queue_depth, 1);
        assert_eq!(
            third.snapshot.last_prework_consumption_block_sequence,
            Some(3)
        );
        assert_eq!(
            third.snapshot.last_prework_consumed_from_block_sequence,
            Some(1)
        );
    }

    #[test]
    fn runtime_prework_queue_evicts_oldest_future_entry_when_capacity_is_exceeded() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-eviction".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 64,
                tempo_bpm: 120.0,
                loop_state: None,
            })
            .unwrap();

        for offset in 0..4 {
            let target_block_sequence = 2 + offset;
            let block = synthetic_stereo_block(
                SampleRate(48_000),
                FrameCount(8),
                12 + target_block_sequence,
            );
            let batch_epoch = runtime
                .projection_epoch()
                .saturating_add(3)
                .saturating_add(offset);
            let transport = TransportProjection {
                playing: true,
                timeline_position_samples: 72 + (offset as i64 * 8),
                tempo_bpm: 120.0,
                loop_state: None,
            };
            assert!(runtime
                .prepare_engine_prework_for_block_with_future_state(
                    1,
                    target_block_sequence,
                    1,
                    block,
                    Some(batch_epoch),
                    Some(transport),
                )
                .unwrap());
        }

        let primed = runtime.get_engine_block_snapshot();
        assert_eq!(primed.prework_cache_queue_capacity, 3);
        assert_eq!(primed.prework_cache_queue_depth, 3);
        assert_eq!(primed.prework_cache_peak_queue_depth, 3);
        assert_eq!(primed.prework_cache_queued_admissions, 4);
        assert_eq!(primed.prework_cache_invalidation_count, 1);
        assert_eq!(
            primed.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::QueueCapacityExceeded)
        );
        assert_eq!(
            primed.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::QueueCapacityExceeded)
        );
        assert_eq!(primed.last_prework_retired_unconsumed, Some(true));
    }

    #[test]
    fn runtime_reuses_existing_future_queue_entry_when_target_state_matches() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-reuse".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let transport = TransportProjection {
            playing: true,
            timeline_position_samples: 96,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 41);
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                block.clone(),
                Some(9),
                Some(transport),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                2,
                2,
                2,
                block,
                Some(9),
                Some(transport),
            )
            .unwrap());

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_admissions, 1);
        assert_eq!(snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(1));
    }

    #[test]
    fn runtime_replaces_future_queue_entry_when_target_state_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:queued-prework-replace".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let first_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 96,
            tempo_bpm: 120.0,
            loop_state: None,
        };
        let replacement_transport = TransportProjection {
            playing: true,
            timeline_position_samples: 104,
            tempo_bpm: 121.0,
            loop_state: None,
        };
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                2,
                1,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 42),
                Some(9),
                Some(first_transport),
            )
            .unwrap());
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                2,
                2,
                2,
                synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 43),
                Some(10),
                Some(replacement_transport),
            )
            .unwrap());

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_admissions, 2);
        assert_eq!(snapshot.prework_cache_queued_admissions, 1);
        assert_eq!(snapshot.prework_cache_invalidation_count, 1);
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::SupersededByAdmission)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::SupersededByAdmission)
        );
        assert_eq!(snapshot.last_prework_retired_unconsumed, Some(true));
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(2));
    }

    #[test]
    fn runtime_planning_window_retires_future_entries_not_in_revised_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:prework-window-revision".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let targets = vec![
            RuntimePreworkWindowTarget {
                target_block_sequence: 2,
                admitted_from_block_sequence: 1,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 52),
                parameter_epoch_override: Some(9),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 96,
                    tempo_bpm: 120.0,
                    loop_state: None,
                }),
            },
            RuntimePreworkWindowTarget {
                target_block_sequence: 3,
                admitted_from_block_sequence: 1,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 53),
                parameter_epoch_override: Some(10),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 104,
                    tempo_bpm: 121.0,
                    loop_state: None,
                }),
            },
        ];
        assert_eq!(
            runtime
                .prepare_engine_prework_window(1, targets)
                .expect("initial planning window"),
            2
        );

        let revised_targets = vec![RuntimePreworkWindowTarget {
            target_block_sequence: 3,
            admitted_from_block_sequence: 2,
            buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 53),
            parameter_epoch_override: Some(10),
            transport_override: Some(TransportProjection {
                playing: true,
                timeline_position_samples: 104,
                tempo_bpm: 121.0,
                loop_state: None,
            }),
        }];
        assert_eq!(
            runtime
                .prepare_engine_prework_window(2, revised_targets)
                .expect("revised planning window"),
            1
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_window_target_count, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![3]
        );
        assert_eq!(snapshot.prework_cache_invalidation_count, 1);
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::PlanningWindowRevised)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::PlanningWindowRevised)
        );
        assert_eq!(snapshot.last_prework_retired_unconsumed, Some(true));
    }

    #[test]
    fn runtime_planning_window_reuses_existing_future_sequences_and_allocates_missing() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:prework-window-sequences".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        let first_future_sequence = runtime.allocate_block_sequence();
        let second_future_sequence = runtime.allocate_block_sequence();

        let initial_targets = vec![
            RuntimePreworkWindowTarget {
                target_block_sequence: first_future_sequence,
                admitted_from_block_sequence: current_sequence,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 61),
                parameter_epoch_override: Some(9),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 96,
                    tempo_bpm: 120.0,
                    loop_state: None,
                }),
            },
            RuntimePreworkWindowTarget {
                target_block_sequence: second_future_sequence,
                admitted_from_block_sequence: current_sequence,
                buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 62),
                parameter_epoch_override: Some(10),
                transport_override: Some(TransportProjection {
                    playing: true,
                    timeline_position_samples: 104,
                    tempo_bpm: 121.0,
                    loop_state: None,
                }),
            },
        ];
        runtime
            .prepare_engine_prework_window(1, initial_targets)
            .expect("initial planning window");

        let revised_sequences =
            runtime.plan_prework_window_block_sequences(first_future_sequence, 2);
        assert_eq!(
            revised_sequences,
            vec![
                second_future_sequence,
                second_future_sequence.saturating_add(1)
            ]
        );
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_cache_window_target_count, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![first_future_sequence, second_future_sequence]
        );
    }

    #[test]
    fn runtime_primes_prework_window_from_forecast_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-prework".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 2,
            prepare_budget_per_cycle: 2,
            buffer_seed_offset: 17,
            transport_playing: true,
            transport_tempo_bpm: 122.0,
            transport_loop_length_blocks: 24,
            parameter_target: "engine.server.balance".into(),
            parameter_cycle_length: 6,
        };

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime forecast window");
        assert_eq!(admitted, 2);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2]
        );
        assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(0));

        let transport = runtime.forecast_transport_projection_for_block(2, &policy);
        assert_eq!(transport.tempo_bpm, 122.0);
        assert_eq!(transport.timeline_position_samples, 512);

        let batch = runtime.forecast_parameter_batch_for_block(2, &policy);
        assert_eq!(batch.epoch, 4);
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].target, "engine.server.balance");
        assert!((batch.events[0].normalized_value - 0.4).abs() < 1.0e-6);
    }

    #[test]
    fn runtime_forecast_policy_limits_prework_window_depth() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-limit".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let policy = RuntimePreworkForecastPolicy {
            target_window_blocks: 1,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        };

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
            .expect("prime limited forecast window");
        assert_eq!(admitted, 1);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_cache_window_target_count, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1]
        );
    }

    #[test]
    fn runtime_forecast_runner_leaves_pending_targets_when_budget_is_smaller_than_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded raw forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-budget".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        assert_eq!(runtime.engine.prework_queue.len(), 1);
        assert_eq!(runtime.engine.pending_prework_targets.len(), 2);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(snapshot.prework_pending_target_count, 2);
        assert_eq!(snapshot.prework_cache_window_target_count, 3);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![0, 1, 2]
        );

        let serviced_once = runtime
            .service_prework_lane(1, 1)
            .expect("service pending prework once");
        assert_eq!(serviced_once, 1);
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_pending_target_count, 1);
        assert!(snapshot.prework_service_cycle_count >= 1);
        assert!(snapshot.prework_service_prepared_targets >= 1);
        assert_eq!(snapshot.last_prework_service_processing_epoch, Some(1));
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(1));
        assert_eq!(snapshot.last_prework_service_prepared_targets, 1);

        let serviced_again = runtime
            .service_prework_lane(1, 2)
            .expect("service pending prework until idle");
        assert_eq!(serviced_again, 1);
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 3);
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert!(snapshot.prework_service_cycle_count >= 3);
        assert!(snapshot.prework_service_prepared_targets >= 2);
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_prepared_targets, 1);
    }

    #[test]
    fn runtime_prework_service_lane_enters_starved_state_when_budget_is_zero() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 0,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set zero-budget forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-starved".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let paused = runtime.get_engine_block_snapshot();
        assert_eq!(
            paused.prework_service_state,
            RuntimePreworkServiceState::Paused
        );
        assert_eq!(paused.prework_pending_target_count, 3);

        runtime.start().expect("start runtime");
        runtime
            .service_prework_lane(1, 1)
            .expect("service prework lane with zero effective budget");
        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Starved
        );
        assert_eq!(snapshot.prework_cache_queue_depth, 0);
        assert_eq!(snapshot.prework_pending_target_count, 3);
        assert!(snapshot.prework_service_starvation_count >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_resumes_after_start() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-resume".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let paused = runtime.get_engine_block_snapshot();
        assert_eq!(
            paused.prework_service_state,
            RuntimePreworkServiceState::Paused
        );
        assert!(paused.prework_pending_target_count > 0);

        runtime.start().expect("start runtime");

        let resumed = runtime.get_engine_block_snapshot();
        assert!(matches!(
            resumed.prework_service_state,
            RuntimePreworkServiceState::Pending | RuntimePreworkServiceState::Idle
        ));
        assert!(resumed.prework_service_pause_count >= 1);
        assert!(resumed.prework_service_resume_count >= 1);
        assert!(resumed.prework_service_prepared_targets >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_yields_under_critical_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 3,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-critical".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Critical)
            .expect("set critical prework pressure");
        runtime
            .service_prework_lane(1, 3)
            .expect("service prework lane under critical pressure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Critical
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 0);
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(0)
        );
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_prework_service_lane_throttles_under_elevated_pressure() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set bounded forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-elevated".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");
        runtime
            .service_prework_lane(1, 3)
            .expect("service prework lane under elevated pressure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated
        );
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert!(snapshot.last_prework_service_effective_cycles <= 1);
        assert!(matches!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(0 | 1)
        ));
        assert!(snapshot.prework_service_throttle_count >= 1);
        assert!(
            snapshot.prework_service_prepared_targets >= 1
                || snapshot.prework_service_yield_count >= 1
        );
    }

    #[test]
    fn runtime_elevated_pressure_preserves_deferred_prework_targets() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set elevated forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-runner-backlog-classes".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");
        runtime
            .service_prework_lane(2, 3)
            .expect("service elevated lane second cycle");
        runtime
            .service_prework_lane(3, 3)
            .expect("service elevated lane third cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert_eq!(snapshot.prework_pending_immediate_target_count, 0);
        assert_eq!(snapshot.prework_pending_near_term_target_count, 0);
        assert!(snapshot.prework_pending_deferred_target_count > 0);
        assert_eq!(
            snapshot.prework_pending_target_count,
            snapshot.prework_pending_deferred_target_count
        );
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_latency_focused_graph_expands_elevated_pressure_service_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set latency-focused forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:latency-focused-prework-priority".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");
        runtime
            .service_prework_lane(2, 3)
            .expect("service elevated lane second cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(2)
        );
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert_eq!(
            snapshot.last_prework_serviced_backlog_class,
            Some(RuntimePreworkBacklogClass::Deferred)
        );
        assert!(snapshot.prework_service_throttle_count >= 1);
    }

    #[test]
    fn runtime_plugin_backed_graph_constrains_elevated_pressure_service_scope() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 4,
                prepare_budget_per_cycle: 3,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-constrained forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-constrained-prework-priority".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane first cycle");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.plugin_backed_node_count, 1);
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_throttle_count >= 1);
    }

    #[test]
    fn runtime_plugin_backed_policy_tracks_active_plugin_sandbox_count() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-policy-tracking".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
        runtime.set_active_plugin_sandboxes(1);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        runtime.set_active_plugin_sandboxes(0);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::LatencyFocused
        );
    }

    #[test]
    fn runtime_plugin_constrained_lane_yields_when_multiple_plugin_sandboxes_are_active() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 1,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-constrained forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-gate".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime.set_active_plugin_sandboxes(2);
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated lane");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert_eq!(snapshot.prework_service_active_plugin_sandboxes, 2);
        assert!(snapshot.prework_service_plugin_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_pending_target_count > 0);
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_plugin_bindings_project_into_snapshot_and_track_bound_sessions() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-bindings".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-bindings".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-bound".into(),
                }],
            })
            .expect("apply plugin-backed bindings");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 1);
        assert!(snapshot.planned_nodes.iter().any(|node| {
            node.node_id == "plugin" && node.plugin_sandbox_id.as_deref() == Some("sandbox-bound")
        }));

        runtime
            .begin_transport_session(
                "sandbox-bound",
                "lease-bound",
                "region-bound",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin bound transport session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-bound",
            "lease-bound",
            "region-bound",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
    }

    #[test]
    fn runtime_degraded_bound_plugin_session_gates_prework_lane() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 6,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 32,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            })
            .expect("set plugin-bound forecast policy");
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:plugin-bound-gate".into(),
                node_count: 3,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "plugin".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 96,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();
        runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:runtime:plugin-bound-gate".into(),
                bindings: vec![PluginBackedNodeBinding {
                    node_id: "plugin".into(),
                    sandbox_id: "sandbox-a".into(),
                }],
            })
            .expect("apply plugin-backed bindings");
        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .expect("begin transport session");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::Attached,
            Some(1),
            None,
        );
        runtime.start().expect("start runtime");
        runtime
            .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
            .expect("set elevated prework pressure");
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late detach fault".into()),
        );

        runtime
            .service_prework_lane(1, 3)
            .expect("service elevated prework lane");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_service_semantic_policy,
            RuntimePreworkServiceSemanticPolicy::PluginConstrained
        );
        assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 0);
        assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 1);
        assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
        assert!(snapshot.prework_service_plugin_gate_active);
        assert_eq!(
            snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
        );
        assert!(snapshot.prework_service_yield_count >= 1);
    }

    #[test]
    fn runtime_apply_forecast_state_primes_window_and_applies_current_block_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-advance".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: None,
            })
            .expect("set prework forecast profile");

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("apply forecast state");
        assert_eq!(admitted, 2);

        assert_eq!(
            runtime
                .applied_transport
                .as_ref()
                .map(|transport| transport.tempo_bpm),
            Some(122.0)
        );
        assert_eq!(
            runtime
                .applied_transport
                .as_ref()
                .map(|transport| transport.timeline_position_samples),
            Some(0)
        );
        assert_eq!(
            runtime.latest_parameter_epoch,
            runtime
                .forecast_parameter_batch_for_block(
                    current_sequence,
                    &SignalRuntime::prework_forecast_policy_for_profile(
                        RuntimePreworkForecastProfileSelection {
                            profile: RuntimePreworkForecastProfile::Server,
                            target_window_blocks_override: None,
                        },
                    ),
                )
                .epoch
        );

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2]
        );
        assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(0));
    }

    #[test]
    fn runtime_reconfigure_uses_role_default_after_requested_mode_is_reset() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("set prework forecast profile");
        assert_eq!(
            runtime.get_engine_block_snapshot().prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::RuntimeRoleDefault)
            .expect("reset requested mode to runtime role default");

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-role-default-after-reconfigure".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("forecast apply should use runtime-role default");
        assert_eq!(admitted, 2);
    }

    #[test]
    fn runtime_selects_forecast_profile_with_target_window_override() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-profile-override".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(4),
            })
            .expect("set prework forecast profile");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            Some(4)
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(4)
        );

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("apply forecast state");
        assert_eq!(admitted, 4);
    }

    #[test]
    fn runtime_configure_seeds_default_forecast_profile_from_runtime_role() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            None
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );
    }

    #[test]
    fn runtime_can_disable_and_restore_role_default_forecast_mode() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-mode-toggle".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::Disabled)
            .expect("disable prework forecast mode");
        let disabled_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            disabled_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            disabled_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert!(disabled_snapshot.prework_forecast_policy_configured);
        assert_eq!(
            disabled_snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::PlanningDisabled)
        );

        let block_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, block_sequence)
            .expect("apply forecast state while disabled");
        assert_eq!(admitted, 0);

        runtime
            .set_prework_forecast_mode(RuntimePreworkForecastMode::RuntimeRoleDefault)
            .expect("restore role-default forecast mode");
        let restored_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            restored_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            restored_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_policy_target_window_blocks,
            Some(2)
        );

        let next_block_sequence = runtime.allocate_block_sequence();
        let restored_admitted = runtime
            .apply_forecast_state_for_block(2, next_block_sequence)
            .expect("apply forecast state after restore");
        assert_eq!(restored_admitted, 2);
    }

    #[test]
    fn runtime_retires_queued_prework_when_forecast_profile_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-plan-change".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(admitted, 2);
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("switch explicit profile");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 3);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2, 3]
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ForecastPlanChanged)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::ForecastPlanChanged)
        );
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
    }

    #[test]
    fn runtime_rebuilds_missing_queued_prework_when_forecast_window_expands() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-expand".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(3),
            })
            .expect("expand local forecast window");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 3);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1, 2, 3]
        );
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(snapshot.prework_cache_retirement_count, 0);
    }

    #[test]
    fn runtime_preserves_compatible_queued_prework_when_forecast_mode_changes_but_plan_matches() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-plan-compatible".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        let admitted = runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(admitted, 2);

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: None,
            })
            .expect("switch to explicit profile with matching plan");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(snapshot.prework_cache_invalidation_count, 0);
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
    }

    #[test]
    fn runtime_selectively_trims_queued_prework_when_forecast_window_shrinks() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:forecast-window-shrink".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime local role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(1),
            })
            .expect("shrink local forecast window");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 1);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences,
            vec![1]
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ForecastPlanChanged)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::ForecastPlanChanged)
        );
    }

    #[test]
    fn runtime_configure_with_anticipative_disabled_enters_disabled_forecast_mode() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, false);

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert!(snapshot.prework_forecast_policy_configured);
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Local,
                target_window_blocks_override: Some(3),
            })
            .expect("store explicit profile while anticipative planning is off");
        let explicit_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            explicit_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            explicit_snapshot.prework_forecast_profile_target_window_override,
            Some(3)
        );
    }

    #[test]
    fn runtime_retires_queued_prework_when_effective_mode_drops_to_disabled() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:disable-retire".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let current_sequence = runtime.allocate_block_sequence();
        runtime
            .apply_forecast_state_for_block(1, current_sequence)
            .expect("prime role-default prework");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        let mut disabled_request = RuntimeConfigRequest::new(48_000, 256);
        disabled_request.anticipative_enabled = false;
        runtime
            .configure(disabled_request)
            .expect("disable anticipative");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 0);
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
        );
        assert_eq!(
            snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::RuntimeReconfigured)
        );
    }

    #[test]
    fn runtime_apply_graph_projection_primes_prework_window_from_stored_forecast_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);

        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:auto-prime-on-graph-apply".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .expect("apply graph projection");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences.len(),
            2
        );
        assert!(
            snapshot.prework_cache_window_target_block_sequences[0]
                < snapshot.prework_cache_window_target_block_sequences[1]
        );
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
    }

    #[test]
    fn runtime_start_rebuilds_prework_window_after_runtime_stop() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:restart-rebuild".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .expect("apply graph projection");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            2
        );

        runtime.start().expect("start runtime");
        runtime
            .stop(StopReason::UserRequested)
            .expect("stop runtime");
        assert_eq!(
            runtime
                .get_engine_block_snapshot()
                .prework_cache_queue_depth,
            0
        );

        runtime.start().expect("restart runtime");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(snapshot.prework_cache_queue_depth, 2);
        assert_eq!(
            snapshot.prework_cache_window_target_block_sequences.len(),
            2
        );
        assert!(
            snapshot.prework_cache_window_target_block_sequences[0]
                < snapshot.prework_cache_window_target_block_sequences[1]
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RuntimeRoleDefault
        );
    }

    #[test]
    fn runtime_reconfigure_preserves_explicit_forecast_profile_request() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(4),
            })
            .expect("set explicit forecast profile");

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_target_window_override,
            Some(4)
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(4)
        );
    }

    #[test]
    fn runtime_restores_requested_explicit_forecast_mode_after_anticipative_reenable() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
                profile: RuntimePreworkForecastProfile::Server,
                target_window_blocks_override: Some(3),
            })
            .expect("set explicit forecast profile");

        let mut disabled_request = RuntimeConfigRequest::new(48_000, 256);
        disabled_request.anticipative_enabled = false;
        runtime
            .configure(disabled_request)
            .expect("disable anticipative");

        let disabled_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            disabled_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            disabled_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::Disabled
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("reenable anticipative");

        let restored_snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            restored_snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            restored_snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::ExplicitProfile
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile,
            Some(RuntimePreworkForecastProfile::Server)
        );
        assert_eq!(
            restored_snapshot.prework_forecast_profile_target_window_override,
            Some(3)
        );
    }

    #[test]
    fn runtime_restart_preserves_raw_forecast_override_request() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime.start().expect("start runtime");

        runtime
            .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
                target_window_blocks: 5,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 11,
                transport_playing: true,
                transport_tempo_bpm: 130.0,
                transport_loop_length_blocks: 12,
                parameter_target: "engine.test.raw".into(),
                parameter_cycle_length: 9,
            })
            .expect("set raw forecast policy");

        runtime
            .restart(RestartRequest { reconfigure: None })
            .expect("restart without reconfigure");

        let snapshot = runtime.get_engine_block_snapshot();
        assert_eq!(
            snapshot.prework_forecast_requested_mode,
            RuntimePreworkForecastMode::RawPolicyOverride
        );
        assert_eq!(
            snapshot.prework_forecast_mode,
            RuntimePreworkForecastMode::RawPolicyOverride
        );
        assert_eq!(
            snapshot.prework_forecast_policy_target_window_blocks,
            Some(5)
        );
        assert_eq!(
            snapshot.prework_forecast_profile_source,
            Some(RuntimePreworkForecastProfileSource::RawPolicyOverride)
        );
    }

    #[test]
    fn runtime_prework_cache_expires_by_block_sequence_window() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:block-expiry".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 16,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 31);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        let second = runtime.process_engine_block(1, 2, block.clone()).unwrap();
        let third = runtime.process_engine_block(1, 3, block.clone()).unwrap();
        let fourth = runtime.process_engine_block(1, 4, block).unwrap();

        assert_eq!(first.snapshot.prework_cache_misses, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(second.snapshot.prework_cache_hits, 1);
        assert_eq!(third.snapshot.prework_cache_hits, 2);
        assert_eq!(third.snapshot.prework_cache_consumptions, 3);
        assert_eq!(
            third.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Exhausted
        );
        assert_eq!(third.snapshot.prework_cache_remaining_valid_blocks, Some(0));
        assert_eq!(
            fourth.snapshot.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::BlockSequenceExpired)
        );
        assert_eq!(
            fourth.snapshot.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::BlockSequenceExpired)
        );
        assert_eq!(fourth.snapshot.last_prework_retired_unconsumed, Some(false));
        assert_eq!(fourth.snapshot.prework_cache_retirement_count, 1);
        assert_eq!(fourth.snapshot.prework_cache_consumed_retirement_count, 1);
        assert_eq!(fourth.snapshot.prework_cache_unconsumed_retirement_count, 0);
        assert_eq!(fourth.snapshot.prework_cache_misses, 2);
        assert_eq!(
            fourth.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(fourth.snapshot.prework_cache_consumptions, 4);
        assert_eq!(
            fourth.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(
            fourth.snapshot.prework_cache_valid_until_block_sequence,
            Some(6)
        );
        assert_eq!(fourth.snapshot.last_prework_source_block_sequence, Some(4));
    }

    #[test]
    fn runtime_invalidates_prework_cache_on_parameter_and_transport_changes() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        handshake_and_configure_with_anticipative(&mut runtime, true);
        runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:runtime:invalidate".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "inline".into(),
                        execution_class: GraphNodeExecutionClass::PureTransform,
                        latency_samples: 0,
                        stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                    },
                    GraphNodeProjection {
                        node_id: "latency".into(),
                        execution_class: GraphNodeExecutionClass::LatencyBearing,
                        latency_samples: 24,
                        stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                    },
                ],
            })
            .unwrap();

        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 21);
        let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
        assert_eq!(
            first.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(first.snapshot.prework_cache_admissions, 1);
        assert_eq!(first.snapshot.prework_cache_consumptions, 1);
        assert_eq!(
            first.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );

        runtime
            .apply_parameter_batch(ParameterBatch {
                epoch: runtime.projection_epoch(),
                events: vec![ParameterEvent {
                    target: "invalidate.param".into(),
                    normalized_value: 0.25,
                }],
            })
            .unwrap();
        let after_parameter = runtime.get_engine_block_snapshot();
        assert_eq!(
            after_parameter.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        );
        assert_eq!(
            after_parameter.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::ParameterBatchApplied)
        );
        assert_eq!(after_parameter.prework_cache_invalidation_count, 1);
        assert_eq!(after_parameter.prework_cache_retirement_count, 1);
        assert_eq!(
            after_parameter.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::ParameterBatchApplied)
        );
        assert_eq!(after_parameter.last_prework_retired_unconsumed, Some(false));
        assert_eq!(after_parameter.prework_cache_unconsumed_retirement_count, 0);
        assert_eq!(after_parameter.prework_cache_consumed_retirement_count, 1);
        assert_eq!(
            after_parameter.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Invalidated
        );
        assert_eq!(
            after_parameter.prework_cache_valid_until_processing_epoch,
            None
        );

        let second = runtime.process_engine_block(2, 2, block.clone()).unwrap();
        assert_eq!(second.snapshot.prework_cache_misses, 2);
        assert!(!second.snapshot.last_prework_cache_hit);
        assert_eq!(
            second.snapshot.prework_cache_state,
            RuntimePreworkCacheState::Consumed
        );
        assert_eq!(second.snapshot.prework_cache_admissions, 2);
        assert_eq!(second.snapshot.prework_cache_consumptions, 2);
        assert_eq!(
            second.snapshot.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Fresh
        );

        runtime
            .apply_transport_projection(TransportProjection {
                playing: true,
                timeline_position_samples: 512,
                tempo_bpm: 130.0,
                loop_state: None,
            })
            .unwrap();
        let after_transport = runtime.get_engine_block_snapshot();
        assert_eq!(
            after_transport.prework_cache_state,
            RuntimePreworkCacheState::Invalidated
        );
        assert_eq!(
            after_transport.last_prework_invalidation_reason,
            Some(RuntimePreworkInvalidationReason::TransportChanged)
        );
        assert_eq!(after_transport.prework_cache_invalidation_count, 2);
        assert_eq!(after_transport.prework_cache_retirement_count, 2);
        assert_eq!(
            after_transport.last_prework_retirement_reason,
            Some(RuntimePreworkRetirementReason::TransportChanged)
        );
        assert_eq!(after_transport.last_prework_retired_unconsumed, Some(false));
        assert_eq!(after_transport.prework_cache_unconsumed_retirement_count, 0);
        assert_eq!(after_transport.prework_cache_consumed_retirement_count, 2);
        assert_eq!(
            after_transport.prework_cache_freshness_state,
            RuntimePreworkFreshnessState::Invalidated
        );
        assert_eq!(
            after_transport.prework_cache_valid_until_processing_epoch,
            None
        );
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
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::RecoveryCycle {
                sandbox_id: "sandbox-a".into(),
                intent: RecoveryRestartIntent::WatchdogRecovery,
                stop_reason: StopReason::DegradedModeRecovery,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxLifecycle {
                sandbox_id: "sandbox-a".into(),
                stage: PluginSandboxLifecycleStage::TransportAttached,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(4),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::LeaseRollover {
                sandbox_id: "sandbox-a".into(),
                previous_lease_id: "lease-3".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                first_block_sequence: 12,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerInvalidation {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: Some(12),
                stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                reason: "watchdog recovery teardown".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 12,
                stage: CompletionSlotStage::FallbackApplied,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-a".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Detached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachFault,
                processing_epoch: Some(4),
                detail: Some("broker detach fault: stale region mapping".into()),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SandboxOperationFailure {
                sandbox_id: "sandbox-a".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                operation: "processBlock".into(),
                error_kind: "resourceUnavailable".into(),
                stage: SandboxOperationFailureStage::ProcessAttach,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );

        let diagnostics = recorder.diagnostics();
        assert_eq!(diagnostics.total_events, 17);
        assert_eq!(diagnostics.supervision_update_count(), 1);
        assert_eq!(diagnostics.plugin_fault_count(), 2);
        assert_eq!(diagnostics.recovery_event_count(), 1);
        assert_eq!(diagnostics.lifecycle_event_count(), 1);
        assert_eq!(diagnostics.transport_event_count(), 4);
        assert_eq!(diagnostics.heartbeat_event_count(), 1);
        assert_eq!(diagnostics.block_dispatch_event_count(), 1);
        assert_eq!(diagnostics.lease_rollover_event_count(), 1);
        assert_eq!(diagnostics.invalidation_event_count(), 1);
        assert_eq!(diagnostics.completion_slot_event_count(), 2);
        assert_eq!(diagnostics.transport_fault_event_count(), 8);
        assert_eq!(diagnostics.broker_failure_event_count(), 1);
        assert_eq!(diagnostics.sandbox_operation_failure_event_count(), 1);
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
        assert_eq!(
            diagnostics
                .last_recovery_event()
                .map(|event| event.processing_epoch),
            Some(Some(4))
        );
        assert_eq!(
            diagnostics
                .last_lifecycle_event()
                .map(|event| event.processing_epoch),
            Some(Some(4))
        );
        assert_eq!(
            diagnostics.last_transport_event().map(|event| event.stage),
            Some(PluginSandboxTransportStage::DetachFault)
        );
        assert_eq!(
            diagnostics
                .transport_events
                .first()
                .map(|event| event.region_id.as_str()),
            Some("region-4")
        );
        assert_eq!(
            diagnostics
                .last_heartbeat_event()
                .map(|event| event.block_sequence),
            Some(Some(12))
        );
        assert_eq!(
            diagnostics
                .last_block_dispatch_event()
                .map(|event| event.completion_state),
            Some(Some(CompletionState::Completed))
        );
        assert_eq!(
            diagnostics
                .last_lease_rollover_event()
                .map(|event| event.previous_lease_id.as_str()),
            Some("lease-3")
        );
        assert_eq!(
            diagnostics
                .last_invalidation_event()
                .map(|event| event.reason.as_str()),
            Some("watchdog recovery teardown")
        );
        assert_eq!(
            diagnostics
                .last_completion_slot_event()
                .map(|event| event.stage),
            Some(CompletionSlotStage::FallbackApplied)
        );
        assert_eq!(
            diagnostics.last_transport_fault_event().map(|event| (
                event.source,
                event.stage,
                event.phase,
                event.resource
            )),
            Some((
                crate::interfaces::TransportFaultSource::SandboxOperation,
                crate::interfaces::TransportFaultStage::ProcessAttach,
                crate::interfaces::TransportFaultPhase::Dispatch,
                crate::interfaces::TransportFaultResource::SharedMemoryLease,
            ))
        );
        assert_eq!(
            diagnostics
                .last_broker_failure_event()
                .map(|event| event.stage),
            Some(BrokerFailureStage::PayloadRead)
        );
        assert_eq!(
            diagnostics
                .last_sandbox_operation_failure_event()
                .map(|event| event.stage),
            Some(SandboxOperationFailureStage::ProcessAttach)
        );
        assert!(diagnostics.render_compact().contains("plugin_faults=2"));
        assert!(diagnostics.render_compact().contains("recovery_events=1"));
        assert!(diagnostics.render_compact().contains("lifecycle_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("block_dispatch_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("lease_rollover_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("invalidation_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("completion_slot_events=2"));
        assert!(diagnostics
            .render_compact()
            .contains("transport_fault_events=8"));
        assert!(diagnostics
            .render_compact()
            .contains("broker_failure_events=1"));
        assert!(diagnostics
            .render_compact()
            .contains("sandbox_operation_failure_events=1"));

        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);
        runtime.start().unwrap();
        let first_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", first_sequence);
        let second_sequence = runtime.allocate_block_sequence();
        runtime.record_block_sequence("sandbox-a", 1, "lease-a", second_sequence);
        let report = RuntimeObservationReport::capture(&runtime, &recorder);
        assert!(report.render_compact().contains("readiness=Ready"));
        assert!(report.render_compact().contains("handshaken=true"));
        assert!(report.render_compact().contains("configures=1"));
        assert!(report.render_compact().contains("plugin_faults=2"));
        assert!(report.render_compact().contains("next_block_sequence=2"));
        assert!(report
            .render_compact()
            .contains("transport_fault_boundary=FaultAdjacentOnly"));
        assert_eq!(
            report.transport_fault_summary.boundary_mode,
            crate::interfaces::TransportFaultBoundaryMode::FaultAdjacentOnly
        );
        assert_eq!(report.transport_fault_summary.total_events, 8);
        assert_eq!(report.transport_fault_summary.host_broker_events, 4);
        assert_eq!(report.transport_fault_summary.sandbox_operation_events, 1);
        assert_eq!(report.transport_fault_summary.runtime_dispatch_events, 3);
        assert_eq!(report.transport_fault_summary.prepare_events, 0);
        assert_eq!(report.transport_fault_summary.dispatch_events, 5);
        assert_eq!(report.transport_fault_summary.teardown_events, 3);
        assert_eq!(report.transport_fault_summary.control_events, 0);
        assert_eq!(
            report.transport_fault_summary.first_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_fault_summary.last_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_fault_summary.first_block_sequence,
            Some(12)
        );
        assert_eq!(report.transport_fault_summary.last_block_sequence, Some(12));
        assert_eq!(
            report.transport_session_summary.boundary_mode,
            crate::interfaces::TransportSessionBoundaryMode::HealthyPathVisible
        );
        assert_eq!(
            report.transport_session_summary.current_state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );
        assert!(!report.transport_session_summary.currently_attached);
        assert_eq!(
            report.transport_session_summary.heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Fresh
        );
        assert_eq!(
            report.transport_session_summary.dispatch_state,
            crate::interfaces::TransportDispatchState::Completed
        );
        assert_eq!(report.transport_session_summary.attach_events, 1);
        assert_eq!(report.transport_session_summary.detach_requested_events, 1);
        assert_eq!(report.transport_session_summary.detached_events, 1);
        assert_eq!(report.transport_session_summary.detach_fault_events, 1);
        assert_eq!(
            report.transport_session_summary.heartbeat_requested_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.heartbeat_responded_events,
            1
        );
        assert_eq!(report.transport_session_summary.heartbeat_missed_events, 0);
        assert_eq!(
            report.transport_session_summary.dispatch_requested_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.dispatch_completed_events,
            1
        );
        assert_eq!(
            report.transport_session_summary.dispatch_timed_out_events,
            0
        );
        assert_eq!(
            report.transport_session_summary.first_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_session_summary.last_processing_epoch,
            Some(4)
        );
        assert_eq!(
            report.transport_session_summary.first_block_sequence,
            Some(12)
        );
        assert_eq!(
            report.transport_session_summary.last_block_sequence,
            Some(12)
        );
        assert_eq!(
            report
                .transport_session_summary
                .active_sandbox_id
                .as_deref(),
            None
        );
        assert_eq!(
            report.transport_session_summary.active_lease_id.as_deref(),
            None
        );
        assert_eq!(
            report.transport_session_summary.active_region_id.as_deref(),
            None
        );
        assert_eq!(report.transport_session_summary.active_block_sequence, None);
        assert_eq!(
            report
                .transport_session_summary
                .current_attached_session_count,
            0
        );
        assert_eq!(
            report
                .transport_session_summary
                .max_concurrent_attached_sessions,
            1
        );
        assert!(report.transport_session_summary.active_sessions.is_empty());
        assert_eq!(
            report.transport_session_summary.last_sandbox_id.as_deref(),
            Some("sandbox-a")
        );
        assert_eq!(
            report.transport_session_summary.last_lease_id.as_deref(),
            Some("lease-4")
        );
        assert_eq!(
            report.transport_session_summary.last_region_id.as_deref(),
            Some("region-4")
        );
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
        assert_eq!(supervisor.event_count(), 5);
        assert_eq!(supervisor.supervision_update_count(), 1);
        assert_eq!(supervisor.plugin_fault_count(), 2);
        assert_eq!(supervisor.recovery_event_count(), 1);
        assert_eq!(supervisor.lifecycle_event_count(), 1);
        assert_eq!(
            supervisor.last_watchdog_trigger(),
            Some(RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert!(supervisor.render_compact().contains("event_stream=5"));
        assert!(supervisor.render_compact().contains("recovery_events=1"));
        assert!(supervisor.render_compact().contains("lifecycle_events=1"));
        assert!(supervisor.render_multiline().contains("plugin_faults=2"));
        assert!(supervisor
            .render_multiline()
            .contains("recovery_sequence=["));
        assert!(supervisor
            .render_multiline()
            .contains("lifecycle_sequence=["));
        assert!(supervisor
            .render_multiline()
            .contains("sequence_segments=1"));
        assert!(supervisor
            .render_multiline()
            .contains("automation_param=4096"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_fault_boundary=FaultAdjacentOnly"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_fault_host_broker_events=4"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_boundary=HealthyPathVisible"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_attach_events=1"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_state=DetachFaulted"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_heartbeat_state=Fresh"));
        assert!(supervisor
            .render_multiline()
            .contains("transport_session_dispatch_state=Completed"));
        let json = supervisor.render_json();
        assert!(json.contains("\"readiness\":\"Ready\""));
        assert!(json.contains("\"control\":{\"handshaken\":true"));
        assert!(json.contains("\"next_block_sequence\":2"));
        assert!(json.contains("\"sequence_segments\":1"));
        assert!(json.contains("\"plugin_faults\":2"));
        assert!(json.contains("\"recovery_events\":1"));
        assert!(json.contains("\"recovery_sequence\":[{"));
        assert!(json.contains("\"intent\":\"WatchdogRecovery\""));
        assert!(json.contains("\"lifecycle_events\":1"));
        assert!(json.contains("\"lifecycle_sequence\":[{"));
        assert!(json.contains("\"stage\":\"TransportAttached\""));
        assert!(json.contains("\"transport_fault_events\":8"));
        assert!(json.contains("\"last_transport_fault\":{"));
        assert!(json.contains("\"transport_fault_sequence\":[{"));
        assert!(json.contains("\"source\":\"HostBroker\""));
        assert!(json.contains("\"source\":\"SandboxOperation\""));
        assert!(json.contains("\"source\":\"RuntimeDispatch\""));
        assert!(json.contains("\"phase\":\"Dispatch\""));
        assert!(json.contains("\"phase\":\"Teardown\""));
        assert!(json.contains("\"resource\":\"SharedMemoryPayload\""));
        assert!(json.contains("\"resource\":\"SharedMemoryLease\""));
        assert!(json.contains("\"resource\":\"CompletionSlot\""));
        assert!(json.contains("\"operation\":\"block_payload.read\""));
        assert!(json.contains("\"operation\":\"transport.detach_request\""));
        assert!(json.contains("\"operation\":\"transport.detached\""));
        assert!(json.contains("\"operation\":\"transport.detach_fault\""));
        assert!(json.contains("\"operation\":\"completion_region.invalidate\""));
        assert!(json.contains("\"operation\":\"completion_slot.timeout\""));
        assert!(json.contains("\"operation\":\"completion_slot.fallback_apply\""));
        assert!(json.contains("\"operation\":\"processBlock\""));
        assert!(json.contains("\"stage\":\"TransportDetachRequested\""));
        assert!(json.contains("\"stage\":\"TransportDetached\""));
        assert!(json.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(json.contains("\"stage\":\"CompletionSlotTimedOut\""));
        assert!(json.contains("\"stage\":\"FallbackApplied\""));
        assert!(json.contains("\"stage\":\"PayloadRead\""));
        assert!(json.contains("\"stage\":\"ProcessAttach\""));
        assert!(json.contains("\"transport_fault_summary\":{"));
        assert!(json.contains("\"boundary_mode\":\"FaultAdjacentOnly\""));
        assert!(json.contains("\"host_broker_events\":4"));
        assert!(json.contains("\"sandbox_operation_events\":1"));
        assert!(json.contains("\"runtime_dispatch_events\":3"));
        assert!(json.contains("\"dispatch_events\":5"));
        assert!(json.contains("\"teardown_events\":3"));
        assert!(json.contains("\"transport_session_summary\":{"));
        assert!(json.contains("\"boundary_mode\":\"HealthyPathVisible\""));
        assert!(json.contains("\"current_state\":\"DetachFaulted\""));
        assert!(json.contains("\"currently_attached\":false"));
        assert!(json.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(json.contains("\"dispatch_state\":\"Completed\""));
        assert!(json.contains("\"current_attached_session_count\":0"));
        assert!(json.contains("\"max_concurrent_attached_sessions\":1"));
        assert!(json.contains("\"attach_events\":1"));
        assert!(json.contains("\"detach_requested_events\":1"));
        assert!(json.contains("\"detached_events\":1"));
        assert!(json.contains("\"detach_fault_events\":1"));
        assert!(json.contains("\"heartbeat_responded_events\":1"));
        assert!(json.contains("\"dispatch_completed_events\":1"));
        assert!(json.contains("\"active_sandbox_id\":null"));
        assert!(json.contains("\"active_lease_id\":null"));
        assert!(json.contains("\"active_region_id\":null"));
        assert!(json.contains("\"active_block_sequence\":null"));
        assert!(json.contains("\"active_sessions\":[]"));
        assert!(json.contains("\"last_region_id\":\"region-4\""));
        assert!(json.contains("\"automation\":{\"parameter_id\":4096"));
    }

    #[test]
    fn transport_session_summary_tracks_concurrent_active_sessions() {
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(2),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                region_id: "region-b".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(3),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Missed,
                processing_epoch: Some(4),
                block_sequence: Some(11),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-b".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(5),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                frame_count: 512,
                stage: BlockDispatchStage::TimedOut,
                completion_state: Some(CompletionState::TimedOut),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                processing_epoch: 5,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-b".into(),
                lease_id: Some("lease-b".into()),
                processing_epoch: Some(5),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "stale shared-memory mapping".into(),
            },
        );

        let diagnostics = recorder.diagnostics();
        let summary = crate::interfaces::TransportSessionSummary::from_diagnostics(&diagnostics);
        assert_eq!(summary.current_attached_session_count, 2);
        assert_eq!(summary.max_concurrent_attached_sessions, 2);
        assert_eq!(
            summary.current_state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert!(summary.currently_attached);
        assert_eq!(summary.active_sessions.len(), 2);
        assert_eq!(summary.active_sandbox_id.as_deref(), Some("sandbox-a"));
        assert_eq!(summary.active_lease_id.as_deref(), Some("lease-a"));
        assert_eq!(summary.active_region_id.as_deref(), Some("region-a"));
        assert_eq!(summary.active_block_sequence, Some(12));
        assert_eq!(summary.active_sessions[0].sandbox_id.as_str(), "sandbox-a");
        assert_eq!(
            summary.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert!(summary.active_sessions[0].currently_attached);
        assert_eq!(
            summary.active_sessions[0].heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Missed
        );
        assert_eq!(
            summary.active_sessions[0].dispatch_state,
            crate::interfaces::TransportDispatchState::TimedOut
        );
        assert_eq!(summary.active_sessions[0].processing_epoch, Some(4));
        assert_eq!(summary.active_sessions[0].active_block_sequence, Some(11));
        assert_eq!(summary.active_sessions[0].transport_fault_count, 1);
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_source,
            Some(crate::interfaces::TransportFaultSource::RuntimeDispatch)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_stage,
            Some(crate::interfaces::TransportFaultStage::CompletionSlotTimedOut)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_phase,
            Some(crate::interfaces::TransportFaultPhase::Dispatch)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_processing_epoch,
            Some(4)
        );
        assert_eq!(
            summary.active_sessions[0].last_transport_fault_block_sequence,
            Some(11)
        );
        assert_eq!(summary.active_sessions[1].sandbox_id.as_str(), "sandbox-b");
        assert_eq!(
            summary.active_sessions[1].state,
            crate::interfaces::TransportSessionState::AttachActive
        );
        assert!(summary.active_sessions[1].currently_attached);
        assert_eq!(
            summary.active_sessions[1].heartbeat_freshness,
            crate::interfaces::TransportHeartbeatFreshness::Fresh
        );
        assert_eq!(
            summary.active_sessions[1].dispatch_state,
            crate::interfaces::TransportDispatchState::Completed
        );
        assert_eq!(summary.active_sessions[1].processing_epoch, Some(5));
        assert_eq!(summary.active_sessions[1].active_block_sequence, Some(12));
        assert_eq!(summary.active_sessions[1].transport_fault_count, 1);
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_source,
            Some(crate::interfaces::TransportFaultSource::HostBroker)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_stage,
            Some(crate::interfaces::TransportFaultStage::PayloadRead)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_phase,
            Some(crate::interfaces::TransportFaultPhase::Dispatch)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_processing_epoch,
            Some(5)
        );
        assert_eq!(
            summary.active_sessions[1].last_transport_fault_block_sequence,
            Some(12)
        );
    }

    #[test]
    fn runtime_owns_transport_session_admission_policy() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        let first = runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .unwrap();
        assert_eq!(first.current_attached_sessions, 1);
        assert_eq!(first.peak_attached_sessions, 1);
        assert_eq!(first.current_recovery_overlap_sessions, 0);
        assert_eq!(first.current_lingering_sessions, 0);
        assert_eq!(
            first.active_sessions[0].state,
            crate::interfaces::TransportSessionState::AttachActive
        );

        let steady_reject = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::SteadyState,
            )
            .unwrap_err();
        assert_eq!(steady_reject.kind, RuntimeErrorKind::ResourceUnavailable);

        let overlap = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(overlap.current_attached_sessions, 2);
        assert_eq!(overlap.peak_attached_sessions, 2);
        assert_eq!(overlap.current_recovery_overlap_sessions, 1);
        assert_eq!(overlap.peak_recovery_overlap_sessions, 1);
        assert_eq!(overlap.current_lingering_sessions, 0);

        let overlap_reject = runtime
            .begin_transport_session(
                "sandbox-c",
                "lease-c",
                "region-c",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap_err();
        assert_eq!(overlap_reject.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(overlap_reject
            .message
            .contains("recovery overlap session limit 1"));

        let snapshot = runtime.get_transport_concurrency_snapshot();
        assert_eq!(snapshot.current_attached_sessions, 2);
        assert_eq!(snapshot.peak_attached_sessions, 2);
        assert_eq!(snapshot.current_lingering_sessions, 0);
        assert_eq!(
            snapshot.last_admitted_sandbox_id.as_deref(),
            Some("sandbox-b")
        );
        assert_eq!(
            snapshot.last_rejected_sandbox_id.as_deref(),
            Some("sandbox-c")
        );
        assert!(snapshot
            .last_rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));

        let after_end = runtime.end_transport_session("sandbox-a", "lease-a", "region-a");
        assert_eq!(after_end.current_attached_sessions, 1);
        assert_eq!(after_end.current_recovery_overlap_sessions, 1);
        assert_eq!(after_end.current_lingering_sessions, 0);

        let re_admit = runtime
            .begin_transport_session(
                "sandbox-c",
                "lease-c",
                "region-c",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap_err();
        assert!(re_admit
            .message
            .contains("recovery overlap session limit 1"));

        let after_overlap_end = runtime.end_transport_session("sandbox-b", "lease-b", "region-b");
        assert_eq!(after_overlap_end.current_attached_sessions, 0);
        assert_eq!(after_overlap_end.current_recovery_overlap_sessions, 0);
        assert_eq!(after_overlap_end.current_lingering_sessions, 0);

        let re_admitted = runtime
            .begin_transport_session(
                "sandbox-c",
                "lease-c",
                "region-c",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(re_admitted.current_attached_sessions, 1);
        assert_eq!(re_admitted.current_recovery_overlap_sessions, 1);
        assert_eq!(re_admitted.current_lingering_sessions, 0);
        assert_eq!(
            re_admitted.last_admitted_sandbox_id.as_deref(),
            Some("sandbox-c")
        );

        runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .unwrap();
        let reset = runtime.get_transport_concurrency_snapshot();
        assert_eq!(reset.current_attached_sessions, 0);
        assert_eq!(reset.current_lingering_sessions, 0);
        assert!(reset.active_sessions.is_empty());
        assert_eq!(reset.peak_attached_sessions, 0);
        assert_eq!(reset.peak_lingering_sessions, 0);
    }

    #[test]
    fn runtime_tracks_lingering_transport_sessions_as_first_class_admission_state() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        runtime
            .begin_transport_session(
                "sandbox-a",
                "lease-a",
                "region-a",
                TransportAttachIntent::SteadyState,
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachRequested,
            Some(2),
            None,
        );

        let requested = runtime.get_transport_concurrency_snapshot();
        assert_eq!(requested.current_attached_sessions, 1);
        assert_eq!(requested.current_lingering_sessions, 1);
        assert_eq!(requested.peak_lingering_sessions, 1);
        assert_eq!(requested.current_detach_requested_sessions, 1);
        assert_eq!(requested.current_detach_faulted_sessions, 0);
        assert_eq!(
            requested.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );

        let steady_reject = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::SteadyState,
            )
            .unwrap_err();
        assert_eq!(steady_reject.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(steady_reject.message.contains("lingering session"));

        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-a",
            "region-a",
            PluginSandboxTransportStage::DetachFault,
            Some(2),
            Some("teardown fault".into()),
        );

        let faulted = runtime.get_transport_concurrency_snapshot();
        assert_eq!(faulted.current_attached_sessions, 1);
        assert_eq!(faulted.current_lingering_sessions, 1);
        assert_eq!(faulted.current_detach_requested_sessions, 0);
        assert_eq!(faulted.current_detach_faulted_sessions, 1);
        assert_eq!(
            faulted.active_sessions[0].state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );

        let overlap = runtime
            .begin_transport_session(
                "sandbox-b",
                "lease-b",
                "region-b",
                TransportAttachIntent::RecoveryOverlap,
            )
            .unwrap();
        assert_eq!(overlap.current_attached_sessions, 2);
        assert_eq!(overlap.current_recovery_overlap_sessions, 1);
        assert_eq!(overlap.current_lingering_sessions, 1);
        assert_eq!(overlap.current_detach_faulted_sessions, 1);
        assert_eq!(overlap.peak_lingering_sessions, 1);

        runtime.end_transport_session("sandbox-b", "lease-b", "region-b");
        runtime.end_transport_session("sandbox-a", "lease-a", "region-a");

        let cleared = runtime.get_transport_concurrency_snapshot();
        assert_eq!(cleared.current_attached_sessions, 0);
        assert_eq!(cleared.current_lingering_sessions, 0);
        assert_eq!(cleared.current_detach_requested_sessions, 0);
        assert_eq!(cleared.current_detach_faulted_sessions, 0);
    }

    #[test]
    fn runtime_orders_lingering_cleanup_candidates_by_provenance_then_attach_sequence() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        handshake_and_configure(&mut runtime);

        runtime
            .begin_transport_session_with_metadata_for_epoch(
                "sandbox-a",
                "lease-origin",
                "region-origin",
                TransportAttachIntent::SteadyState,
                Some(2),
                TransportSessionProvenance::SteadyOrigin,
                Some("/tmp/signal-origin".into()),
                Some(4096),
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-origin",
            "region-origin",
            PluginSandboxTransportStage::DetachFault,
            Some(2),
            Some("origin detach fault".into()),
        );

        runtime
            .begin_transport_session_with_metadata_for_epoch(
                "sandbox-a",
                "lease-replacement",
                "region-replacement",
                TransportAttachIntent::RecoveryOverlap,
                Some(3),
                TransportSessionProvenance::RecoveryReplacement,
                Some("/tmp/signal-replacement".into()),
                Some(8192),
            )
            .unwrap();
        runtime.record_plugin_sandbox_transport(
            "sandbox-a",
            "lease-replacement",
            "region-replacement",
            PluginSandboxTransportStage::DetachRequested,
            Some(3),
            None,
        );

        let cleanup_receipt = runtime
            .enqueue_lingering_cleanup_work(
                "sandbox-a",
                LingeringCleanupMode::StrictPreAttach,
                LingeringCleanupTrigger::RecoveryPreAttach,
                4,
                None,
                None,
            )
            .expect("cleanup work should be queued");
        let queued = runtime.get_transport_concurrency_snapshot();
        assert_eq!(queued.pending_cleanup_work_items, 1);
        assert_eq!(queued.pending_deferred_retry_work_items, 0);
        assert_eq!(queued.next_cleanup_epoch, 2);
        assert_eq!(queued.oldest_pending_cleanup_ready_epoch, Some(4));
        assert_eq!(queued.pending_cleanup_waves.len(), 1);
        assert_eq!(queued.pending_cleanup_waves[0].cleanup_wave, 1);
        assert_eq!(
            queued.pending_cleanup_waves[0].first_trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );
        assert_eq!(
            queued.pending_cleanup_waves[0].latest_trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );

        let cleanup_plan = runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 4)
            .expect("cleanup plan should dequeue");
        assert_eq!(cleanup_plan.work_id, cleanup_receipt.work_id);
        assert_eq!(cleanup_plan.cleanup_epoch, cleanup_receipt.cleanup_epoch);
        assert_eq!(cleanup_plan.cleanup_wave, cleanup_receipt.cleanup_wave);
        assert_eq!(cleanup_plan.sandbox_id, "sandbox-a");
        assert_eq!(cleanup_plan.mode, LingeringCleanupMode::StrictPreAttach);
        assert_eq!(
            cleanup_plan.trigger,
            LingeringCleanupTrigger::RecoveryPreAttach
        );
        assert_eq!(cleanup_plan.retry_count, 0);
        assert_eq!(cleanup_plan.processing_epoch, 4);
        assert_eq!(cleanup_plan.ready_at_processing_epoch, 4);
        assert_eq!(cleanup_plan.exclude_lease_id, None);
        assert_eq!(cleanup_plan.exclude_region_id, None);
        let cleanup_candidates = cleanup_plan.candidates;
        assert_eq!(cleanup_candidates.len(), 2);
        assert!(cleanup_candidates[0].attach_sequence < cleanup_candidates[1].attach_sequence);

        assert_eq!(
            cleanup_candidates[0].provenance,
            TransportSessionProvenance::SteadyOrigin
        );
        assert_eq!(cleanup_candidates[0].attach_processing_epoch, Some(2));
        assert_eq!(
            cleanup_candidates[0].state,
            crate::interfaces::TransportSessionState::DetachFaulted
        );
        assert_eq!(cleanup_candidates[0].lease_id, "lease-origin");
        assert_eq!(cleanup_candidates[0].cleanup_attempt_count, 1);
        assert_eq!(
            cleanup_candidates[0].last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(cleanup_candidates[0].last_cleanup_wave, Some(1));
        assert!(cleanup_candidates[0].cleanup_in_progress);
        assert_eq!(cleanup_candidates[0].last_cleanup_epoch, Some(4));
        assert_eq!(cleanup_candidates[0].last_cleanup_error, None);

        assert_eq!(
            cleanup_candidates[1].provenance,
            TransportSessionProvenance::RecoveryReplacement
        );
        assert_eq!(cleanup_candidates[1].attach_processing_epoch, Some(3));
        assert_eq!(
            cleanup_candidates[1].state,
            crate::interfaces::TransportSessionState::DetachRequested
        );
        assert_eq!(cleanup_candidates[1].lease_id, "lease-replacement");
        assert_eq!(cleanup_candidates[1].cleanup_attempt_count, 1);
        assert_eq!(
            cleanup_candidates[1].last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(cleanup_candidates[1].last_cleanup_wave, Some(1));
        assert!(cleanup_candidates[1].cleanup_in_progress);

        let snapshot = runtime.get_transport_concurrency_snapshot();
        assert_eq!(snapshot.active_sessions.len(), 2);
        assert!(snapshot
            .active_sessions
            .iter()
            .all(|session| session.cleanup_in_progress));

        let failed = runtime.record_lingering_cleanup_failure(
            "sandbox-a",
            "lease-origin",
            "region-origin",
            LingeringCleanupMode::StrictPreAttach,
            4,
            "cleanup failed",
        );
        let origin = failed
            .active_sessions
            .iter()
            .find(|session| session.lease_id == "lease-origin")
            .unwrap();
        assert!(!origin.cleanup_in_progress);
        assert_eq!(origin.cleanup_attempt_count, 1);
        assert_eq!(
            origin.last_cleanup_mode,
            Some(LingeringCleanupMode::StrictPreAttach)
        );
        assert_eq!(origin.last_cleanup_epoch, Some(4));
        assert_eq!(origin.last_cleanup_error.as_deref(), Some("cleanup failed"));

        let retried = runtime.record_lingering_cleanup_failure(
            "sandbox-a",
            "lease-replacement",
            "region-replacement",
            LingeringCleanupMode::BestEffortPostStart,
            5,
            "late cleanup failed",
        );
        assert_eq!(retried.pending_cleanup_work_items, 1);
        assert_eq!(retried.pending_deferred_retry_work_items, 1);
        assert_eq!(retried.next_cleanup_epoch, 3);
        assert_eq!(retried.oldest_pending_cleanup_ready_epoch, Some(6));
        assert_eq!(retried.pending_cleanup_waves.len(), 1);
        assert_eq!(retried.pending_cleanup_waves[0].cleanup_wave, 1);
        assert_eq!(
            retried.pending_cleanup_waves[0].latest_trigger,
            LingeringCleanupTrigger::DeferredRetry
        );
        assert!(runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 5)
            .is_none());
        let deferred_retry = runtime
            .dequeue_lingering_cleanup_work_for_sandbox("sandbox-a", 6)
            .expect("deferred retry should dequeue");
        assert_eq!(deferred_retry.cleanup_epoch, 2);
        assert_eq!(deferred_retry.cleanup_wave, 1);
        assert_eq!(
            deferred_retry.trigger,
            LingeringCleanupTrigger::DeferredRetry
        );
        assert_eq!(deferred_retry.retry_count, 1);
        assert_eq!(
            deferred_retry.mode,
            LingeringCleanupMode::BestEffortPostStart
        );
        assert_eq!(deferred_retry.ready_at_processing_epoch, 6);
        assert_eq!(
            deferred_retry.exclude_lease_id.as_deref(),
            Some("lease-replacement")
        );
        assert_eq!(
            deferred_retry.exclude_region_id.as_deref(),
            Some("region-replacement")
        );

        runtime
            .enqueue_lingering_cleanup_work(
                "sandbox-a",
                LingeringCleanupMode::BestEffortPostStart,
                LingeringCleanupTrigger::PostStartReconciliation,
                7,
                None,
                None,
            )
            .expect("second cleanup wave should queue");
        let next_wave = runtime.get_transport_concurrency_snapshot();
        assert_eq!(next_wave.pending_cleanup_waves.len(), 1);
        assert_eq!(next_wave.pending_cleanup_waves[0].cleanup_wave, 2);
        assert_eq!(
            next_wave.pending_cleanup_waves[0].first_trigger,
            LingeringCleanupTrigger::PostStartReconciliation
        );
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
