use signal_graph::{synthetic_stereo_block, GraphNodeExecutionClass, GraphStageSpec};
use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_ipc::{
    PluginInstanceStatePayload, PluginMessageEnvelope, PluginMessagePayload, SharedMemoryBroker,
    SharedMemoryTransportPayload,
};
use signal_plugin::{
    CompletionState, PluginFormat, PluginSandboxRequest, SandboxPolicy, SandboxWatchdogState,
    WatchdogOutcome, WatchdogTriggerReason,
};
use signal_plugin_clap::{
    classify_sandbox_failure, sandbox_failure_event, BrokeredBlockOutcome, ClapBlockProtocol,
    ClapDiscoveredPluginType, ClapPluginHostAdapter, ClapSandboxFailureStage,
    ClapSandboxLifecycleHarness,
};
use signal_primitives::FrameCount;
use signal_runtime::{
    BackendPolicyOverride, BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage,
    CompletionSlotStage, GraphNodeProjection, GraphProjection, HandshakeRequest,
    HeartbeatCycleStage, LingeringCleanupMode, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginSandboxInstanceFaultRecord,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
    RuntimeClipProcessingRegistration, RuntimeConfigRequest, RuntimeError, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeOfflineRenderExecutionProgressReceipt,
    RuntimeOfflineRenderExecutionReceipt, RuntimeOfflineRenderPurgeReceipt,
    RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderQueueResult, RuntimeOfflineRenderRequest,
    RuntimeOfflineRenderResult, RuntimePluginDiscoveredTypeRecord, RuntimePreworkServicePressure,
    RuntimeProjectionApi, RuntimeRecordingCaptureCommitReceipt,
    RuntimeRecordingCaptureStartRequest, RuntimeSupervisorApi, RuntimeSupervisorReport,
    RuntimeWarpClipRegistration, RuntimeWatchdogTrigger, SandboxOperationFailureStage,
    SignalRuntime, StopReason, TransportAttachIntent, WatchdogRestartRecord,
};

const WATCHDOG_TRIGGER_WINDOW_BLOCKS: u64 = 3;
const STEADY_STATE_BLOCKS: u64 = 8;
const SOAK_RESTART_EPISODES: u32 = 3;
const INTER_EPISODE_CONTINUITY_BLOCKS: u64 = 2;

fn runtime_plugin_discovered_type_record(
    discovered: ClapDiscoveredPluginType,
) -> RuntimePluginDiscoveredTypeRecord {
    let plugin_type_id = discovered.plugin_type_id.0;
    let descriptor = discovered.descriptor;
    let summary = format!(
        "plugin_type={} plugin_id={} format={:?} features={} io={:?} parameters={}",
        plugin_type_id,
        descriptor.plugin_id,
        descriptor.format,
        descriptor.features.len(),
        discovered.default_io_layout,
        descriptor.parameters.len(),
    );
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id,
        plugin_id: descriptor.plugin_id.clone(),
        vendor: descriptor.vendor.clone(),
        name: descriptor.name.clone(),
        format: descriptor.format,
        version: descriptor.version.clone(),
        features: descriptor.features.clone(),
        default_io_layout: discovered.default_io_layout,
        audio_bus_count: descriptor.audio_buses.len(),
        parameter_count: descriptor.parameters.len(),
        state_contract: descriptor.state_contract,
        processing_contract: descriptor.processing_contract,
        lifecycle_contract: descriptor.lifecycle_contract,
        summary,
    }
}

#[derive(Clone, Debug, Default)]
struct ServerSupervisorState {
    scans_started: u64,
    sandboxes: u64,
    restarts: u64,
    teardowns: u64,
    backend_policy: Option<BackendPolicyTier>,
    last_scan_roots: Vec<String>,
    last_sandbox_id: Option<String>,
    last_recovery_intent: Option<RecoveryRestartIntent>,
    last_stop_reason: Option<StopReason>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultInjection {
    Timeout,
    Crash,
    HeartbeatMiss,
    RecoveryDeferredTeardownFailure,
    RecoveryDeferredTeardownThenCleanup,
    RecoveryDeferredTeardownCleanupRetry,
    RecoveryTeardownFailure,
    RecoveryRestartFailure,
    RecoveryOverlapContention,
    RecoveryInterleavedFailures,
    EscalatingHeartbeatMisses { restart_episodes: u32 },
    MixedWatchdogEpisodes { restart_episodes: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecoveryFailureInjection {
    OldTransportTeardown,
    DeferredOldTransportTeardown,
    LingeringCleanupTeardown,
    ReplacementStart,
    CompetingOverlapAttach,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerPayloadSummary {
    pub event_count: usize,
    pub parameter_event_count: usize,
    pub parameter_gesture_event_count: usize,
    pub parameter_modulation_event_count: usize,
    pub note_event_count: usize,
    pub note_expression_event_count: usize,
    pub midi_event_count: usize,
    pub generated_event_bytes: u32,
    pub first_output_sample: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerExecutionSummary {
    pub control_requests: usize,
    pub control_responses: usize,
    pub heartbeat_responses: usize,
    pub processed_blocks: usize,
    pub engine_processed_blocks: usize,
    pub last_control_message: String,
    pub last_completion_state: CompletionState,
    pub last_block_sequence: u64,
    pub last_engine_graph_id: Option<String>,
    pub last_engine_output_peak: Option<f32>,
    pub last_engine_output_rms: Option<f32>,
    pub processing_epoch: u64,
    pub restart_count: u64,
    pub teardown_count: u64,
    pub last_recovery_intent: Option<RecoveryRestartIntent>,
    pub last_stop_reason: Option<StopReason>,
    pub last_plugin_state: Option<PluginSandboxInstanceStateRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerTransportSummary {
    pub sandbox_id: String,
    pub shared_memory_lease_id: String,
    pub shared_memory_region_id: String,
    pub shared_memory_path: String,
    pub shared_memory_bytes: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerFaultSummary {
    pub deadline_misses: u32,
    pub heartbeat_misses: u32,
    pub watchdog_triggered: bool,
    pub watchdog_trigger_reason: Option<WatchdogTriggerReason>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerRuntimeHostSummary {
    pub scan_roots: Vec<String>,
    pub execution: ServerExecutionSummary,
    pub transport: ServerTransportSummary,
    pub last_payload: ServerPayloadSummary,
    pub faults: ServerFaultSummary,
}

pub struct ServerRuntimeHost {
    runtime: SignalRuntime,
    broker: SharedMemoryBroker,
    supervisor: ServerSupervisorState,
    events: RuntimeEventRecorder,
}

impl ServerRuntimeHost {
    pub fn new(runtime: SignalRuntime) -> Self {
        let events = RuntimeEventRecorder::default();
        let mut runtime = runtime;
        runtime.subscribe(Box::new(events.clone()));

        Self {
            runtime,
            broker: SharedMemoryBroker::default(),
            supervisor: ServerSupervisorState::default(),
            events,
        }
    }

    fn discovered_plugins_for_scan(
        &self,
        request: &PluginScanRequest,
    ) -> Vec<RuntimePluginDiscoveredTypeRecord> {
        let include_clap =
            request.formats.is_empty() || request.formats.contains(&PluginFormat::Clap);
        if !include_clap {
            return Vec::new();
        }

        let clap = ClapPluginHostAdapter::default();
        ["plugin:clap:server", "plugin:clap:sandbox"]
            .into_iter()
            .filter_map(|plugin_type_id| clap.discover_plugin_type(plugin_type_id))
            .map(runtime_plugin_discovered_type_record)
            .collect()
    }

    pub fn boot_default(&mut self) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(None)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_timeout_recovery(&mut self) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Timeout))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_crash_recovery(&mut self) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Crash))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_heartbeat_miss_recovery(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::HeartbeatMiss))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_teardown_failure(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryTeardownFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_failure(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_then_cleanup(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownThenCleanup))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_cleanup_retry(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownCleanupRetry))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_restart_failure(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryRestartFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_overlap_contention(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryOverlapContention))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_interleaved_failures(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryInterleavedFailures))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_escalating_heartbeat_failures(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: 2,
        }))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_watchdog_soak(&mut self) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: SOAK_RESTART_EPISODES,
        }))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_mixed_watchdog_soak(
        &mut self,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::MixedWatchdogEpisodes {
            restart_episodes: SOAK_RESTART_EPISODES,
        }))
    }

    fn boot_with_fault_recovery(
        &mut self,
        fault: Option<FaultInjection>,
    ) -> Result<ServerRuntimeHostSummary, RuntimeError> {
        let mut runtime_config = RuntimeConfigRequest::new(
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size,
        );
        runtime_config.anticipative_enabled = false;
        self.runtime.handshake(HandshakeRequest {
            client_version: "signal-host-server".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })?;
        self.runtime.configure(runtime_config)?;
        let assembly = server_demo_runtime_assembly();
        self.runtime
            .apply_graph_projection(assembly.graph.clone())?;

        let hardware_request = HardwareConfigRequest::new(
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size,
            BackendPolicyTier::Tier0InHost,
        );
        self.runtime.apply_hardware_config(hardware_request)?;
        self.runtime
            .set_active_output_device("server:virtual-output");
        self.set_backend_policy(BackendPolicyOverride {
            tier: hardware_request.backend_policy,
        })?;
        self.runtime
            .set_backend_policy_tier(hardware_request.backend_policy);

        self.start_plugin_scan(PluginScanRequest {
            roots: vec!["/srv/plugins/clap".into()],
            formats: vec![PluginFormat::Clap],
        })?;

        for sandbox in &assembly.plugin_sandboxes {
            self.ensure_plugin_sandbox(sandbox.spec())?;
        }
        self.runtime
            .apply_plugin_backed_node_bindings(assembly.plugin_bindings())?;
        self.runtime
            .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());
        let sandbox = assembly.primary_sandbox();
        self.runtime.set_cpu_load_percent(1.2);
        self.runtime.set_graph_latency_ms(1.1);
        self.runtime.start()?;

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:server",
            "instance:server:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            2048,
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let mut run = self.run_lifecycle(
            &protocol,
            sandbox.request.sandbox_id.as_str(),
            1,
            &mut lifecycle,
        )?;
        if let Some(fault) = fault {
            match fault {
                FaultInjection::Timeout => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let timeout_result = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            false,
                            true,
                        )?;
                        if let Some(outcome) = timeout_result {
                            if outcome.result.slot.state == CompletionState::TimedOut
                                && run.current_watchdog_triggered
                            {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    fault,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                run = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    None,
                                )?;
                                break;
                            }
                        }
                    }
                }
                FaultInjection::Crash => {
                    let crash_sequence = self.runtime.allocate_block_sequence();
                    let _ = self.run_realtime_cycle(
                        &protocol,
                        &mut run,
                        crash_sequence,
                        &mut lifecycle,
                        false,
                        false,
                    )?;
                    let failure = build_fault_envelope(
                        &sandbox.request.sandbox_id,
                        "instance:server:default",
                        &run.shared_memory_lease_id,
                        run.processing_epoch,
                        fault,
                    );
                    record_runtime_fault(&mut self.runtime, &failure);
                    run = self.recover_sandbox(
                        &protocol,
                        &sandbox.request.sandbox_id,
                        &mut lifecycle,
                        &run,
                        RecoveryRestartIntent::CrashRecovery,
                        None,
                    )?;
                }
                FaultInjection::HeartbeatMiss => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let outcome = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            true,
                            false,
                        )?;
                        if outcome.is_none() && run.current_watchdog_triggered {
                            let failure = build_fault_envelope(
                                &sandbox.request.sandbox_id,
                                "instance:server:default",
                                &run.shared_memory_lease_id,
                                run.processing_epoch,
                                fault,
                            );
                            record_runtime_fault(&mut self.runtime, &failure);
                            run = self.handle_watchdog_recovery(
                                &protocol,
                                &sandbox.request.sandbox_id,
                                &mut lifecycle,
                                &run,
                                None,
                            )?;
                            break;
                        }
                    }
                }
                FaultInjection::RecoveryTeardownFailure => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let timeout_result = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            false,
                            true,
                        )?;
                        if let Some(outcome) = timeout_result {
                            if outcome.result.slot.state == CompletionState::TimedOut
                                && run.current_watchdog_triggered
                            {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::Timeout,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                return match self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::OldTransportTeardown),
                                ) {
                                    Ok(_) => Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected injected recovery teardown failure",
                                    )),
                                    Err(error) => Err(error),
                                };
                            }
                        }
                    }
                }
                FaultInjection::RecoveryDeferredTeardownFailure => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let timeout_result = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            false,
                            true,
                        )?;
                        if let Some(outcome) = timeout_result {
                            if outcome.result.slot.state == CompletionState::TimedOut
                                && run.current_watchdog_triggered
                            {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::Timeout,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                return match self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::DeferredOldTransportTeardown),
                                ) {
                                    Ok(_) => Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected deferred teardown recovery failure",
                                    )),
                                    Err(error) => Err(error),
                                };
                            }
                        }
                    }
                }
                FaultInjection::RecoveryDeferredTeardownThenCleanup => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let timeout_result = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            false,
                            true,
                        )?;
                        if let Some(outcome) = timeout_result {
                            if outcome.result.slot.state == CompletionState::TimedOut
                                && run.current_watchdog_triggered
                            {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::Timeout,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                let first_error = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::DeferredOldTransportTeardown),
                                );
                                if first_error.is_ok() {
                                    return Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected deferred teardown recovery failure",
                                    ));
                                }
                                run = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    None,
                                )?;
                                break;
                            }
                        }
                    }
                }
                FaultInjection::RecoveryDeferredTeardownCleanupRetry => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let timeout_result = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            false,
                            true,
                        )?;
                        if let Some(outcome) = timeout_result {
                            if outcome.result.slot.state == CompletionState::TimedOut
                                && run.current_watchdog_triggered
                            {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::Timeout,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                let first_error = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::DeferredOldTransportTeardown),
                                );
                                if first_error.is_ok() {
                                    return Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected deferred teardown recovery failure",
                                    ));
                                }
                                let second_error = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::LingeringCleanupTeardown),
                                );
                                if second_error.is_ok() {
                                    return Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected lingering cleanup retry failure",
                                    ));
                                }
                                run = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    None,
                                )?;
                                break;
                            }
                        }
                    }
                }
                FaultInjection::RecoveryRestartFailure => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let timeout_result = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            false,
                            true,
                        )?;
                        if let Some(outcome) = timeout_result {
                            if outcome.result.slot.state == CompletionState::TimedOut
                                && run.current_watchdog_triggered
                            {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::Timeout,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                return match self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::ReplacementStart),
                                ) {
                                    Ok(_) => Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected injected replacement start failure",
                                    )),
                                    Err(error) => Err(error),
                                };
                            }
                        }
                    }
                }
                FaultInjection::RecoveryOverlapContention => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let timeout_result = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            false,
                            true,
                        )?;
                        if let Some(outcome) = timeout_result {
                            if outcome.result.slot.state == CompletionState::TimedOut
                                && run.current_watchdog_triggered
                            {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::Timeout,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                return match self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::CompetingOverlapAttach),
                                ) {
                                    Ok(_) => Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected injected overlap contention failure",
                                    )),
                                    Err(error) => Err(error),
                                };
                            }
                        }
                    }
                }
                FaultInjection::RecoveryInterleavedFailures => {
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let timeout_result = self.run_realtime_cycle(
                            &protocol,
                            &mut run,
                            block_sequence,
                            &mut lifecycle,
                            false,
                            true,
                        )?;
                        if let Some(outcome) = timeout_result {
                            if outcome.result.slot.state == CompletionState::TimedOut
                                && run.current_watchdog_triggered
                            {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::Timeout,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                let first_error = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::DeferredOldTransportTeardown),
                                );
                                if first_error.is_ok() {
                                    return Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected deferred teardown recovery failure",
                                    ));
                                }
                                return match self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    Some(RecoveryFailureInjection::CompetingOverlapAttach),
                                ) {
                                    Ok(_) => Err(RuntimeError::new(
                                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                                        "expected interleaved recovery failures",
                                    )),
                                    Err(error) => Err(error),
                                };
                            }
                        }
                    }
                }
                FaultInjection::EscalatingHeartbeatMisses { restart_episodes } => {
                    for restart_episode in 0..restart_episodes {
                        for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                            let block_sequence = self.runtime.allocate_block_sequence();
                            let outcome = self.run_realtime_cycle(
                                &protocol,
                                &mut run,
                                block_sequence,
                                &mut lifecycle,
                                true,
                                false,
                            )?;
                            if outcome.is_none() && run.current_watchdog_triggered {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::HeartbeatMiss,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                run = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    None,
                                )?;
                                if restart_episode + 1 < restart_episodes {
                                    self.execute_block_sequence(
                                        &protocol,
                                        &mut run,
                                        INTER_EPISODE_CONTINUITY_BLOCKS,
                                        &mut lifecycle,
                                        false,
                                    )?;
                                }
                                break;
                            }
                        }
                    }
                }
                FaultInjection::MixedWatchdogEpisodes { restart_episodes } => {
                    for restart_episode in 0..restart_episodes {
                        let episode_fault = if restart_episode % 2 == 0 {
                            FaultInjection::HeartbeatMiss
                        } else {
                            FaultInjection::Timeout
                        };
                        for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                            let block_sequence = self.runtime.allocate_block_sequence();
                            let (simulate_heartbeat_miss, simulate_timeout) = match episode_fault {
                                FaultInjection::HeartbeatMiss => (true, false),
                                FaultInjection::Timeout => (false, true),
                                _ => (false, false),
                            };
                            let outcome = self.run_realtime_cycle(
                                &protocol,
                                &mut run,
                                block_sequence,
                                &mut lifecycle,
                                simulate_heartbeat_miss,
                                simulate_timeout,
                            )?;
                            let watchdog_fired = match episode_fault {
                                FaultInjection::HeartbeatMiss => {
                                    outcome.is_none() && run.current_watchdog_triggered
                                }
                                FaultInjection::Timeout => {
                                    outcome.as_ref().is_some_and(|outcome| {
                                        outcome.result.slot.state == CompletionState::TimedOut
                                            && run.current_watchdog_triggered
                                    })
                                }
                                _ => false,
                            };
                            if watchdog_fired {
                                let failure = build_fault_envelope(
                                    &sandbox.request.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    episode_fault,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                if matches!(episode_fault, FaultInjection::Timeout) {
                                    self.runtime.increment_xruns();
                                }
                                run = self.handle_watchdog_recovery(
                                    &protocol,
                                    &sandbox.request.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                    None,
                                )?;
                                if restart_episode + 1 < restart_episodes {
                                    self.execute_block_sequence(
                                        &protocol,
                                        &mut run,
                                        INTER_EPISODE_CONTINUITY_BLOCKS,
                                        &mut lifecycle,
                                        false,
                                    )?;
                                }
                                break;
                            }
                        }
                    }
                }
            }
            self.execute_block_sequence(
                &protocol,
                &mut run,
                STEADY_STATE_BLOCKS,
                &mut lifecycle,
                false,
            )?;
        } else {
            self.execute_block_sequence(
                &protocol,
                &mut run,
                STEADY_STATE_BLOCKS,
                &mut lifecycle,
                false,
            )?;
        }
        let header = protocol.block_header(
            run.processing_epoch,
            run.last_block_sequence,
            self.runtime.config().graph.block_size as u32,
        );
        Ok(ServerRuntimeHostSummary {
            scan_roots: self.supervisor.last_scan_roots.clone(),
            execution: ServerExecutionSummary {
                control_requests: run.control_requests,
                control_responses: run.control_responses,
                heartbeat_responses: run.heartbeat_responses,
                processed_blocks: run.processed_blocks,
                engine_processed_blocks: run.engine_processed_blocks,
                last_control_message: run.last_control_message.clone(),
                last_completion_state: run.last_completion_state,
                last_block_sequence: run.last_block_sequence,
                last_engine_graph_id: run.last_engine_graph_id.clone(),
                last_engine_output_peak: run.last_engine_output_peak,
                last_engine_output_rms: run.last_engine_output_rms,
                processing_epoch: run.processing_epoch,
                restart_count: self.supervisor.restarts,
                teardown_count: self.supervisor.teardowns,
                last_recovery_intent: self.supervisor.last_recovery_intent,
                last_stop_reason: self.supervisor.last_stop_reason,
                last_plugin_state: run.last_plugin_state.clone(),
            },
            transport: ServerTransportSummary {
                sandbox_id: self
                    .supervisor
                    .last_sandbox_id
                    .clone()
                    .unwrap_or_else(|| sandbox.request.sandbox_id.clone()),
                shared_memory_lease_id: run.shared_memory_lease_id,
                shared_memory_region_id: run
                    .transport
                    .as_ref()
                    .map(|transport| transport.region_id.clone())
                    .unwrap_or_default(),
                shared_memory_path: run
                    .transport
                    .as_ref()
                    .map(|transport| transport.backing_path.clone())
                    .unwrap_or_default(),
                shared_memory_bytes: header.layout.total_bytes(),
            },
            last_payload: ServerPayloadSummary {
                event_count: run.last_output_event_count,
                parameter_event_count: run.last_parameter_event_count,
                parameter_gesture_event_count: run.last_parameter_gesture_event_count,
                parameter_modulation_event_count: run.last_parameter_modulation_event_count,
                note_event_count: run.last_note_event_count,
                note_expression_event_count: run.last_note_expression_event_count,
                midi_event_count: run.last_midi_event_count,
                generated_event_bytes: run.last_generated_event_bytes,
                first_output_sample: run.last_output_first_sample,
            },
            faults: ServerFaultSummary {
                deadline_misses: run.deadline_misses,
                heartbeat_misses: run.heartbeat_misses,
                watchdog_triggered: run.watchdog_triggered,
                watchdog_trigger_reason: run.watchdog_trigger_reason,
            },
        })
    }

    fn run_lifecycle(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        processing_epoch: u64,
        lifecycle: &mut ClapSandboxLifecycleHarness,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        let control_sequence = protocol
            .lifecycle_sequence(
                &self.broker,
                sandbox_id,
                self.runtime.config().sample_rate.0,
                self.runtime.config().graph.block_size as u32,
                processing_epoch,
            )
            .map_err(|error| {
                record_broker_failure_and_convert(
                    &mut self.runtime,
                    sandbox_id,
                    None,
                    Some(processing_epoch),
                    None,
                    BrokerFailureStage::PreparePlanCreate,
                    error,
                )
            })?;
        let mut responses = Vec::with_capacity(control_sequence.len());
        for request in control_sequence.iter().cloned() {
            if let Some(stage) = lifecycle_stage_for_request(&request.payload) {
                self.runtime.record_plugin_sandbox_lifecycle(
                    sandbox_id,
                    stage,
                    Some(processing_epoch),
                );
            }
            match lifecycle.handle(request) {
                Ok(response) => {
                    if let Some(instance_state) = plugin_instance_state_record_from_response(
                        sandbox_id,
                        Some(processing_epoch),
                        &response,
                    ) {
                        self.runtime
                            .record_plugin_sandbox_instance_state(instance_state);
                    }
                    responses.push(response);
                }
                Err(failure) => {
                    record_runtime_fault(&mut self.runtime, &failure);
                    return Err(runtime_error_from_failure(&failure));
                }
            }
        }
        self.runtime.record_heartbeat_cycle(
            sandbox_id,
            HeartbeatCycleStage::Requested,
            Some(processing_epoch),
            None,
        );
        let heartbeat = lifecycle
            .handle(protocol.heartbeat_request(sandbox_id, Some(processing_epoch)))
            .map_err(|failure| {
                record_runtime_fault(&mut self.runtime, &failure);
                runtime_error_from_failure(&failure)
            })?;
        if let Some(instance_state) = plugin_instance_state_record_from_response(
            sandbox_id,
            Some(processing_epoch),
            &heartbeat,
        ) {
            self.runtime
                .record_plugin_sandbox_instance_state(instance_state);
        }
        self.runtime.record_heartbeat_cycle(
            sandbox_id,
            HeartbeatCycleStage::Responded,
            Some(processing_epoch),
            None,
        );
        responses.push(heartbeat);

        let (shared_memory_lease_id, transport) = extract_prepare_metadata(&responses);
        if let Some(transport) = &transport {
            if let Err(error) = self
                .runtime
                .begin_transport_session_with_metadata_for_epoch(
                    sandbox_id,
                    shared_memory_lease_id.as_str(),
                    transport.region_id.as_str(),
                    transport_attach_intent(processing_epoch),
                    Some(processing_epoch),
                    match transport_attach_intent(processing_epoch) {
                        TransportAttachIntent::SteadyState => {
                            signal_runtime::TransportSessionProvenance::SteadyOrigin
                        }
                        TransportAttachIntent::RecoveryOverlap => {
                            signal_runtime::TransportSessionProvenance::RecoveryReplacement
                        }
                    },
                    Some(transport.backing_path.clone()),
                    Some(transport.total_bytes),
                )
            {
                self.rollback_unadmitted_lifecycle_setup(
                    protocol,
                    sandbox_id,
                    lifecycle,
                    processing_epoch,
                    shared_memory_lease_id.as_str(),
                    transport,
                    "transport admission rejected",
                );
                return Err(error);
            }
            self.runtime.record_plugin_sandbox_lifecycle(
                sandbox_id,
                PluginSandboxLifecycleStage::TransportAttached,
                Some(processing_epoch),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
                PluginSandboxTransportStage::Attached,
                Some(processing_epoch),
                None,
            );
        }

        Ok(LifecycleRunSummary {
            sandbox_id: sandbox_id.to_string(),
            control_requests: control_sequence.len() + 1,
            control_responses: responses.len(),
            heartbeat_responses: 1,
            processed_blocks: 0,
            engine_processed_blocks: 0,
            last_control_message: responses
                .last()
                .map(|response| response.message.name.clone())
                .unwrap_or_default(),
            last_completion_state: CompletionState::Idle,
            last_block_sequence: 0,
            last_engine_graph_id: None,
            last_engine_output_peak: None,
            last_engine_output_rms: None,
            last_output_event_count: 0,
            last_parameter_event_count: 0,
            last_parameter_gesture_event_count: 0,
            last_parameter_modulation_event_count: 0,
            last_note_event_count: 0,
            last_note_expression_event_count: 0,
            last_midi_event_count: 0,
            last_generated_event_bytes: 0,
            last_output_first_sample: None,
            deadline_misses: 0,
            heartbeat_misses: 0,
            watchdog_triggered: false,
            watchdog_trigger_reason: None,
            current_watchdog_triggered: false,
            watchdog: SandboxWatchdogState::default(),
            processing_epoch,
            shared_memory_lease_id,
            transport,
            last_plugin_state: responses
                .iter()
                .filter_map(|response| {
                    plugin_instance_state_record_from_response(
                        sandbox_id,
                        Some(processing_epoch),
                        response,
                    )
                })
                .last(),
        })
    }

    fn rollback_unadmitted_lifecycle_setup(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        processing_epoch: u64,
        lease_id: &str,
        transport: &SharedMemoryTransportPayload,
        detail: &str,
    ) {
        for request in protocol.teardown_sequence(sandbox_id, processing_epoch) {
            match lifecycle.handle(request.clone()) {
                Ok(_) => {
                    if let Some(stage) = lifecycle_stage_for_request(&request.payload) {
                        self.runtime.record_plugin_sandbox_lifecycle(
                            sandbox_id,
                            stage,
                            Some(processing_epoch),
                        );
                    }
                }
                Err(failure) => record_runtime_fault(&mut self.runtime, &failure),
            }
        }

        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            lease_id,
            transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(processing_epoch),
            Some(detail.into()),
        );

        let destroy_error = self.broker.destroy_region(transport).err();
        if let Some(error) = destroy_error.as_ref() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(lease_id.to_string()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                lease_id,
                transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.to_string()),
            );
        }

        let teardown_error = lifecycle.teardown_active_transport().err();
        if let Some(error) = teardown_error.as_ref() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(lease_id.to_string()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                lease_id,
                transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.to_string()),
            );
        }

        if destroy_error.is_none() && teardown_error.is_none() {
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                lease_id,
                transport.region_id.as_str(),
                PluginSandboxTransportStage::Detached,
                Some(processing_epoch),
                Some(detail.into()),
            );
            self.runtime.record_plugin_sandbox_lifecycle(
                sandbox_id,
                PluginSandboxLifecycleStage::TransportTornDown,
                Some(processing_epoch),
            );
        }
    }

    fn execute_block_sequence(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        block_count: u64,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        simulate_timeout: bool,
    ) -> Result<(), RuntimeError> {
        let mut last_completed_block_sequence = None;
        for block_offset in 0..block_count {
            let block_sequence = self
                .runtime
                .next_planned_prework_block_sequence(last_completed_block_sequence)
                .unwrap_or_else(|| self.runtime.allocate_block_sequence());
            let should_timeout = simulate_timeout && block_offset == 0;
            let _ = self.run_realtime_cycle(
                protocol,
                run,
                block_sequence,
                lifecycle,
                false,
                should_timeout,
            )?;
            last_completed_block_sequence = Some(block_sequence);
        }
        Ok(())
    }

    fn run_realtime_cycle(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        simulate_heartbeat_miss: bool,
        simulate_timeout: bool,
    ) -> Result<Option<BrokeredBlockOutcome>, RuntimeError> {
        self.runtime
            .set_prework_service_pressure(self.prework_service_pressure(
                run,
                simulate_heartbeat_miss,
                simulate_timeout,
            ))?;
        let _ = self.runtime.service_prework_lane(run.processing_epoch, 1)?;
        if !self.poll_heartbeat(
            protocol,
            run,
            lifecycle,
            simulate_heartbeat_miss,
            Some(block_sequence),
        )? {
            return Ok(None);
        }

        let result =
            self.execute_block(protocol, run, block_sequence, lifecycle, simulate_timeout)?;
        self.runtime
            .set_prework_service_pressure(self.prework_service_pressure(
                run,
                false,
                simulate_timeout && run.current_watchdog_triggered,
            ))?;
        let _ = self.runtime.service_prework_lane(run.processing_epoch, 1)?;
        Ok(Some(result))
    }

    fn prework_service_pressure(
        &self,
        run: &LifecycleRunSummary,
        simulate_heartbeat_miss: bool,
        simulate_timeout: bool,
    ) -> RuntimePreworkServicePressure {
        if simulate_heartbeat_miss || simulate_timeout || run.current_watchdog_triggered {
            RuntimePreworkServicePressure::Critical
        } else if run.watchdog_triggered {
            RuntimePreworkServicePressure::Elevated
        } else {
            RuntimePreworkServicePressure::Normal
        }
    }

    fn poll_heartbeat(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        simulate_miss: bool,
        block_sequence: Option<u64>,
    ) -> Result<bool, RuntimeError> {
        if simulate_miss {
            self.runtime.record_heartbeat_cycle(
                run.sandbox_id.as_str(),
                HeartbeatCycleStage::Missed,
                Some(run.processing_epoch),
                block_sequence,
            );
            run.heartbeat_misses = run.heartbeat_misses.saturating_add(1);
            if let WatchdogOutcome::RestartRequired {
                reason,
                consecutive_misses: _,
            } = run.watchdog.record_heartbeat_miss()
            {
                run.watchdog_triggered = true;
                run.watchdog_trigger_reason = Some(reason);
                run.current_watchdog_triggered = true;
                self.runtime.record_watchdog_restart(WatchdogRestartRecord {
                    sandbox_id: run.sandbox_id.clone(),
                    trigger: runtime_watchdog_trigger(reason),
                    processing_epoch: run.processing_epoch,
                });
            }
            return Ok(false);
        }

        self.runtime.record_heartbeat_cycle(
            run.sandbox_id.as_str(),
            HeartbeatCycleStage::Requested,
            Some(run.processing_epoch),
            block_sequence,
        );
        let heartbeat = lifecycle
            .handle(protocol.heartbeat_request(run.sandbox_id.as_str(), Some(run.processing_epoch)))
            .map_err(|failure| {
                record_runtime_fault(&mut self.runtime, &failure);
                runtime_error_from_failure(&failure)
            })?;
        if let Some(instance_state) = plugin_instance_state_record_from_response(
            run.sandbox_id.as_str(),
            Some(run.processing_epoch),
            &heartbeat,
        ) {
            self.runtime
                .record_plugin_sandbox_instance_state(instance_state.clone());
            run.last_plugin_state = Some(instance_state);
        }
        self.runtime.record_heartbeat_cycle(
            run.sandbox_id.as_str(),
            HeartbeatCycleStage::Responded,
            Some(run.processing_epoch),
            block_sequence,
        );
        run.control_requests = run.control_requests.saturating_add(1);
        run.control_responses = run.control_responses.saturating_add(1);
        run.heartbeat_responses = run.heartbeat_responses.saturating_add(1);
        run.last_control_message = heartbeat.message.name.clone();
        run.watchdog.record_heartbeat_response();
        if let Some(sequence) = block_sequence {
            run.last_block_sequence = sequence;
        }
        Ok(true)
    }

    fn execute_block(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        simulate_timeout: bool,
    ) -> Result<BrokeredBlockOutcome, RuntimeError> {
        let frame_count = self.runtime.config().graph.block_size as u32;
        let transport = run.transport.clone().ok_or_else(|| {
            RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "lifecycle completed without brokered shared-memory transport",
            )
        })?;
        let dispatch = protocol.block_dispatch(
            run.processing_epoch,
            block_sequence,
            frame_count,
            protocol.default_render_context(frame_count),
        );
        self.runtime.record_block_dispatch(
            run.sandbox_id.as_str(),
            run.shared_memory_lease_id.as_str(),
            run.processing_epoch,
            block_sequence,
            frame_count,
            BlockDispatchStage::Requested,
            None,
        );
        let payload = protocol.test_input_payload(block_sequence, frame_count);
        protocol
            .write_block_payload(&self.broker, &transport, &dispatch, &payload)
            .map_err(|error| {
                record_broker_failure_and_convert(
                    &mut self.runtime,
                    run.sandbox_id.as_str(),
                    Some(run.shared_memory_lease_id.clone()),
                    Some(run.processing_epoch),
                    Some(block_sequence),
                    BrokerFailureStage::PayloadWrite,
                    error,
                )
            })?;
        self.runtime.record_completion_slot_transition(
            run.sandbox_id.as_str(),
            run.shared_memory_lease_id.as_str(),
            run.processing_epoch,
            block_sequence,
            CompletionSlotStage::ReadyForProcessing,
        );
        let _ = if simulate_timeout {
            lifecycle.mark_deadline_miss()
        } else {
            lifecycle.process_pending_block()
        }
        .map_err(|failure| {
            record_runtime_fault(&mut self.runtime, &failure);
            runtime_error_from_failure(&failure)
        })?;
        let stored_result = protocol
            .read_block_outcome(&self.broker, &transport, &dispatch)
            .map_err(|error| {
                record_broker_failure_and_convert(
                    &mut self.runtime,
                    run.sandbox_id.as_str(),
                    Some(run.shared_memory_lease_id.clone()),
                    Some(run.processing_epoch),
                    Some(block_sequence),
                    BrokerFailureStage::PayloadRead,
                    error,
                )
            })?;
        if simulate_timeout {
            self.runtime.record_completion_slot_transition(
                run.sandbox_id.as_str(),
                run.shared_memory_lease_id.as_str(),
                run.processing_epoch,
                block_sequence,
                CompletionSlotStage::TimedOut,
            );
            if stored_result.result.fallback_applied {
                self.runtime.record_completion_slot_transition(
                    run.sandbox_id.as_str(),
                    run.shared_memory_lease_id.as_str(),
                    run.processing_epoch,
                    block_sequence,
                    CompletionSlotStage::FallbackApplied,
                );
            }
        } else {
            self.runtime.record_completion_slot_transition(
                run.sandbox_id.as_str(),
                run.shared_memory_lease_id.as_str(),
                run.processing_epoch,
                block_sequence,
                CompletionSlotStage::Processing,
            );
            if stored_result.result.slot.state == CompletionState::Completed {
                self.runtime.record_completion_slot_transition(
                    run.sandbox_id.as_str(),
                    run.shared_memory_lease_id.as_str(),
                    run.processing_epoch,
                    block_sequence,
                    CompletionSlotStage::Completed,
                );
            }
        }
        let event_summary = stored_result.output.events.summary();
        let _ = self
            .runtime
            .apply_forecast_state_for_block(run.processing_epoch, block_sequence)?;
        let engine_result = self.runtime.process_engine_block(
            run.processing_epoch,
            block_sequence,
            synthetic_stereo_block(
                self.runtime.config().sample_rate,
                FrameCount(frame_count as usize),
                block_sequence.saturating_add(17),
            ),
        )?;
        run.processed_blocks = run.processed_blocks.saturating_add(1);
        run.engine_processed_blocks = run.engine_processed_blocks.saturating_add(1);
        run.last_completion_state = stored_result.result.slot.state;
        run.last_block_sequence = block_sequence;
        run.last_engine_graph_id = engine_result.snapshot.graph_id.clone();
        run.last_engine_output_peak = engine_result.snapshot.last_output_peak;
        run.last_engine_output_rms = engine_result.snapshot.last_output_rms;
        run.last_output_event_count = stored_result.output.events.event_count();
        run.last_parameter_event_count = event_summary.parameter_value_events;
        run.last_parameter_gesture_event_count = event_summary.parameter_gesture_events;
        run.last_parameter_modulation_event_count = event_summary.parameter_modulation_events;
        run.last_note_event_count = event_summary.note_events;
        run.last_note_expression_event_count = event_summary.note_expression_events;
        run.last_midi_event_count = event_summary.midi_events;
        run.last_generated_event_bytes = stored_result.result.generated_event_bytes;
        let automation_summary = stored_result
            .output
            .events
            .parameter_automation_summary(protocol.automation_parameter_id());
        self.runtime.record_automation_summary(
            run.processing_epoch,
            run.shared_memory_lease_id.as_str(),
            automation_summary,
        );
        let dispatch_stage = if stored_result.result.slot.state == CompletionState::TimedOut {
            BlockDispatchStage::TimedOut
        } else {
            BlockDispatchStage::Completed
        };
        self.runtime.record_block_dispatch(
            run.sandbox_id.as_str(),
            run.shared_memory_lease_id.as_str(),
            run.processing_epoch,
            block_sequence,
            frame_count,
            dispatch_stage,
            Some(stored_result.result.slot.state),
        );
        self.runtime.record_block_sequence(
            run.sandbox_id.as_str(),
            run.processing_epoch,
            run.shared_memory_lease_id.as_str(),
            block_sequence,
        );
        run.last_output_first_sample = stored_result.output.audio.first_sample();
        if stored_result.result.slot.state == CompletionState::TimedOut {
            run.deadline_misses = run.deadline_misses.saturating_add(1);
        }
        if let WatchdogOutcome::RestartRequired {
            reason,
            consecutive_misses: _,
        } = run
            .watchdog
            .record_block_completion(stored_result.result.slot.state)
        {
            run.watchdog_triggered = true;
            run.watchdog_trigger_reason = Some(reason);
            run.current_watchdog_triggered = true;
            self.runtime.record_watchdog_restart(WatchdogRestartRecord {
                sandbox_id: run.sandbox_id.clone(),
                trigger: runtime_watchdog_trigger(reason),
                processing_epoch: run.processing_epoch,
            });
        }
        Ok(stored_result)
    }

    fn recover_sandbox(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        intent: RecoveryRestartIntent,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        let current_transport = run.transport.clone().ok_or_else(|| {
            RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "lifecycle completed without brokered shared-memory transport",
            )
        })?;
        let prior_history = run.recovery_history();
        let next_epoch = run.processing_epoch.saturating_add(1);
        self.stop_runtime_for_recovery()?;
        self.supervisor.last_recovery_intent = Some(intent);
        self.supervisor.last_stop_reason = Some(StopReason::DegradedModeRecovery);
        self.runtime.record_recovery_cycle(
            sandbox_id,
            intent,
            StopReason::DegradedModeRecovery,
            Some(run.processing_epoch),
        );
        let (completion_invalidated, lease_invalidated) =
            lifecycle.invalidate_active_epoch(run.processing_epoch);
        let recovery_reason = match intent {
            RecoveryRestartIntent::CrashRecovery => "crash recovery teardown",
            RecoveryRestartIntent::WatchdogRecovery => "watchdog recovery teardown",
        };
        if completion_invalidated {
            self.runtime.record_completion_slot_transition(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                run.processing_epoch,
                run.last_block_sequence,
                CompletionSlotStage::Invalidated,
            );
            self.runtime.record_broker_invalidation(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                run.processing_epoch,
                Some(run.last_block_sequence),
                BrokerInvalidationStage::CompletionRegionInvalidated,
                recovery_reason,
            );
        }
        if lease_invalidated {
            self.runtime.record_broker_invalidation(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                run.processing_epoch,
                Some(run.last_block_sequence),
                BrokerInvalidationStage::LeaseEpochInvalidated,
                recovery_reason,
            );
        }
        if failure != Some(RecoveryFailureInjection::CompetingOverlapAttach)
            && self.session_is_lingering(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            )
        {
            return self.recover_from_lingering_session(
                protocol,
                sandbox_id,
                lifecycle,
                run,
                prior_history,
                next_epoch,
                failure,
            );
        }
        self.cleanup_orphan_lingering_sessions_for_sandbox(
            sandbox_id,
            run.processing_epoch,
            Some(run.shared_memory_lease_id.as_str()),
            Some(current_transport.region_id.as_str()),
            LingeringCleanupMode::StrictPreAttach,
        )?;
        let mut replacement_lifecycle = ClapSandboxLifecycleHarness::default();
        let mut replacement_run =
            self.run_lifecycle(protocol, sandbox_id, next_epoch, &mut replacement_lifecycle)?;
        replacement_run.apply_recovery_history(prior_history);
        self.runtime.set_active_plugin_sandboxes(2);
        if matches!(
            failure,
            Some(RecoveryFailureInjection::CompetingOverlapAttach)
        ) {
            let mut competing_lifecycle = ClapSandboxLifecycleHarness::default();
            let contention_error = match self.run_lifecycle(
                protocol,
                sandbox_id,
                next_epoch.saturating_add(1),
                &mut competing_lifecycle,
            ) {
                Ok(competing_run) => {
                    self.rollback_replacement_recovery_session(
                        protocol,
                        sandbox_id,
                        &mut competing_lifecycle,
                        &competing_run,
                    );
                    RuntimeError::new(
                        signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                        "expected overlapping replacement attach contention",
                    )
                }
                Err(error) => error,
            };
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.abort_origin_recovery_session(protocol, sandbox_id, lifecycle, run);
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(contention_error);
        }
        for request in protocol.teardown_sequence(sandbox_id, run.processing_epoch) {
            match lifecycle.handle(request.clone()) {
                Ok(_) => {
                    if let Some(stage) = lifecycle_stage_for_request(&request.payload) {
                        self.runtime.record_plugin_sandbox_lifecycle(
                            sandbox_id,
                            stage,
                            Some(run.processing_epoch),
                        );
                    }
                }
                Err(failure) => {
                    record_runtime_fault(&mut self.runtime, &failure);
                    self.rollback_replacement_recovery_session(
                        protocol,
                        sandbox_id,
                        &mut replacement_lifecycle,
                        &replacement_run,
                    );
                    self.runtime.set_active_plugin_sandboxes(0);
                    return Err(runtime_error_from_failure(&failure));
                }
            }
        }
        self.runtime.set_active_plugin_sandboxes(1);
        if let Err(error) = self.teardown_plugin_sandbox(sandbox_id) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(run.processing_epoch),
            None,
        );
        if matches!(
            failure,
            Some(RecoveryFailureInjection::DeferredOldTransportTeardown)
        ) {
            let error =
                std::io::Error::other("deferred old transport teardown during recovery retry");
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(1);
            return Err(runtime_error_from_io(error));
        }
        if let Err(error) = self.broker.destroy_region(&current_transport) {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        if matches!(
            failure,
            Some(RecoveryFailureInjection::OldTransportTeardown)
        ) {
            let error = std::io::Error::other(
                "injected old transport teardown failure during overlap recovery",
            );
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        if let Err(error) = lifecycle.teardown_active_transport() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
            );
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(runtime_error_from_io(error));
        }
        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::Detached,
            Some(run.processing_epoch),
            None,
        );
        self.runtime.end_transport_session(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::TransportTornDown,
            Some(run.processing_epoch),
        );
        if let Err(error) = self.restart_plugin_sandbox(sandbox_id) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        self.runtime.set_active_plugin_sandboxes(1);
        if matches!(failure, Some(RecoveryFailureInjection::ReplacementStart)) {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "injected replacement start failure during overlap recovery",
            ));
        }
        if let Err(error) = self.runtime.start() {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut replacement_lifecycle,
                &replacement_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        self.reconcile_late_lingering_sessions_after_start(sandbox_id, &replacement_run);
        *lifecycle = replacement_lifecycle;
        Ok(replacement_run)
    }

    fn session_is_lingering(&self, sandbox_id: &str, lease_id: &str, region_id: &str) -> bool {
        self.runtime
            .get_transport_concurrency_snapshot()
            .active_sessions
            .iter()
            .find(|session| {
                session.sandbox_id == sandbox_id
                    && session.lease_id == lease_id
                    && session.region_id == region_id
            })
            .is_some_and(|session| {
                matches!(
                    session.state,
                    signal_runtime::TransportSessionState::DetachRequested
                        | signal_runtime::TransportSessionState::DetachFaulted
                )
            })
    }

    fn cleanup_orphan_lingering_sessions_for_sandbox(
        &mut self,
        sandbox_id: &str,
        processing_epoch: u64,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
        mode: LingeringCleanupMode,
    ) -> Result<(), RuntimeError> {
        let trigger = match mode {
            LingeringCleanupMode::StrictPreAttach => {
                signal_runtime::LingeringCleanupTrigger::RecoveryPreAttach
            }
            LingeringCleanupMode::BestEffortPostStart => {
                signal_runtime::LingeringCleanupTrigger::PostStartReconciliation
            }
        };
        let _ = self.runtime.enqueue_lingering_cleanup_work(
            sandbox_id,
            mode,
            trigger,
            processing_epoch,
            exclude_lease_id,
            exclude_region_id,
        );
        while let Some(plan) = self
            .runtime
            .dequeue_lingering_cleanup_work_for_sandbox(sandbox_id, processing_epoch)
        {
            for session in plan.candidates {
                if let Err(error) =
                    self.cleanup_orphan_lingering_transport(&session, plan.processing_epoch)
                {
                    self.runtime.record_lingering_cleanup_failure(
                        session.sandbox_id.as_str(),
                        session.lease_id.as_str(),
                        session.region_id.as_str(),
                        plan.mode,
                        plan.processing_epoch,
                        error.message.as_str(),
                    );
                    if plan.mode == LingeringCleanupMode::StrictPreAttach {
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    fn cleanup_orphan_lingering_transport(
        &mut self,
        session: &signal_runtime::ActiveTransportConcurrencySession,
        processing_epoch: u64,
    ) -> Result<(), RuntimeError> {
        let Some(backing_path) = session.backing_path.clone() else {
            let error = RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "orphan lingering transport is missing backing_path metadata",
            );
            self.runtime.record_broker_failure(
                session.sandbox_id.as_str(),
                Some(session.lease_id.clone()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportTeardown,
                error.message.clone(),
            );
            self.runtime.record_plugin_sandbox_transport(
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.message.clone()),
            );
            return Err(error);
        };
        let Some(total_bytes) = session.total_bytes else {
            let error = RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "orphan lingering transport is missing total_bytes metadata",
            );
            self.runtime.record_broker_failure(
                session.sandbox_id.as_str(),
                Some(session.lease_id.clone()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportTeardown,
                error.message.clone(),
            );
            self.runtime.record_plugin_sandbox_transport(
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.message.clone()),
            );
            return Err(error);
        };

        let transport = SharedMemoryTransportPayload {
            region_id: session.region_id.clone(),
            transport_kind: signal_ipc::SharedMemoryTransportKind::MappedFile,
            backing_path,
            total_bytes,
        };

        self.runtime.record_plugin_sandbox_transport(
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(processing_epoch),
            Some("orphan lingering cleanup".into()),
        );

        if let Err(error) = self.broker.destroy_region(&transport) {
            self.runtime.record_broker_failure(
                session.sandbox_id.as_str(),
                Some(session.lease_id.clone()),
                Some(processing_epoch),
                None,
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                session.sandbox_id.as_str(),
                session.lease_id.as_str(),
                session.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(processing_epoch),
                Some(error.to_string()),
            );
            return Err(runtime_error_from_io(error));
        }

        self.runtime.record_plugin_sandbox_transport(
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
            PluginSandboxTransportStage::Detached,
            Some(processing_epoch),
            Some("orphan lingering cleanup".into()),
        );
        self.runtime.complete_lingering_cleanup_success(
            session.sandbox_id.as_str(),
            session.lease_id.as_str(),
            session.region_id.as_str(),
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            session.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::TransportTornDown,
            Some(processing_epoch),
        );
        Ok(())
    }

    fn cleanup_lingering_origin_transport(
        &mut self,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<(), RuntimeError> {
        let Some(current_transport) = run.transport.as_ref() else {
            return Ok(());
        };

        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(run.processing_epoch),
            Some("lingering cleanup retry".into()),
        );

        if matches!(
            failure,
            Some(RecoveryFailureInjection::LingeringCleanupTeardown)
        ) {
            let error = std::io::Error::other("injected lingering cleanup retry failure");
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            return Err(runtime_error_from_io(error));
        }

        if let Err(error) = self.broker.destroy_region(current_transport) {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            return Err(runtime_error_from_io(error));
        }

        if let Err(error) = lifecycle.teardown_active_transport() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                current_transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
            return Err(runtime_error_from_io(error));
        }

        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
            PluginSandboxTransportStage::Detached,
            Some(run.processing_epoch),
            Some("lingering cleanup retry".into()),
        );
        self.runtime.end_transport_session(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            current_transport.region_id.as_str(),
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::TransportTornDown,
            Some(run.processing_epoch),
        );
        Ok(())
    }

    fn recover_from_lingering_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        prior_history: RecoveryHistory,
        next_epoch: u64,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        self.cleanup_lingering_origin_transport(sandbox_id, lifecycle, run, failure)?;
        self.cleanup_orphan_lingering_sessions_for_sandbox(
            sandbox_id,
            run.processing_epoch,
            Some(run.shared_memory_lease_id.as_str()),
            run.transport
                .as_ref()
                .map(|transport| transport.region_id.as_str()),
            LingeringCleanupMode::StrictPreAttach,
        )?;
        self.runtime.set_active_plugin_sandboxes(0);
        self.restart_plugin_sandbox(sandbox_id)?;
        self.runtime.set_active_plugin_sandboxes(1);

        let mut restarted_lifecycle = ClapSandboxLifecycleHarness::default();
        let mut restarted_run =
            self.run_lifecycle(protocol, sandbox_id, next_epoch, &mut restarted_lifecycle)?;
        restarted_run.apply_recovery_history(prior_history);

        if let Err(error) = self.runtime.start() {
            self.rollback_replacement_recovery_session(
                protocol,
                sandbox_id,
                &mut restarted_lifecycle,
                &restarted_run,
            );
            self.runtime.set_active_plugin_sandboxes(0);
            return Err(error);
        }
        self.reconcile_late_lingering_sessions_after_start(sandbox_id, &restarted_run);

        *lifecycle = restarted_lifecycle;
        Ok(restarted_run)
    }

    fn reconcile_late_lingering_sessions_after_start(
        &mut self,
        sandbox_id: &str,
        active_run: &LifecycleRunSummary,
    ) {
        let _ = self.cleanup_orphan_lingering_sessions_for_sandbox(
            sandbox_id,
            active_run.processing_epoch,
            Some(active_run.shared_memory_lease_id.as_str()),
            active_run
                .transport
                .as_ref()
                .map(|transport| transport.region_id.as_str()),
            LingeringCleanupMode::BestEffortPostStart,
        );
    }

    fn stop_runtime_for_recovery(&mut self) -> Result<(), RuntimeError> {
        if self.runtime.get_control_snapshot().running {
            self.runtime.stop(StopReason::DegradedModeRecovery)
        } else {
            Ok(())
        }
    }

    fn abort_origin_recovery_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
    ) {
        for request in protocol.teardown_sequence(sandbox_id, run.processing_epoch) {
            match lifecycle.handle(request.clone()) {
                Ok(_) => {
                    if let Some(stage) = lifecycle_stage_for_request(&request.payload) {
                        self.runtime.record_plugin_sandbox_lifecycle(
                            sandbox_id,
                            stage,
                            Some(run.processing_epoch),
                        );
                    }
                }
                Err(failure) => record_runtime_fault(&mut self.runtime, &failure),
            }
        }

        let Some(transport) = run.transport.as_ref() else {
            return;
        };

        let _ = self.teardown_plugin_sandbox(sandbox_id);
        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(run.processing_epoch),
            Some("origin recovery abort".into()),
        );

        let destroy_error = self.broker.destroy_region(transport).err();
        if let Some(error) = destroy_error.as_ref() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
        }

        let teardown_error = lifecycle.teardown_active_transport().err();
        if let Some(error) = teardown_error.as_ref() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
        }

        if destroy_error.is_none() && teardown_error.is_none() {
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
                PluginSandboxTransportStage::Detached,
                Some(run.processing_epoch),
                Some("origin recovery abort".into()),
            );
            self.runtime.record_plugin_sandbox_lifecycle(
                sandbox_id,
                PluginSandboxLifecycleStage::TransportTornDown,
                Some(run.processing_epoch),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
            );
        }
    }

    fn rollback_replacement_recovery_session(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
    ) {
        for request in protocol.teardown_sequence(sandbox_id, run.processing_epoch) {
            match lifecycle.handle(request.clone()) {
                Ok(_) => {
                    if let Some(stage) = lifecycle_stage_for_request(&request.payload) {
                        self.runtime.record_plugin_sandbox_lifecycle(
                            sandbox_id,
                            stage,
                            Some(run.processing_epoch),
                        );
                    }
                }
                Err(failure) => record_runtime_fault(&mut self.runtime, &failure),
            }
        }

        let Some(transport) = run.transport.as_ref() else {
            return;
        };

        self.runtime.record_plugin_sandbox_transport(
            sandbox_id,
            run.shared_memory_lease_id.as_str(),
            transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachRequested,
            Some(run.processing_epoch),
            Some("replacement rollback".into()),
        );

        let destroy_error = self.broker.destroy_region(transport).err();
        if let Some(error) = destroy_error.as_ref() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportDestroy,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
        }

        let teardown_error = lifecycle.teardown_active_transport().err();
        if let Some(error) = teardown_error.as_ref() {
            self.runtime.record_broker_failure(
                sandbox_id,
                Some(run.shared_memory_lease_id.clone()),
                Some(run.processing_epoch),
                Some(run.last_block_sequence),
                BrokerFailureStage::TransportTeardown,
                error.to_string(),
            );
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
                PluginSandboxTransportStage::DetachFault,
                Some(run.processing_epoch),
                Some(error.to_string()),
            );
        }

        if destroy_error.is_none() && teardown_error.is_none() {
            self.runtime.record_plugin_sandbox_transport(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
                PluginSandboxTransportStage::Detached,
                Some(run.processing_epoch),
                Some("replacement rollback".into()),
            );
            self.runtime.record_plugin_sandbox_lifecycle(
                sandbox_id,
                PluginSandboxLifecycleStage::TransportTornDown,
                Some(run.processing_epoch),
            );
            self.runtime.end_transport_session(
                sandbox_id,
                run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
            );
        }
    }

    fn handle_watchdog_recovery(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
        failure: Option<RecoveryFailureInjection>,
    ) -> Result<LifecycleRunSummary, RuntimeError> {
        self.recover_sandbox(
            protocol,
            sandbox_id,
            lifecycle,
            run,
            RecoveryRestartIntent::WatchdogRecovery,
            failure,
        )
    }

    pub fn runtime(&self) -> &SignalRuntime {
        &self.runtime
    }

    #[allow(dead_code)]
    pub fn observation_diagnostics(&self) -> RuntimeObservationDiagnostics {
        self.events.diagnostics()
    }

    #[allow(dead_code)]
    pub fn observation_report(&self) -> RuntimeObservationReport {
        RuntimeObservationReport::capture(&self.runtime, &self.events)
    }

    pub fn supervisor_report(&self) -> RuntimeSupervisorReport {
        RuntimeSupervisorReport::capture(&self.runtime, &self.events)
    }
}

#[derive(Clone, Debug)]
struct LifecycleRunSummary {
    sandbox_id: String,
    control_requests: usize,
    control_responses: usize,
    heartbeat_responses: usize,
    processed_blocks: usize,
    engine_processed_blocks: usize,
    last_control_message: String,
    last_completion_state: CompletionState,
    last_block_sequence: u64,
    last_engine_graph_id: Option<String>,
    last_engine_output_peak: Option<f32>,
    last_engine_output_rms: Option<f32>,
    last_output_event_count: usize,
    last_parameter_event_count: usize,
    last_parameter_gesture_event_count: usize,
    last_parameter_modulation_event_count: usize,
    last_note_event_count: usize,
    last_note_expression_event_count: usize,
    last_midi_event_count: usize,
    last_generated_event_bytes: u32,
    last_output_first_sample: Option<f32>,
    deadline_misses: u32,
    heartbeat_misses: u32,
    watchdog_triggered: bool,
    watchdog_trigger_reason: Option<WatchdogTriggerReason>,
    current_watchdog_triggered: bool,
    watchdog: SandboxWatchdogState,
    processing_epoch: u64,
    shared_memory_lease_id: String,
    transport: Option<SharedMemoryTransportPayload>,
    last_plugin_state: Option<PluginSandboxInstanceStateRecord>,
}

#[derive(Clone, Debug, Default)]
struct RecoveryHistory {
    control_requests: usize,
    control_responses: usize,
    heartbeat_responses: usize,
    processed_blocks: usize,
    engine_processed_blocks: usize,
    deadline_misses: u32,
    heartbeat_misses: u32,
    watchdog_triggered: bool,
    watchdog_trigger_reason: Option<WatchdogTriggerReason>,
}

impl LifecycleRunSummary {
    fn recovery_history(&self) -> RecoveryHistory {
        RecoveryHistory {
            control_requests: self.control_requests,
            control_responses: self.control_responses,
            heartbeat_responses: self.heartbeat_responses,
            processed_blocks: self.processed_blocks,
            engine_processed_blocks: self.engine_processed_blocks,
            deadline_misses: self.deadline_misses,
            heartbeat_misses: self.heartbeat_misses,
            watchdog_triggered: self.watchdog_triggered,
            watchdog_trigger_reason: self.watchdog_trigger_reason,
        }
    }

    fn apply_recovery_history(&mut self, history: RecoveryHistory) {
        self.control_requests = self
            .control_requests
            .saturating_add(history.control_requests);
        self.control_responses = self
            .control_responses
            .saturating_add(history.control_responses);
        self.heartbeat_responses = self
            .heartbeat_responses
            .saturating_add(history.heartbeat_responses);
        self.processed_blocks = self
            .processed_blocks
            .saturating_add(history.processed_blocks);
        self.engine_processed_blocks = self
            .engine_processed_blocks
            .saturating_add(history.engine_processed_blocks);
        self.deadline_misses = self.deadline_misses.saturating_add(history.deadline_misses);
        self.heartbeat_misses = self
            .heartbeat_misses
            .saturating_add(history.heartbeat_misses);
        self.watchdog_triggered |= history.watchdog_triggered;
        self.current_watchdog_triggered = false;
        if self.watchdog_trigger_reason.is_none() {
            self.watchdog_trigger_reason = history.watchdog_trigger_reason;
        }
    }
}

fn server_demo_graph_projection() -> GraphProjection {
    GraphProjection {
        graph_id: "signal.host.server.demo".into(),
        node_count: 3,
        nodes: vec![
            GraphNodeProjection {
                node_id: "input-shape".into(),
                execution_class: GraphNodeExecutionClass::PureTransform,
                latency_samples: 0,
                stages: vec![
                    GraphStageSpec::Gain { linear: 0.6 },
                    GraphStageSpec::Bias { amount: -0.04 },
                ],
            },
            GraphNodeProjection {
                node_id: "drive".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 0,
                stages: vec![GraphStageSpec::TanhDrive { drive: 1.6 }],
            },
            GraphNodeProjection {
                node_id: "output-trim".into(),
                execution_class: GraphNodeExecutionClass::LatencyBearing,
                latency_samples: 32,
                stages: vec![
                    GraphStageSpec::StereoBalance { balance: 0.3 },
                    GraphStageSpec::HardClip { threshold: 0.7 },
                ],
            },
        ],
    }
}

#[derive(Clone, Debug)]
struct ServerDemoPluginSandboxAssembly {
    request: PluginSandboxRequest,
    plugin_format: PluginFormat,
    bound_node_ids: Vec<&'static str>,
}

impl ServerDemoPluginSandboxAssembly {
    fn spec(&self) -> PluginSandboxSpec {
        PluginSandboxSpec {
            sandbox_id: self.request.sandbox_id.clone(),
            plugin_format: self.plugin_format,
            plugin_type_id: None,
        }
    }
}

#[derive(Clone, Debug)]
struct ServerDemoRuntimeAssembly {
    graph: GraphProjection,
    plugin_sandboxes: Vec<ServerDemoPluginSandboxAssembly>,
}

impl ServerDemoRuntimeAssembly {
    fn primary_sandbox(&self) -> &ServerDemoPluginSandboxAssembly {
        self.plugin_sandboxes
            .first()
            .expect("server demo assembly should define a primary sandbox")
    }

    fn active_plugin_sandbox_count(&self) -> u32 {
        self.plugin_sandboxes.len() as u32
    }

    fn plugin_bindings(&self) -> PluginBackedNodeBindingProjection {
        PluginBackedNodeBindingProjection {
            graph_id: self.graph.graph_id.clone(),
            bindings: self
                .plugin_sandboxes
                .iter()
                .flat_map(|sandbox| {
                    sandbox
                        .bound_node_ids
                        .iter()
                        .map(|node_id| PluginBackedNodeBinding {
                            node_id: (*node_id).into(),
                            sandbox_id: sandbox.request.sandbox_id.clone(),
                        })
                })
                .collect(),
        }
    }
}

fn server_demo_runtime_assembly() -> ServerDemoRuntimeAssembly {
    ServerDemoRuntimeAssembly {
        graph: server_demo_graph_projection(),
        plugin_sandboxes: vec![ServerDemoPluginSandboxAssembly {
            request: PluginSandboxRequest::new(
                "server-default-sandbox",
                PluginFormat::Clap,
                SandboxPolicy::Strict,
            ),
            plugin_format: PluginFormat::Clap,
            bound_node_ids: vec!["drive"],
        }],
    }
}

fn runtime_watchdog_trigger(reason: WatchdogTriggerReason) -> RuntimeWatchdogTrigger {
    match reason {
        WatchdogTriggerReason::DeadlineMisses => RuntimeWatchdogTrigger::DeadlineMisses,
        WatchdogTriggerReason::HeartbeatMisses => RuntimeWatchdogTrigger::HeartbeatMisses,
    }
}

fn transport_attach_intent(processing_epoch: u64) -> TransportAttachIntent {
    if processing_epoch > 1 {
        TransportAttachIntent::RecoveryOverlap
    } else {
        TransportAttachIntent::SteadyState
    }
}

fn lifecycle_stage_for_request(
    payload: &PluginMessagePayload,
) -> Option<PluginSandboxLifecycleStage> {
    match payload {
        PluginMessagePayload::SandboxHandshakeRequest { .. } => {
            Some(PluginSandboxLifecycleStage::SandboxHandshaken)
        }
        PluginMessagePayload::LoadPluginTypeRequest { .. } => {
            Some(PluginSandboxLifecycleStage::PluginTypeLoaded)
        }
        PluginMessagePayload::CreateInstanceRequest { .. } => {
            Some(PluginSandboxLifecycleStage::InstanceCreated)
        }
        PluginMessagePayload::PrepareInstanceRequest { .. } => {
            Some(PluginSandboxLifecycleStage::InstancePrepared)
        }
        PluginMessagePayload::ActivateInstanceRequest { .. } => {
            Some(PluginSandboxLifecycleStage::InstanceActivated)
        }
        PluginMessagePayload::DeactivateInstanceRequest { .. } => {
            Some(PluginSandboxLifecycleStage::InstanceDeactivated)
        }
        PluginMessagePayload::ResetInstanceRequest { .. } => {
            Some(PluginSandboxLifecycleStage::InstanceReset)
        }
        PluginMessagePayload::DestroyInstanceRequest { .. } => {
            Some(PluginSandboxLifecycleStage::InstanceDestroyed)
        }
        _ => None,
    }
}

fn record_runtime_fault(runtime: &mut SignalRuntime, failure: &signal_ipc::PluginMessageEnvelope) {
    if let signal_ipc::PluginMessagePayload::SandboxFailure {
        sandbox_id,
        detail,
        processing_epoch,
        fault,
        instance_state,
        ..
    } = &failure.payload
    {
        let kind = runtime_plugin_fault_kind(Some(fault));
        runtime.record_plugin_sandbox_fault(
            sandbox_id.clone(),
            kind,
            detail.clone(),
            *processing_epoch,
        );
        if let Some(instance_state) = instance_state.as_ref() {
            runtime.record_plugin_sandbox_instance_state(plugin_instance_state_record(
                sandbox_id,
                *processing_epoch,
                instance_state,
            ));
        }
        if let Some(classification) = classify_sandbox_failure(failure) {
            runtime.record_sandbox_operation_failure(
                classification.sandbox_id,
                classification.lease_id,
                classification.processing_epoch,
                classification.operation,
                classification.error_kind,
                map_clap_sandbox_failure_stage(classification.stage),
                classification.detail,
            );
        }
    }
}

fn runtime_plugin_fault_kind(fault: Option<&signal_ipc::PluginFaultPayload>) -> PluginFaultKind {
    match fault.map(|fault| fault.kind.as_str()) {
        Some("timeout") => PluginFaultKind::Timeout,
        Some("crash") => PluginFaultKind::Crash,
        _ => PluginFaultKind::ProtocolViolation,
    }
}

fn plugin_instance_state_record(
    sandbox_id: &str,
    processing_epoch: Option<u64>,
    state: &PluginInstanceStatePayload,
) -> PluginSandboxInstanceStateRecord {
    let processing = state.processing.as_ref();
    PluginSandboxInstanceStateRecord {
        sandbox_id: sandbox_id.to_string(),
        plugin_type_id: state.plugin_type_id.clone(),
        instance_id: state.instance_id.clone(),
        lifecycle_state: state.lifecycle_state.clone(),
        readiness_state: state.readiness_state.clone(),
        degraded_reasons: state.degraded_reasons.clone(),
        active: state.active,
        processing_epoch,
        processing_sample_rate_hz: processing.map(|processing| processing.sample_rate_hz),
        processing_max_block_frames: processing.map(|processing| processing.max_block_frames),
        audio_inputs: processing.map(|processing| processing.io_layout.audio_inputs),
        audio_outputs: processing.map(|processing| processing.io_layout.audio_outputs),
        midi_inputs: processing.map(|processing| processing.io_layout.midi_inputs),
        midi_outputs: processing.map(|processing| processing.io_layout.midi_outputs),
        last_fault: state
            .last_fault
            .as_ref()
            .map(|fault| PluginSandboxInstanceFaultRecord {
                kind: fault.kind.clone(),
                severity: fault.severity.clone(),
                message: fault.message.clone(),
            }),
    }
}

fn plugin_instance_state_record_from_response(
    sandbox_id: &str,
    processing_epoch: Option<u64>,
    response: &PluginMessageEnvelope,
) -> Option<PluginSandboxInstanceStateRecord> {
    match &response.payload {
        PluginMessagePayload::CreateInstanceResponse { instance_state, .. }
        | PluginMessagePayload::PrepareInstanceResponse { instance_state, .. }
        | PluginMessagePayload::ActivateInstanceResponse { instance_state, .. }
        | PluginMessagePayload::DeactivateInstanceResponse { instance_state, .. }
        | PluginMessagePayload::ResetInstanceResponse { instance_state, .. }
        | PluginMessagePayload::DestroyInstanceResponse { instance_state, .. } => Some(
            plugin_instance_state_record(sandbox_id, processing_epoch, instance_state),
        ),
        PluginMessagePayload::HeartbeatResponse {
            instance_state: Some(instance_state),
            ..
        } => Some(plugin_instance_state_record(
            sandbox_id,
            processing_epoch,
            instance_state,
        )),
        PluginMessagePayload::SandboxFailure {
            instance_state: Some(instance_state),
            ..
        } => Some(plugin_instance_state_record(
            sandbox_id,
            processing_epoch,
            instance_state,
        )),
        _ => None,
    }
}

fn map_clap_sandbox_failure_stage(stage: ClapSandboxFailureStage) -> SandboxOperationFailureStage {
    match stage {
        ClapSandboxFailureStage::PrepareAttach => SandboxOperationFailureStage::PrepareAttach,
        ClapSandboxFailureStage::ProcessAttach => SandboxOperationFailureStage::ProcessAttach,
        ClapSandboxFailureStage::ProcessFlush => SandboxOperationFailureStage::ProcessFlush,
        ClapSandboxFailureStage::ProcessProtocolViolation => {
            SandboxOperationFailureStage::ProcessProtocolViolation
        }
        ClapSandboxFailureStage::ControlProtocolViolation => {
            SandboxOperationFailureStage::ControlProtocolViolation
        }
    }
}

fn build_fault_envelope(
    sandbox_id: &str,
    instance_id: &str,
    lease_id: &str,
    processing_epoch: u64,
    fault: FaultInjection,
) -> PluginMessageEnvelope {
    let (error_kind, detail) = match fault {
        FaultInjection::Timeout => ("timeout", "sandbox exceeded block deadline"),
        FaultInjection::Crash => ("crash", "sandbox process exited unexpectedly"),
        FaultInjection::HeartbeatMiss
        | FaultInjection::RecoveryDeferredTeardownFailure
        | FaultInjection::RecoveryDeferredTeardownThenCleanup
        | FaultInjection::RecoveryDeferredTeardownCleanupRetry
        | FaultInjection::RecoveryTeardownFailure
        | FaultInjection::RecoveryRestartFailure
        | FaultInjection::RecoveryOverlapContention
        | FaultInjection::RecoveryInterleavedFailures
        | FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: _,
        }
        | FaultInjection::MixedWatchdogEpisodes {
            restart_episodes: _,
        } => ("timeout", "sandbox heartbeat watchdog threshold exceeded"),
    };
    sandbox_failure_event(
        sandbox_id,
        Some(instance_id.into()),
        "processBlock",
        error_kind,
        detail,
        Some(processing_epoch),
        Some(lease_id.into()),
        None,
    )
}

fn extract_prepare_metadata(
    responses: &[PluginMessageEnvelope],
) -> (String, Option<SharedMemoryTransportPayload>) {
    responses
        .iter()
        .find_map(|response| match &response.payload {
            PluginMessagePayload::PrepareInstanceResponse {
                shared_memory_lease_id,
                shared_memory_transport,
                ..
            } => Some((
                shared_memory_lease_id.clone(),
                Some(shared_memory_transport.clone()),
            )),
            _ => None,
        })
        .unwrap_or_default()
}

fn runtime_error_from_failure(failure: &signal_ipc::PluginMessageEnvelope) -> RuntimeError {
    match &failure.payload {
        signal_ipc::PluginMessagePayload::SandboxFailure { detail, fault, .. } => RuntimeError {
            kind: match Some(fault).map(|fault| fault.kind.as_str()) {
                Some("timeout") => signal_runtime::RuntimeErrorKind::Timeout,
                Some("crash") | Some("fatal") => signal_runtime::RuntimeErrorKind::Fatal,
                _ => signal_runtime::RuntimeErrorKind::PluginFailure,
            },
            message: detail.clone(),
        },
        _ => RuntimeError {
            kind: signal_runtime::RuntimeErrorKind::PluginFailure,
            message: "plugin sandbox lifecycle failed".into(),
        },
    }
}

fn runtime_error_from_io(error: std::io::Error) -> RuntimeError {
    RuntimeError {
        kind: signal_runtime::RuntimeErrorKind::ResourceUnavailable,
        message: error.to_string(),
    }
}

fn record_broker_failure_and_convert(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: Option<String>,
    processing_epoch: Option<u64>,
    block_sequence: Option<u64>,
    stage: BrokerFailureStage,
    error: std::io::Error,
) -> RuntimeError {
    let detail = error.to_string();
    runtime.record_broker_failure(
        sandbox_id,
        lease_id,
        processing_epoch,
        block_sequence,
        stage,
        detail.clone(),
    );
    RuntimeError {
        kind: signal_runtime::RuntimeErrorKind::ResourceUnavailable,
        message: detail,
    }
}

impl RuntimeSupervisorApi for ServerRuntimeHost {
    fn start_plugin_scan(
        &mut self,
        request: PluginScanRequest,
    ) -> Result<signal_runtime::ScanHandle, RuntimeError> {
        let handle = self.runtime.record_plugin_scan_request(&request);
        let discovered_types = self.discovered_plugins_for_scan(&request);
        self.runtime
            .record_plugin_scan_results(handle, discovered_types);
        self.supervisor.scans_started = handle.0;
        self.supervisor.last_scan_roots = request.roots;
        Ok(handle)
    }

    fn ensure_plugin_sandbox(
        &mut self,
        request: PluginSandboxSpec,
    ) -> Result<signal_runtime::SandboxHandle, RuntimeError> {
        self.supervisor.sandboxes = self.supervisor.sandboxes.saturating_add(1);
        self.runtime.record_plugin_sandbox_spec(&request);
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        self.supervisor.last_sandbox_id = Some(request.sandbox_id);
        Ok(signal_runtime::SandboxHandle(self.supervisor.sandboxes))
    }

    fn start_recording_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
    ) -> Result<(), RuntimeError> {
        self.runtime.start_recording_capture(request)
    }

    fn finish_recording_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError> {
        self.runtime.finish_recording_capture()
    }

    fn cancel_recording_capture(&mut self) -> Result<(), RuntimeError> {
        self.runtime.cancel_recording_capture()
    }

    fn reconcile_media_assets(
        &mut self,
        assets: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError> {
        self.runtime.reconcile_media_assets(assets)
    }

    fn start_media_preview(&mut self, asset_id: &str) -> Result<(), RuntimeError> {
        self.runtime.start_media_preview(asset_id)
    }

    fn stop_media_preview(&mut self) -> Result<(), RuntimeError> {
        self.runtime.stop_media_preview()
    }

    fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError> {
        self.runtime.reconcile_warp_clips(clips)
    }

    fn reconcile_clip_processing_clips(
        &mut self,
        clips: Vec<RuntimeClipProcessingRegistration>,
    ) -> Result<(), RuntimeError> {
        self.runtime.reconcile_clip_processing_clips(clips)
    }

    fn render_offline(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError> {
        self.runtime.render_offline(request)
    }

    fn render_offline_with_checkpoints(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionReceipt, RuntimeError> {
        self.runtime.render_offline_with_checkpoints(request)
    }

    fn begin_offline_render_execution(
        &mut self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime.begin_offline_render_execution(request)
    }

    fn pause_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime.pause_offline_render_execution(request_id)
    }

    fn resume_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime.resume_offline_render_execution(request_id)
    }

    fn interrupt_offline_render_execution(
        &mut self,
        request_id: &str,
        reason: String,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime
            .interrupt_offline_render_execution(request_id, reason)
    }

    fn advance_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError> {
        self.runtime.advance_offline_render_execution(request_id)
    }

    fn cancel_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeError> {
        self.runtime.cancel_offline_render_execution(request_id)
    }

    fn render_offline_queue(
        &self,
        requests: Vec<RuntimeOfflineRenderRequest>,
    ) -> Result<RuntimeOfflineRenderQueueResult, RuntimeError> {
        self.runtime.render_offline_queue(requests)
    }

    fn purge_offline_render_artifacts(
        &self,
        request: RuntimeOfflineRenderPurgeRequest,
    ) -> Result<RuntimeOfflineRenderPurgeReceipt, RuntimeError> {
        self.runtime.purge_offline_render_artifacts(request)
    }

    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError> {
        self.supervisor.teardowns = self.supervisor.teardowns.saturating_add(1);
        self.supervisor.last_sandbox_id = Some(sandbox_id.to_string());
        self.runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::SandboxTeardown,
            None,
        );
        Ok(())
    }

    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError> {
        self.supervisor.restarts = self.supervisor.restarts.saturating_add(1);
        self.supervisor.last_sandbox_id = Some(sandbox_id.to_string());
        self.runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::SandboxRestarted,
            None,
        );
        Ok(())
    }

    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError> {
        self.supervisor.backend_policy = Some(request.tier);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{server_demo_runtime_assembly, LifecycleRunSummary, ServerRuntimeHost};
    use signal_plugin::{CompletionState, PluginFormat, WatchdogTriggerReason};
    use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
    use signal_runtime::{
        BackendPolicyOverride, BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage,
        CompletionSlotStage, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
        PluginSandboxLifecycleStage, PluginSandboxTransportStage, PluginScanRequest,
        RecoveryRestartIntent, RuntimeConfig, RuntimeConfigRequest, RuntimeErrorKind,
        RuntimeLifecycleApi, RuntimeObservationApi, RuntimeProjectionApi, RuntimeReadiness,
        RuntimeSupervisorApi, RuntimeSupervisorReport, SandboxOperationFailureStage, SignalRuntime,
        StopReason, TransportAttachIntent,
    };
    use std::path::Path;

    fn assert_runtime_automation_values(
        supervisor: &RuntimeSupervisorReport,
        value_events: usize,
        modulation_events: usize,
        gesture_begin_events: usize,
        gesture_end_events: usize,
        first_value: f32,
        last_value: f32,
        last_modulation: f32,
    ) {
        let snapshot = &supervisor.observation.automation_snapshot;
        assert_eq!(snapshot.parameter_id, 4096);
        assert_eq!(snapshot.value_events, value_events);
        assert_eq!(snapshot.modulation_events, modulation_events);
        assert_eq!(snapshot.gesture_begin_events, gesture_begin_events);
        assert_eq!(snapshot.gesture_end_events, gesture_end_events);
        assert!(snapshot
            .first_value
            .is_some_and(|observed| (observed - first_value).abs() < 1.0e-6));
        assert!(snapshot
            .last_value
            .is_some_and(|observed| (observed - last_value).abs() < 1.0e-6));
        assert!(snapshot
            .last_modulation
            .is_some_and(|observed| (observed - last_modulation).abs() < 1.0e-6));
    }

    fn assert_runtime_automation_continuity(
        supervisor: &RuntimeSupervisorReport,
        first_epoch: u64,
        last_epoch: u64,
        epochs: &[u64],
        lease_rollovers: usize,
    ) {
        let snapshot = &supervisor.observation.automation_snapshot;
        assert_eq!(snapshot.first_epoch, Some(first_epoch));
        assert_eq!(snapshot.last_epoch, Some(last_epoch));
        assert_eq!(snapshot.segment_count, epochs.len());
        assert_eq!(snapshot.segment_epochs, epochs);
        assert_eq!(snapshot.lease_rollovers, lease_rollovers);
    }

    fn assert_runtime_sequence_continuity(
        supervisor: &RuntimeSupervisorReport,
        epochs: &[u64],
        first_block_sequence: u64,
        last_block_sequence: u64,
        sequence_gaps: usize,
        lease_rollovers: usize,
    ) {
        let timeline = &supervisor
            .observation
            .timeline_snapshot
            .block_sequence_continuity;
        assert_eq!(timeline.segment_count(), epochs.len());
        assert_eq!(timeline.segment_epochs(), epochs);
        assert_eq!(timeline.first_block_sequence(), Some(first_block_sequence));
        assert_eq!(timeline.last_block_sequence(), Some(last_block_sequence));
        assert_eq!(timeline.sequence_gaps, sequence_gaps);
        assert_eq!(timeline.lease_rollovers, lease_rollovers);
    }

    fn prepare_server_host_with_lifecycle() -> (
        ServerRuntimeHost,
        ClapBlockProtocol,
        ClapSandboxLifecycleHarness,
        LifecycleRunSummary,
    ) {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let mut runtime_config = RuntimeConfigRequest::new(
            host.runtime.config().sample_rate.0,
            host.runtime.config().graph.block_size,
        );
        runtime_config.anticipative_enabled = false;
        host.runtime
            .handshake(HandshakeRequest {
                client_version: "signal-host-server".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(192_000),
            })
            .expect("handshake");
        host.runtime.configure(runtime_config).expect("configure");
        let assembly = server_demo_runtime_assembly();
        host.runtime
            .apply_graph_projection(assembly.graph.clone())
            .expect("graph projection");

        let hardware_request = signal_hardware::HardwareConfigRequest::new(
            host.runtime.config().sample_rate.0,
            host.runtime.config().graph.block_size,
            signal_hardware::BackendPolicyTier::Tier0InHost,
        );
        host.runtime
            .apply_hardware_config(hardware_request)
            .expect("hardware config");
        host.runtime
            .set_active_output_device("server:virtual-output");
        host.set_backend_policy(BackendPolicyOverride {
            tier: hardware_request.backend_policy,
        })
        .expect("backend policy");
        host.runtime
            .set_backend_policy_tier(hardware_request.backend_policy);
        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["/srv/plugins/clap".into()],
            formats: vec![PluginFormat::Clap],
        })
        .expect("plugin scan");
        for sandbox in &assembly.plugin_sandboxes {
            host.ensure_plugin_sandbox(sandbox.spec())
                .expect("ensure sandbox");
        }
        host.runtime
            .apply_plugin_backed_node_bindings(assembly.plugin_bindings())
            .expect("plugin bindings");
        host.runtime
            .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());
        host.runtime.set_cpu_load_percent(1.2);
        host.runtime.set_graph_latency_ms(1.1);
        host.runtime.start().expect("start runtime");

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:server",
            "instance:server:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            2048,
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let sandbox = assembly.primary_sandbox();
        let run = host
            .run_lifecycle(
                &protocol,
                sandbox.request.sandbox_id.as_str(),
                1,
                &mut lifecycle,
            )
            .expect("lifecycle");
        (host, protocol, lifecycle, run)
    }

    fn prepare_server_host_without_lifecycle() -> (ServerRuntimeHost, ClapBlockProtocol) {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let mut runtime_config = RuntimeConfigRequest::new(
            host.runtime.config().sample_rate.0,
            host.runtime.config().graph.block_size,
        );
        runtime_config.anticipative_enabled = false;
        host.runtime
            .handshake(HandshakeRequest {
                client_version: "signal-host-server".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(192_000),
            })
            .expect("handshake");
        host.runtime.configure(runtime_config).expect("configure");
        let assembly = server_demo_runtime_assembly();
        host.runtime
            .apply_graph_projection(assembly.graph.clone())
            .expect("graph projection");

        let hardware_request = signal_hardware::HardwareConfigRequest::new(
            host.runtime.config().sample_rate.0,
            host.runtime.config().graph.block_size,
            signal_hardware::BackendPolicyTier::Tier0InHost,
        );
        host.runtime
            .apply_hardware_config(hardware_request)
            .expect("hardware config");
        host.runtime
            .set_active_output_device("server:virtual-output");
        host.set_backend_policy(BackendPolicyOverride {
            tier: hardware_request.backend_policy,
        })
        .expect("backend policy");
        host.runtime
            .set_backend_policy_tier(hardware_request.backend_policy);
        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
            formats: vec![PluginFormat::Clap],
        })
        .expect("plugin scan");
        for sandbox in &assembly.plugin_sandboxes {
            host.ensure_plugin_sandbox(sandbox.spec())
                .expect("ensure sandbox");
        }
        host.runtime
            .apply_plugin_backed_node_bindings(assembly.plugin_bindings())
            .expect("plugin bindings");
        host.runtime
            .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());
        host.runtime.set_cpu_load_percent(3.2);
        host.runtime.set_graph_latency_ms(1.1);
        host.runtime.start().expect("start runtime");

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:server",
            "instance:server:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            2048,
        );
        (host, protocol)
    }

    #[test]
    fn server_host_rolls_leases_forward_after_timeout() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_timeout_recovery()
            .expect("timeout recovery boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(summary.execution.teardown_count, 1);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(
            summary.execution.last_completion_state,
            CompletionState::Completed
        );
        assert_eq!(summary.execution.processed_blocks, 10);
        assert_eq!(summary.execution.engine_processed_blocks, 10);
        assert_eq!(summary.execution.last_block_sequence, 9);
        assert_eq!(
            summary.execution.last_engine_graph_id.as_deref(),
            Some("signal.host.server.demo")
        );
        let plugin_state = summary
            .execution
            .last_plugin_state
            .as_ref()
            .expect("plugin instance state should be projected into server summary");
        assert_eq!(plugin_state.plugin_type_id, "plugin:clap:server");
        assert_eq!(plugin_state.instance_id, "instance:server:default");
        assert_eq!(plugin_state.lifecycle_state, "Active");
        assert_eq!(plugin_state.readiness_state, "Ready");
        assert!(plugin_state.active);
        assert_eq!(plugin_state.processing_sample_rate_hz, Some(48_000));
        assert_eq!(plugin_state.processing_max_block_frames, Some(512));
        assert!(plugin_state.last_fault.is_none());
        let observed_plugin_state = supervisor
            .observation
            .observation
            .last_plugin_instance_state()
            .expect("runtime observation should retain typed plugin state");
        assert_eq!(observed_plugin_state.instance_id, "instance:server:default");
        assert_eq!(observed_plugin_state.lifecycle_state, "Active");
        assert_eq!(observed_plugin_state.readiness_state, "Ready");
        assert!(supervisor
            .render_json()
            .contains("\"plugin_instance_state_events\":"));
        assert!(
            summary
                .execution
                .last_engine_output_peak
                .unwrap_or_default()
                <= 0.7
        );
        assert!(summary.execution.last_engine_output_rms.unwrap_or_default() > 0.0);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            Some(1)
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            Some(true)
        );
        assert!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples)
                .unwrap_or_default()
                > 0
        );
        assert_eq!(supervisor.observation.engine_block_snapshot.node_count, 3);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .stateful_node_count,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .latency_node_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .plugin_backed_node_count,
            1
        );
        assert!(
            !supervisor
                .observation
                .engine_block_snapshot
                .anticipative_planning_enabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .inline_realtime_node_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .stateful_realtime_node_count,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_eligible_node_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_semantic_policy,
            signal_runtime::RuntimePreworkServiceSemanticPolicy::Balanced
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_active_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_bound_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_active_bound_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_degraded_bound_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_missing_bound_plugin_sandboxes,
            0
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .planned_nodes
            .iter()
            .any(|node| node.node_id == "drive"
                && node.plugin_sandbox_id.as_deref() == Some("server-default-sandbox")));
        assert_eq!(supervisor.observation.engine_block_snapshot.phase_count, 2);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_phase_count,
            0
        );
        assert_eq!(supervisor.observation.engine_block_snapshot.lane_count, 1);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_lane_count,
            0
        );
        assert_eq!(
            supervisor.observation.engine_block_snapshot.dispatch_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .dispatch_boundary_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prepared_dispatch_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .realtime_dispatch_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .dispatch_handoff_count,
            0
        );
        assert!(
            !supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_enabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_forecast_requested_mode,
            signal_runtime::RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_forecast_mode,
            signal_runtime::RuntimePreworkForecastMode::Disabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_state,
            signal_runtime::RuntimePreworkCacheState::Disabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_freshness_state,
            signal_runtime::RuntimePreworkFreshnessState::Disabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_admissions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_consumptions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_retirement_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_hits,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_misses,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_output_peak,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_admission_processing_epoch,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_admission_block_sequence,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_consumption_processing_epoch,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_consumption_block_sequence,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_retirement_reason,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_retired_unconsumed,
            None
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_valid_until_block_sequence,
            None
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .last_realtime_input_peak
            .is_some());
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .total_latency_samples,
            32
        );
        assert_eq!(summary.last_payload.event_count, 11);
        assert_eq!(summary.last_payload.parameter_event_count, 2);
        assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
        assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
        assert_eq!(summary.last_payload.note_event_count, 1);
        assert_eq!(summary.last_payload.note_expression_event_count, 3);
        assert_eq!(summary.last_payload.midi_event_count, 1);
        assert_eq!(summary.last_payload.generated_event_bytes, 268);
        assert_eq!(summary.last_payload.first_output_sample, Some(9.0));
        assert_eq!(summary.faults.deadline_misses, 2);
        assert_eq!(summary.faults.heartbeat_misses, 0);
        assert!(summary.faults.watchdog_triggered);
        assert_eq!(
            summary.faults.watchdog_trigger_reason,
            Some(WatchdogTriggerReason::DeadlineMisses)
        );
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            1
        );
        assert!(
            !supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert!(summary.transport.shared_memory_lease_id.contains("epoch-2"));
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_recovery_overlap_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .last_admitted_sandbox_id
                .as_deref(),
            Some("server-default-sandbox")
        );
        assert_runtime_automation_values(&supervisor, 8, 8, 2, 6, 0.2, 0.55, 0.10);
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 9, 0, 1);
    }

    #[test]
    fn server_host_rolls_back_replacement_transport_when_recovery_teardown_fails() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_teardown_failure()
            .expect_err("recovery teardown failure should abort");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error
                .message
                .contains("injected old transport teardown failure"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert!(supervisor
            .observation
            .transport_session_summary
            .active_sessions
            .is_empty());
        assert_eq!(
            supervisor
                .observation
                .transport_session_summary
                .current_attached_session_count,
            0
        );
        assert_eq!(supervisor.observation.control_snapshot.restart_count, 0);
    }

    #[test]
    fn server_host_exposes_lingering_detach_fault_state_after_deferred_recovery_teardown_failure() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_deferred_teardown_failure()
            .expect_err("deferred teardown failure should abort");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error
                .message
                .contains("deferred old transport teardown during recovery retry"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_detach_faulted_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions
                .len(),
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions[0]
                .state,
            signal_runtime::TransportSessionState::DetachFaulted
        );
    }

    #[test]
    fn server_host_recovers_after_lingering_deferred_teardown_cleanup() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_recovery_deferred_teardown_then_cleanup()
            .expect("lingering cleanup recovery should succeed");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_lingering_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_detach_faulted_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions
                .len(),
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions[0]
                .state,
            signal_runtime::TransportSessionState::AttachActive
        );
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 9, 0, 1);
    }

    #[test]
    fn server_host_recovers_after_lingering_cleanup_fails_once_more() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_recovery_deferred_teardown_cleanup_retry()
            .expect("cleanup retry recovery should succeed");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_lingering_sessions,
            2
        );
        assert!(supervisor
            .observation
            .observation
            .broker_failure_events
            .iter()
            .any(|failure| {
                failure.stage == BrokerFailureStage::TransportTeardown
                    && failure
                        .detail
                        .contains("injected lingering cleanup retry failure")
            }));
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions[0]
                .state,
            signal_runtime::TransportSessionState::AttachActive
        );
    }

    #[test]
    fn server_host_sweeps_orphan_lingering_sessions_before_overlap_recovery() {
        let (mut host, protocol, mut lifecycle, run) = prepare_server_host_with_lifecycle();
        let orphan_region = host
            .broker
            .create_region("server-orphan-lingering", 256)
            .expect("orphan region");
        let orphan_transport = orphan_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-orphan",
                orphan_transport.region_id.as_str(),
                TransportAttachIntent::RecoveryOverlap,
                Some(orphan_transport.backing_path.clone()),
                Some(orphan_transport.total_bytes),
            )
            .expect("orphan transport session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-orphan",
            orphan_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("replacement rollback linger".into()),
        );

        let recovered = host
            .recover_sandbox(
                &protocol,
                "server-default-sandbox",
                &mut lifecycle,
                &run,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect("orphan lingering sweep recovery");
        let supervisor = host.supervisor_report();

        assert_eq!(recovered.processing_epoch, 2);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .all(|session| session.lease_id != "lease-orphan"));
        assert!(!Path::new(&orphan_transport.backing_path).exists());
    }

    #[test]
    fn server_host_aborts_when_orphan_lingering_cleanup_fails_before_overlap_recovery() {
        let (mut host, protocol, mut lifecycle, run) = prepare_server_host_with_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-orphan",
                "region-orphan-failure",
                TransportAttachIntent::RecoveryOverlap,
                None,
                None,
            )
            .expect("orphan transport session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-orphan",
            "region-orphan-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("replacement rollback linger".into()),
        );

        let error = host
            .recover_sandbox(
                &protocol,
                "server-default-sandbox",
                &mut lifecycle,
                &run,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect_err("orphan lingering cleanup failure should abort recovery");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(error.message.contains("missing backing_path metadata"));
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            1
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == "lease-orphan"));
    }

    #[test]
    fn server_host_cleans_multiple_orphan_lingering_sessions_for_same_sandbox() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let orphan_region_a = host
            .broker
            .create_region("server-orphan-a", 256)
            .expect("orphan region a");
        let orphan_transport_a = orphan_region_a.metadata().clone();
        let orphan_region_b = host
            .broker
            .create_region("server-orphan-b", 256)
            .expect("orphan region b");
        let orphan_transport_b = orphan_region_b.metadata().clone();

        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-orphan-a",
                orphan_transport_a.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(orphan_transport_a.backing_path.clone()),
                Some(orphan_transport_a.total_bytes),
            )
            .expect("orphan session a");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-orphan-a",
            orphan_transport_a.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("orphan a lingering".into()),
        );
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-orphan-b",
                orphan_transport_b.region_id.as_str(),
                TransportAttachIntent::RecoveryOverlap,
                Some(orphan_transport_b.backing_path.clone()),
                Some(orphan_transport_b.total_bytes),
            )
            .expect("orphan session b");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-orphan-b",
            orphan_transport_b.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("orphan b lingering".into()),
        );

        host.cleanup_orphan_lingering_sessions_for_sandbox(
            "server-default-sandbox",
            1,
            None,
            None,
            LingeringCleanupMode::StrictPreAttach,
        )
        .expect("multiple orphan cleanup");

        let supervisor = host.supervisor_report();
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .is_empty());
        assert!(!Path::new(&orphan_transport_a.backing_path).exists());
        assert!(!Path::new(&orphan_transport_b.backing_path).exists());
    }

    #[test]
    fn server_host_reconciles_late_lingering_completion_without_disturbing_active_replacement() {
        let (mut host, protocol) = prepare_server_host_without_lifecycle();
        let late_region = host
            .broker
            .create_region("server-late-lingering", 256)
            .expect("late lingering region");
        let late_transport = late_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-late-origin",
                late_transport.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(late_transport.backing_path.clone()),
                Some(late_transport.total_bytes),
            )
            .expect("late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-late-origin",
            late_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late origin teardown completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered = host
            .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        host.reconcile_late_lingering_sessions_after_start("server-default-sandbox", &recovered);

        let supervisor = host.supervisor_report();
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions
                .len(),
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .active_sessions[0]
                .lease_id,
            recovered.shared_memory_lease_id
        );
        assert!(!Path::new(&late_transport.backing_path).exists());
    }

    #[test]
    fn server_host_keeps_active_replacement_running_when_late_lingering_cleanup_fails() {
        let (mut host, protocol) = prepare_server_host_without_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-late-origin",
                "region-late-origin-failure",
                TransportAttachIntent::SteadyState,
                None,
                None,
            )
            .expect("late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-late-origin",
            "region-late-origin-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late origin teardown completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered = host
            .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        host.reconcile_late_lingering_sessions_after_start("server-default-sandbox", &recovered);

        let supervisor = host.supervisor_report();
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            1
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == recovered.shared_memory_lease_id));
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == "lease-late-origin"));
        assert!(supervisor
            .observation
            .observation
            .broker_failure_events
            .iter()
            .any(|failure| {
                failure.stage == BrokerFailureStage::TransportTeardown
                    && failure.detail.contains("missing backing_path metadata")
            }));
    }

    #[test]
    fn server_host_sweeps_prior_late_lingering_before_next_overlap_recovery() {
        let (mut host, protocol) = prepare_server_host_without_lifecycle();
        let late_region = host
            .broker
            .create_region("server-adjacent-lingering", 256)
            .expect("late lingering region");
        let late_transport = late_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-prior-lingering",
                late_transport.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(late_transport.backing_path.clone()),
                Some(late_transport.total_bytes),
            )
            .expect("prior late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-prior-lingering",
            late_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("prior late completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered_epoch2 = host
            .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");
        let recovered_transport = recovered_epoch2
            .transport
            .as_ref()
            .expect("recovered transport");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            recovered_epoch2.shared_memory_lease_id.as_str(),
            recovered_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(recovered_epoch2.processing_epoch),
            Some("current replacement became lingering before adjacent recovery".into()),
        );

        let recovered_epoch3 = host
            .recover_sandbox(
                &protocol,
                "server-default-sandbox",
                &mut lifecycle,
                &recovered_epoch2,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect("adjacent recovery should sweep prior lingering session");
        let supervisor = host.supervisor_report();

        assert_eq!(recovered_epoch3.processing_epoch, 3);
        assert!(supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .all(|session| session.lease_id != "lease-prior-lingering"));
        assert!(!Path::new(&late_transport.backing_path).exists());
    }

    #[test]
    fn server_host_aborts_adjacent_overlap_recovery_when_prior_late_lingering_lacks_metadata() {
        let (mut host, protocol) = prepare_server_host_without_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "server-default-sandbox",
                "lease-prior-lingering",
                "region-prior-lingering-failure",
                TransportAttachIntent::SteadyState,
                None,
                None,
            )
            .expect("prior late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "server-default-sandbox",
            "lease-prior-lingering",
            "region-prior-lingering-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("prior late completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered_epoch2 = host
            .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        let error = host
            .recover_sandbox(
                &protocol,
                "server-default-sandbox",
                &mut lifecycle,
                &recovered_epoch2,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect_err("adjacent recovery should abort on stale lingering metadata");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(error.message.contains("missing backing_path metadata"));
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            1
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == "lease-prior-lingering"));
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions
            .iter()
            .any(|session| session.lease_id == recovered_epoch2.shared_memory_lease_id));
    }

    #[test]
    fn server_host_rolls_back_replacement_transport_when_recovery_start_fails() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_restart_failure()
            .expect_err("recovery start failure should abort");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error.message.contains("injected replacement start failure"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert!(supervisor
            .observation
            .transport_session_summary
            .active_sessions
            .is_empty());
        assert_eq!(
            supervisor
                .observation
                .transport_session_summary
                .current_attached_session_count,
            0
        );
        assert_eq!(supervisor.observation.control_snapshot.restart_count, 0);
    }

    #[test]
    fn server_host_rolls_back_partial_overlap_when_competing_recovery_attach_is_rejected() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_overlap_contention()
            .expect_err("overlap contention should abort recovery");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error.message.contains("recovery overlap session limit 1"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .last_rejected_sandbox_id
                .as_deref(),
            Some("server-default-sandbox")
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .last_rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));
        assert!(supervisor
            .observation
            .transport_session_summary
            .active_sessions
            .is_empty());
    }

    #[test]
    fn server_host_handles_interleaved_recovery_failures_across_retries() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let error = host
            .boot_with_recovery_interleaved_failures()
            .expect_err("interleaved failures should abort recovery");
        let supervisor = host.supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
        assert!(
            error.message.contains("recovery overlap session limit 1"),
            "unexpected error: {}",
            error.message
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 1);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(!supervisor.observation.control_snapshot.running);
        assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
        assert_eq!(
            supervisor
                .observation
                .diagnostics_snapshot
                .active_plugin_sandboxes,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .peak_attached_sessions,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .last_rejected_sandbox_id
                .as_deref(),
            Some("server-default-sandbox")
        );
        assert!(supervisor
            .observation
            .transport_concurrency_snapshot
            .last_rejection_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));
        assert!(supervisor
            .observation
            .observation
            .broker_failure_events
            .iter()
            .any(|failure| {
                failure.stage == BrokerFailureStage::TransportTeardown
                    && failure.detail.contains("deferred old transport teardown")
            }));
        assert!(supervisor
            .observation
            .transport_session_summary
            .active_sessions
            .is_empty());
    }

    #[test]
    fn server_host_recovers_after_crash() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_crash_recovery()
            .expect("crash recovery boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(summary.execution.teardown_count, 1);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::CrashRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(
            summary.execution.last_completion_state,
            CompletionState::Completed
        );
        assert_eq!(summary.execution.processed_blocks, 9);
        assert_eq!(summary.last_payload.event_count, 11);
        assert_eq!(summary.last_payload.parameter_event_count, 2);
        assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
        assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
        assert_eq!(summary.last_payload.note_event_count, 1);
        assert_eq!(summary.last_payload.note_expression_event_count, 3);
        assert_eq!(summary.last_payload.midi_event_count, 1);
        assert_eq!(summary.last_payload.first_output_sample, Some(8.0));
        assert_eq!(summary.faults.deadline_misses, 0);
        assert_eq!(summary.faults.heartbeat_misses, 0);
        assert!(!summary.faults.watchdog_triggered);
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            0
        );
        assert!(
            !supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert!(summary
            .transport
            .shared_memory_region_id
            .starts_with("region-"));
        assert_runtime_automation_values(&supervisor, 9, 9, 3, 6, 0.1, 0.5, 0.08);
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 8, 0, 1);
    }

    #[test]
    fn server_host_recovers_after_heartbeat_watchdog_trigger() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_heartbeat_miss_recovery()
            .expect("heartbeat recovery boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 2);
        assert_eq!(summary.execution.restart_count, 1);
        assert_eq!(summary.execution.teardown_count, 1);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(
            summary.execution.last_completion_state,
            CompletionState::Completed
        );
        assert_eq!(summary.execution.processed_blocks, 8);
        assert_eq!(summary.execution.last_block_sequence, 9);
        assert_eq!(summary.faults.heartbeat_misses, 2);
        assert_eq!(summary.faults.deadline_misses, 0);
        assert!(summary.faults.watchdog_triggered);
        assert_eq!(
            summary.faults.watchdog_trigger_reason,
            Some(WatchdogTriggerReason::HeartbeatMisses)
        );
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            1
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert!(supervisor.observation.control_snapshot.running);
        assert!(
            !supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert_runtime_automation_values(&supervisor, 8, 8, 2, 6, 0.2, 0.55, 0.10);
        assert_runtime_automation_continuity(&supervisor, 2, 2, &[2], 0);
        assert_runtime_sequence_continuity(&supervisor, &[2], 2, 9, 0, 0);
    }

    #[test]
    fn server_host_enters_safe_mode_after_repeated_watchdog_restarts() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_escalating_heartbeat_failures()
            .expect("escalating heartbeat recovery boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 3);
        assert_eq!(summary.execution.restart_count, 2);
        assert_eq!(summary.execution.teardown_count, 2);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(summary.execution.processed_blocks, 10);
        assert_eq!(summary.execution.last_block_sequence, 13);
        assert_eq!(summary.faults.heartbeat_misses, 4);
        assert!(summary.faults.watchdog_triggered);
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            2
        );
        assert!(
            supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert!(matches!(
            supervisor.observation.readiness,
            signal_runtime::RuntimeReadiness::Degraded { .. }
        ));
        assert_eq!(supervisor.observation.control_snapshot.start_count, 3);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 2);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_runtime_automation_values(&supervisor, 10, 10, 2, 8, 0.2, 0.75, 0.18);
        assert_runtime_automation_continuity(&supervisor, 2, 3, &[2, 3], 1);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3], 2, 13, 0, 1);
    }

    #[test]
    fn server_host_soak_path_rolls_across_multiple_lease_generations() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host.boot_with_watchdog_soak().expect("watchdog soak boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 4);
        assert_eq!(summary.execution.restart_count, 3);
        assert_eq!(summary.execution.teardown_count, 3);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(summary.execution.processed_blocks, 12);
        assert_eq!(summary.execution.last_block_sequence, 17);
        assert_eq!(summary.faults.heartbeat_misses, 6);
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            3
        );
        assert!(
            supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert!(summary.transport.shared_memory_lease_id.contains("epoch-4"));
        assert_eq!(summary.last_payload.first_output_sample, Some(17.0));
        assert!(matches!(
            supervisor.observation.readiness,
            signal_runtime::RuntimeReadiness::Degraded { .. }
        ));
        assert_eq!(supervisor.observation.control_snapshot.start_count, 4);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 3);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(supervisor.recovery_event_count(), 3);
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::RecoveryCycle {
                        intent: RecoveryRestartIntent::WatchdogRecovery,
                        stop_reason: StopReason::DegradedModeRecovery,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::InstanceDeactivated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::InstanceReset,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::InstanceDestroyed,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxTransport {
                        stage: PluginSandboxTransportStage::DetachRequested,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxTransport {
                        stage: PluginSandboxTransportStage::Detached,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::HeartbeatCycle {
                        stage: HeartbeatCycleStage::Missed,
                        ..
                    }
                ))
                .count(),
            6
        );
        assert_eq!(supervisor.block_dispatch_event_count(), 24);
        assert_eq!(supervisor.lease_rollover_event_count(), 2);
        assert_eq!(supervisor.invalidation_event_count(), 6);
        assert_eq!(supervisor.completion_slot_event_count(), 39);
        assert_eq!(supervisor.broker_failure_event_count(), 0);
        assert_eq!(supervisor.sandbox_operation_failure_event_count(), 0);
        assert_eq!(supervisor.transport_fault_event_count(), 0);
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BlockDispatch {
                        stage: BlockDispatchStage::Requested,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BlockDispatch {
                        stage: BlockDispatchStage::Completed,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerInvalidation {
                        stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerInvalidation {
                        stage: BrokerInvalidationStage::LeaseEpochInvalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::ReadyForProcessing,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Processing,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Completed,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Invalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_runtime_automation_values(&supervisor, 12, 12, 2, 10, 0.2, 0.95, 0.26);
        assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
    }

    #[test]
    fn server_host_mixed_watchdog_soak_tracks_deadlines_and_heartbeats() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        let summary = host
            .boot_with_mixed_watchdog_soak()
            .expect("mixed watchdog soak boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.execution.processing_epoch, 4);
        assert_eq!(summary.execution.restart_count, 3);
        assert_eq!(summary.execution.teardown_count, 3);
        assert_eq!(
            summary.execution.last_recovery_intent,
            Some(RecoveryRestartIntent::WatchdogRecovery)
        );
        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(summary.execution.processed_blocks, 14);
        assert_eq!(summary.execution.last_block_sequence, 17);
        assert_eq!(summary.faults.deadline_misses, 2);
        assert_eq!(summary.faults.heartbeat_misses, 4);
        assert_eq!(
            supervisor
                .observation
                .supervision_snapshot
                .watchdog_restart_count,
            3
        );
        assert!(
            supervisor
                .observation
                .supervision_snapshot
                .safe_mode_enabled
        );
        assert_eq!(supervisor.observation.control_snapshot.start_count, 4);
        assert_eq!(supervisor.observation.control_snapshot.stop_count, 3);
        assert_eq!(
            supervisor.observation.control_snapshot.last_stop_reason,
            Some(StopReason::DegradedModeRecovery)
        );
        assert_eq!(supervisor.recovery_event_count(), 3);
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::RecoveryCycle {
                        intent: RecoveryRestartIntent::WatchdogRecovery,
                        stop_reason: StopReason::DegradedModeRecovery,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::TransportTornDown,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                        stage: PluginSandboxLifecycleStage::SandboxRestarted,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxTransport {
                        stage: PluginSandboxTransportStage::DetachRequested,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::PluginSandboxTransport {
                        stage: PluginSandboxTransportStage::Detached,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::HeartbeatCycle {
                        stage: HeartbeatCycleStage::Missed,
                        ..
                    }
                ))
                .count(),
            4
        );
        assert_eq!(supervisor.block_dispatch_event_count(), 28);
        assert_eq!(supervisor.lease_rollover_event_count(), 2);
        assert_eq!(supervisor.invalidation_event_count(), 6);
        assert_eq!(supervisor.completion_slot_event_count(), 45);
        assert_eq!(supervisor.broker_failure_event_count(), 0);
        assert_eq!(supervisor.sandbox_operation_failure_event_count(), 0);
        assert_eq!(supervisor.transport_fault_event_count(), 0);
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BlockDispatch {
                        stage: BlockDispatchStage::Requested,
                        ..
                    }
                ))
                .count(),
            14
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BlockDispatch {
                        stage: BlockDispatchStage::TimedOut,
                        ..
                    }
                ))
                .count(),
            2
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerInvalidation {
                        stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerInvalidation {
                        stage: BrokerInvalidationStage::LeaseEpochInvalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::SandboxOperationFailure {
                        stage: SandboxOperationFailureStage::ProcessAttach,
                        ..
                    }
                ))
                .count(),
            0
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::BrokerFailure {
                        stage: BrokerFailureStage::PayloadRead,
                        ..
                    }
                ))
                .count(),
            0
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::ReadyForProcessing,
                        ..
                    }
                ))
                .count(),
            14
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Processing,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Completed,
                        ..
                    }
                ))
                .count(),
            12
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::TimedOut,
                        ..
                    }
                ))
                .count(),
            2
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::FallbackApplied,
                        ..
                    }
                ))
                .count(),
            2
        );
        assert_eq!(
            supervisor
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    signal_runtime::RuntimeEvent::CompletionSlotTransition {
                        stage: CompletionSlotStage::Invalidated,
                        ..
                    }
                ))
                .count(),
            3
        );
        assert_runtime_automation_values(&supervisor, 14, 14, 2, 12, 0.2, 0.95, 0.26);
        assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
        assert!(supervisor.event_count() > 24);
        assert_eq!(supervisor.supervision_update_count(), 3);
        assert_eq!(supervisor.plugin_fault_count(), 3);
        assert_eq!(
            supervisor
                .observation
                .observation
                .fault_detail_count_containing("heartbeat watchdog"),
            2
        );
        assert_eq!(
            supervisor
                .observation
                .observation
                .fault_detail_count_containing("block deadline"),
            1
        );
        assert_eq!(
            host.runtime()
                .get_supervision_snapshot()
                .last_watchdog_trigger,
            Some(signal_runtime::RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert_eq!(
            supervisor.last_watchdog_trigger(),
            Some(signal_runtime::RuntimeWatchdogTrigger::HeartbeatMisses)
        );
        assert!(summary.transport.shared_memory_lease_id.contains("epoch-4"));
        let rendered = supervisor.render_compact();
        assert!(rendered.contains("readiness=Degraded"));
        assert!(rendered.contains("supervision_updates=3"));
        assert!(rendered.contains("plugin_faults=3"));
        assert!(rendered.contains("last_watchdog=HeartbeatMisses"));
        assert!(rendered.contains(&format!("event_stream={}", supervisor.event_count())));
    }
}
