use signal_graph::{synthetic_stereo_block, GraphNodeExecutionClass, GraphStageSpec};
use signal_hardware::{
    BackendPolicyTier, HardwareBackend, HardwareConfigRequest, HardwareStreamRequest,
    SimulatedHardwareBackend,
};
use signal_ipc::{SharedMemoryBroker, SharedMemoryTransportPayload};
use signal_plugin::{
    CompletionState, PluginFormat, PluginSandboxRequest, SandboxPolicy, SandboxWatchdogState,
    WatchdogOutcome, WatchdogTriggerReason,
};
use signal_plugin_au::{AuHostAdapter, AuHostPlatform};
use signal_plugin_clap::{
    BrokeredBlockOutcome, ClapBlockProtocol, ClapPluginHostAdapter, ClapSandboxLifecycleHarness,
};
use signal_plugin_lv2::{Lv2HostAdapter, Lv2HostPlatform};
use signal_plugin_vst3::{Vst3HostAdapter, Vst3HostPlatform};
use signal_primitives::FrameCount;
use signal_runtime::{
    BackendPolicyOverride, BlockDispatchStage, BrokerFailureStage, CompletionSlotStage,
    GraphNodeProjection, GraphProjection, HandshakeRequest, HeartbeatCycleStage,
    PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxInstanceStateRecord,
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest,
    RecoveryRestartIntent, RuntimeClipProcessingRegistration, RuntimeConfigRequest, RuntimeError,
    RuntimeEventRecorder, RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState,
    RuntimeHostAudioTransferPolicy, RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain,
    RuntimeHostClockDriftState, RuntimeHostClockFallbackState, RuntimeHostClockTransitionState,
    RuntimeHostClockingSummary, RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
    RuntimeHostHardwareSummary, RuntimeHostIoSummary, RuntimeHostLatencySummary,
    RuntimeLifecycleApi, RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeOfflineRenderExecutionProgressReceipt,
    RuntimeOfflineRenderExecutionReceipt, RuntimeOfflineRenderPurgeReceipt,
    RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderQueueResult, RuntimeOfflineRenderRequest,
    RuntimeOfflineRenderResult, RuntimePluginDiscoveredTypeRecord, RuntimePreworkServicePressure,
    RuntimeProjectionApi, RuntimeRecordingCaptureCommitReceipt,
    RuntimeRecordingCaptureStartRequest, RuntimeSupervisorApi, RuntimeSupervisorReport,
    RuntimeWarpClipRegistration, SignalRuntime, StopReason, WatchdogRestartRecord,
};

#[path = "host_support.rs"]
mod host_support;
use host_support::{
    build_fault_envelope, plugin_instance_state_record_from_response,
    record_broker_failure_and_convert, record_runtime_fault, runtime_au_discovered_type_record,
    runtime_error_from_failure, runtime_host_clock_source, runtime_host_lifecycle_ownership,
    runtime_host_restart_policy, runtime_lv2_discovered_type_record,
    runtime_plugin_discovered_type_record, runtime_plugin_format_platform_coverage,
    runtime_vst3_discovered_type_record, runtime_watchdog_trigger,
};

const WATCHDOG_TRIGGER_WINDOW_BLOCKS: u64 = 3;
const STEADY_STATE_BLOCKS: u64 = 8;
const SOAK_RESTART_EPISODES: u32 = 3;
const INTER_EPISODE_CONTINUITY_BLOCKS: u64 = 2;

fn samples_to_ms(samples: u32, sample_rate_hz: u32) -> f32 {
    if sample_rate_hz == 0 {
        0.0
    } else {
        samples as f32 * 1_000.0 / sample_rate_hz as f32
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
pub(crate) enum RecoveryFailureInjection {
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
    au: AuHostAdapter,
    lv2: Lv2HostAdapter,
    vst3: Vst3HostAdapter,
    supervisor: ServerSupervisorState,
    events: RuntimeEventRecorder,
}

impl ServerRuntimeHost {
    pub fn new(runtime: SignalRuntime) -> Self {
        let events = RuntimeEventRecorder::default();
        let mut runtime = runtime;
        runtime.subscribe(Box::new(events.clone()));
        runtime.record_plugin_format_platform_coverage(runtime_plugin_format_platform_coverage());

        Self {
            runtime,
            broker: SharedMemoryBroker::default(),
            au: AuHostAdapter::default(),
            lv2: Lv2HostAdapter::default(),
            vst3: Vst3HostAdapter::default(),
            supervisor: ServerSupervisorState::default(),
            events,
        }
    }

    fn discovered_plugins_for_scan(
        &self,
        request: &PluginScanRequest,
    ) -> Vec<RuntimePluginDiscoveredTypeRecord> {
        let mut discovered = Vec::new();
        let include_clap =
            request.formats.is_empty() || request.formats.contains(&PluginFormat::Clap);
        if include_clap {
            let clap = ClapPluginHostAdapter::default();
            discovered.extend(
                ["plugin:clap:server", "plugin:clap:sandbox"]
                    .into_iter()
                    .filter_map(|plugin_type_id| clap.discover_plugin_type(plugin_type_id))
                    .map(runtime_plugin_discovered_type_record),
            );
        }

        let include_vst3 =
            request.formats.is_empty() || request.formats.contains(&PluginFormat::Vst3);
        if include_vst3 {
            discovered.extend(
                self.vst3
                    .discover_plugins_for_roots(Vst3HostPlatform::Linux, &request.roots)
                    .into_iter()
                    .map(runtime_vst3_discovered_type_record),
            );
        }

        let include_lv2 =
            request.formats.is_empty() || request.formats.contains(&PluginFormat::Lv2);
        if include_lv2 {
            discovered.extend(
                self.lv2
                    .discover_plugins_for_roots(Lv2HostPlatform::Linux, &request.roots)
                    .into_iter()
                    .map(runtime_lv2_discovered_type_record),
            );
        }

        let include_au = request.formats.is_empty() || request.formats.contains(&PluginFormat::Au);
        if include_au {
            discovered.extend(
                self.au
                    .discover_plugins_for_roots(AuHostPlatform::MacOs, &request.roots)
                    .into_iter()
                    .map(runtime_au_discovered_type_record),
            );
        }

        discovered
    }

    fn ensure_au_sandbox_session(&mut self, request: &PluginSandboxSpec) {
        let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
            return;
        };
        let Some(discovered) = self.au.discover_plugin_type(plugin_type_id) else {
            return;
        };
        let instance = self.au.instantiate_plugin(
            &discovered,
            &format!("instance:server:au:{}", request.sandbox_id),
        );
        let session = self.au.prepare_session(
            &instance,
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size as u32,
        );

        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxHandshaken,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstanceCreated,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstancePrepared,
            None,
        );
        self.runtime
            .record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
                sandbox_id: request.sandbox_id.clone(),
                plugin_type_id: instance.plugin_type_id.0.clone(),
                instance_id: instance.instance_id.0.clone(),
                lifecycle_state: "Prepared".into(),
                readiness_state: "Ready".into(),
                degraded_reasons: Vec::new(),
                active: true,
                processing_epoch: None,
                processing_sample_rate_hz: Some(session.sample_rate_hz),
                processing_max_block_frames: Some(session.max_block_frames),
                audio_inputs: Some(session.io_layout.audio_inputs),
                audio_outputs: Some(session.io_layout.audio_outputs),
                midi_inputs: Some(session.io_layout.midi_inputs),
                midi_outputs: Some(session.io_layout.midi_outputs),
                last_fault: None,
            });
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::TransportAttached,
            None,
        );
        self.runtime.record_plugin_sandbox_transport(
            request.sandbox_id.as_str(),
            format!("lease:{}", request.sandbox_id),
            format!("region:{}", request.sandbox_id),
            PluginSandboxTransportStage::Attached,
            None,
            Some(session.summary),
        );
    }

    fn ensure_lv2_sandbox_session(&mut self, request: &PluginSandboxSpec) {
        let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
            return;
        };
        let Some(discovered) = self.lv2.discover_plugin_type(plugin_type_id) else {
            return;
        };
        let instance = self.lv2.instantiate_plugin(
            &discovered,
            &format!("instance:server:lv2:{}", request.sandbox_id),
        );
        let session = self.lv2.prepare_session(
            &instance,
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size as u32,
        );

        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxHandshaken,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstanceCreated,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstancePrepared,
            None,
        );
        self.runtime
            .record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
                sandbox_id: request.sandbox_id.clone(),
                plugin_type_id: instance.plugin_type_id.0.clone(),
                instance_id: instance.instance_id.0.clone(),
                lifecycle_state: "Prepared".into(),
                readiness_state: "Ready".into(),
                degraded_reasons: Vec::new(),
                active: true,
                processing_epoch: None,
                processing_sample_rate_hz: Some(session.sample_rate_hz),
                processing_max_block_frames: Some(session.max_block_frames),
                audio_inputs: Some(session.io_layout.audio_inputs),
                audio_outputs: Some(session.io_layout.audio_outputs),
                midi_inputs: Some(session.io_layout.midi_inputs),
                midi_outputs: Some(session.io_layout.midi_outputs),
                last_fault: None,
            });
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::TransportAttached,
            None,
        );
        self.runtime.record_plugin_sandbox_transport(
            request.sandbox_id.as_str(),
            format!("lease:{}", request.sandbox_id),
            format!("region:{}", request.sandbox_id),
            PluginSandboxTransportStage::Attached,
            None,
            Some(session.summary),
        );
    }

    fn ensure_vst3_sandbox_session(&mut self, request: &PluginSandboxSpec) {
        let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
            return;
        };
        let Some(discovered) = self.vst3.discover_plugin_type(plugin_type_id) else {
            return;
        };
        let instance = self.vst3.instantiate_plugin(
            &discovered,
            &format!("instance:server:vst3:{}", request.sandbox_id),
        );
        let session = self.vst3.prepare_session(
            &instance,
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size as u32,
        );

        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxHandshaken,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstanceCreated,
            None,
        );
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::InstancePrepared,
            None,
        );
        self.runtime
            .record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
                sandbox_id: request.sandbox_id.clone(),
                plugin_type_id: instance.plugin_type_id.0.clone(),
                instance_id: instance.instance_id.0.clone(),
                lifecycle_state: "Prepared".into(),
                readiness_state: "Ready".into(),
                degraded_reasons: Vec::new(),
                active: true,
                processing_epoch: None,
                processing_sample_rate_hz: Some(session.sample_rate_hz),
                processing_max_block_frames: Some(session.max_block_frames),
                audio_inputs: Some(session.io_layout.audio_inputs),
                audio_outputs: Some(session.io_layout.audio_outputs),
                midi_inputs: Some(session.io_layout.midi_inputs),
                midi_outputs: Some(session.io_layout.midi_outputs),
                last_fault: None,
            });
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::TransportAttached,
            None,
        );
        self.runtime.record_plugin_sandbox_transport(
            request.sandbox_id.as_str(),
            format!("lease:{}", request.sandbox_id),
            format!("region:{}", request.sandbox_id),
            PluginSandboxTransportStage::Attached,
            None,
            Some(session.summary),
        );
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
        self.runtime.record_plugin_event_summary(
            run.processing_epoch,
            run.shared_memory_lease_id.as_str(),
            block_sequence,
            stored_result.result.generated_event_bytes,
            event_summary,
        );
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

    pub fn runtime(&self) -> &SignalRuntime {
        &self.runtime
    }

    #[allow(dead_code)]
    pub fn observation_diagnostics(&self) -> RuntimeObservationDiagnostics {
        self.events.diagnostics()
    }

    #[allow(dead_code)]
    pub fn observation_report(&self) -> RuntimeObservationReport {
        let observation = RuntimeObservationReport::capture(&self.runtime, &self.events);
        let host_io = self.host_io_summary(&observation);
        let jack_host_io = self.jack_host_io_summary(&observation);
        observation
            .with_linux_backend_session_snapshot(&host_io)
            .with_pipewire_alsa_parity_snapshot(&host_io)
            .with_jack_coordination_snapshot(&jack_host_io)
            .with_external_midi_snapshot(
                signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot::empty(
                    "signal-host-server",
                ),
            )
    }

    pub fn supervisor_report(&self) -> RuntimeSupervisorReport {
        let mut report = RuntimeSupervisorReport::capture(&self.runtime, &self.events);
        let host_io = self.host_io_summary(&report.observation);
        let jack_host_io = self.jack_host_io_summary(&report.observation);
        report.observation = report
            .observation
            .with_linux_backend_session_snapshot(&host_io)
            .with_pipewire_alsa_parity_snapshot(&host_io)
            .with_jack_coordination_snapshot(&jack_host_io)
            .with_external_midi_snapshot(
                signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot::empty(
                    "signal-host-server",
                ),
            );
        report
    }

    fn host_io_summary(&self, observation: &RuntimeObservationReport) -> RuntimeHostIoSummary {
        self.simulated_linux_host_io_summary(
            observation,
            SimulatedHardwareBackend::linux_pipewire_duplex(),
            HardwareStreamRequest::new_output("pipewire:default-graph", 48_000, 256)
                .with_input_channels(2)
                .with_output_channels(2),
        )
    }

    fn jack_host_io_summary(&self, observation: &RuntimeObservationReport) -> RuntimeHostIoSummary {
        self.simulated_linux_host_io_summary(
            observation,
            SimulatedHardwareBackend::linux_jack_duplex(),
            HardwareStreamRequest::new_output("jack:graph-main", 48_000, 256)
                .with_input_channels(2)
                .with_output_channels(2),
        )
    }

    fn simulated_linux_host_io_summary(
        &self,
        observation: &RuntimeObservationReport,
        backend: SimulatedHardwareBackend,
        request: HardwareStreamRequest,
    ) -> RuntimeHostIoSummary {
        let diagnostics = backend.diagnostics();
        let stream = backend
            .negotiate_stream(&request)
            .expect("server host linux baseline should negotiate");
        let graph_latency_samples = observation.engine_block_snapshot.total_latency_samples;
        let output_latency_samples = stream.latency.output_latency_samples;
        let input_latency_samples = stream.latency.input_latency_samples;
        let round_trip_latency_samples = stream.latency.round_trip_latency_samples;
        let estimated_output_latency_samples =
            output_latency_samples.saturating_add(graph_latency_samples);
        let estimated_round_trip_latency_samples =
            match (input_latency_samples, round_trip_latency_samples) {
                (_, Some(round_trip)) => Some(round_trip.saturating_add(graph_latency_samples)),
                (Some(input_latency), None) => Some(
                    input_latency
                        .saturating_add(output_latency_samples)
                        .saturating_add(graph_latency_samples),
                ),
                (None, None) => None,
            };
        let endpoint_topology = RuntimeHostEndpointTopology::Duplex;
        RuntimeHostIoSummary {
            hardware: RuntimeHostHardwareSummary {
                backend_identity: stream.device.backend_identity,
                backend_name: stream.device.backend_name.into(),
                linux_backend_identity: RuntimeHostHardwareSummary::classify_linux_backend_identity(
                    stream.device.backend_identity,
                ),
                linux_backend_portability:
                    RuntimeHostHardwareSummary::classify_linux_backend_portability(
                        stream.device.backend_identity,
                        stream.simulated,
                        diagnostics.health,
                        diagnostics.device_loss_count,
                        diagnostics.restart_attempt_count,
                        diagnostics.restart_failure_count,
                    ),
                device_id: stream.device.device_id.clone(),
                device_name: stream.device.name.clone(),
                sample_rate: stream.sample_rate.0,
                buffer_size: stream.buffer_size,
                input_channels: stream.input_channels,
                output_channels: stream.output_channels,
                sample_format: stream.sample_format,
                simulated: stream.simulated,
                backend_health: diagnostics.health,
                xrun_count: diagnostics.xrun_count,
                callback_overrun_count: diagnostics.callback_overrun_count,
                device_loss_count: diagnostics.device_loss_count,
                restart_attempt_count: diagnostics.restart_attempt_count,
                restart_failure_count: diagnostics.restart_failure_count,
            },
            audio_pump: RuntimeHostAudioPumpSummary {
                stream_state: RuntimeHostAudioStreamState::Running,
                transfer_policy: RuntimeHostAudioTransferPolicy {
                    max_callback_frames: stream.buffer_size,
                    max_transfer_channels: stream.input_channels.max(stream.output_channels),
                    zero_fill_unwritten_output: true,
                },
                callback_count: 0,
                total_callback_frames: 0,
                total_runtime_output_frames: 0,
                copied_output_samples: 0,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: None,
                last_runtime_graph_id: None,
            },
            clocking: RuntimeHostClockingSummary {
                clock_source: runtime_host_clock_source(stream.clock_source),
                ownership: runtime_host_lifecycle_ownership(stream.lifecycle.ownership),
                restart_policy: runtime_host_restart_policy(stream.lifecycle.restart_policy),
                processing_sample_rate_hz: observation.effective_config.sample_rate.0,
                hardware_sample_rate_hz: stream.sample_rate.0,
                clock_domain: RuntimeHostClockDomain::Aggregate,
                fallback_state: RuntimeHostClockFallbackState::Direct,
                transition_state: RuntimeHostClockTransitionState::Stable,
                drift_state: RuntimeHostClockDriftState::AggregateManaged,
                discontinuity_state: RuntimeHostClockDiscontinuityState::Continuous,
                duplex_mismatch_state: RuntimeHostDuplexMismatchState::Aligned,
                endpoint_topology,
                linux_clocking_parity:
                    signal_runtime::RuntimeHostIoSummary::classify_linux_clocking_parity(
                        RuntimeHostHardwareSummary::classify_linux_backend_identity(
                            stream.device.backend_identity,
                        ),
                        diagnostics.health,
                        RuntimeHostAudioStreamState::Running,
                        RuntimeHostClockDomain::Aggregate,
                        RuntimeHostClockFallbackState::Direct,
                        RuntimeHostClockTransitionState::Stable,
                        RuntimeHostClockDriftState::AggregateManaged,
                        RuntimeHostClockDiscontinuityState::Continuous,
                    ),
                linux_duplex_parity:
                    signal_runtime::RuntimeHostIoSummary::classify_linux_duplex_parity(
                        RuntimeHostHardwareSummary::classify_linux_backend_identity(
                            stream.device.backend_identity,
                        ),
                        diagnostics.health,
                        RuntimeHostAudioStreamState::Running,
                        RuntimeHostClockDomain::Aggregate,
                        RuntimeHostClockFallbackState::Direct,
                        RuntimeHostClockTransitionState::Stable,
                        RuntimeHostDuplexMismatchState::Aligned,
                        endpoint_topology,
                        false,
                    ),
                linux_endpoint_topology_parity:
                    signal_runtime::RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
                        RuntimeHostHardwareSummary::classify_linux_backend_identity(
                            stream.device.backend_identity,
                        ),
                        diagnostics.health,
                        RuntimeHostClockTransitionState::Stable,
                        RuntimeHostClockDiscontinuityState::Continuous,
                        RuntimeHostDuplexMismatchState::Aligned,
                        endpoint_topology,
                        false,
                    ),
                partial_availability: false,
                crossing_required: false,
                callback_interval_ms: samples_to_ms(
                    stream.buffer_size as u32,
                    stream.sample_rate.0,
                ),
            },
            latency: RuntimeHostLatencySummary {
                input_latency_samples,
                output_latency_samples,
                round_trip_latency_samples,
                graph_latency_samples,
                estimated_output_latency_samples,
                estimated_round_trip_latency_samples,
                output_latency_ms: samples_to_ms(output_latency_samples, stream.sample_rate.0),
                graph_latency_ms: samples_to_ms(graph_latency_samples, stream.sample_rate.0),
                estimated_output_latency_ms: samples_to_ms(
                    estimated_output_latency_samples,
                    stream.sample_rate.0,
                ),
                estimated_round_trip_latency_ms: estimated_round_trip_latency_samples
                    .map(|value| samples_to_ms(value, stream.sample_rate.0)),
            },
            runtime_graph_id_matches_pump: false,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LifecycleRunSummary {
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
pub(crate) struct RecoveryHistory {
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
        if request.plugin_format == PluginFormat::Au {
            self.ensure_au_sandbox_session(&request);
        }
        if request.plugin_format == PluginFormat::Lv2 {
            self.ensure_lv2_sandbox_session(&request);
        }
        if request.plugin_format == PluginFormat::Vst3 {
            self.ensure_vst3_sandbox_session(&request);
        }
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
#[path = "host_test_support.rs"]
mod host_test_support;

#[cfg(test)]
mod tests {
    use super::host_test_support::{
        assert_runtime_automation_continuity, assert_runtime_automation_values,
        assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
        prepare_server_host_with_lifecycle, prepare_server_host_without_lifecycle,
        temp_media_fixture_path,
    };
    use super::ServerRuntimeHost;
    use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
    use signal_plugin::{CompletionState, PluginFormat, WatchdogTriggerReason};
    use signal_plugin_clap::ClapSandboxLifecycleHarness;
    use signal_primitives::{ChannelCount, ChannelLayout};
    use signal_runtime::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
        GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
        GraphProjection, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
        PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage,
        PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
        RuntimeConfig, RuntimeConfigRequest, RuntimeErrorKind, RuntimeExternalIoDeviceChangeState,
        RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
        RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
        RuntimeExternalIoPrimaryRole, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
        RuntimeMediaPreviewState, RuntimeObservationApi, RuntimePluginHostPlatform,
        RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeProjectionApi,
        RuntimeReadiness, RuntimeSupervisorApi, SandboxOperationFailureStage, SignalRuntime,
        StopReason, TransportAttachIntent,
    };
    use std::{fs, path::Path};

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
        assert_runtime_plugin_event_snapshot(&supervisor, 2, 2, &[2], 0);
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
    fn server_host_shared_report_surfaces_unavailable_external_io_monitoring_state() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(
            report.observation.external_io_snapshot.health_state,
            RuntimeExternalIoHealthState::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.device_change_state,
            RuntimeExternalIoDeviceChangeState::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.primary_role,
            RuntimeExternalIoPrimaryRole::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.monitoring_state,
            RuntimeExternalIoMonitoringState::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.monitoring_tap_point,
            RuntimeExternalIoMonitoringTapPoint::Unavailable
        );
        assert_eq!(
            report.observation.external_io_snapshot.loopback_state,
            RuntimeExternalIoLoopbackState::Unavailable
        );
        assert_eq!(
            report
                .observation
                .external_io_snapshot
                .linux_clocking_parity,
            signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
        );
        assert_eq!(
            report.observation.external_io_snapshot.linux_duplex_parity,
            signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
        );
        assert_eq!(
            report
                .observation
                .external_io_snapshot
                .linux_endpoint_topology_parity,
            signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
        );
        assert_eq!(
            report.observation.external_io_snapshot.endpoint_topology,
            signal_runtime::RuntimeHostEndpointTopology::Unconfigured
        );
        assert_eq!(
            report.observation.external_io_snapshot.fallback_state,
            signal_runtime::RuntimeHostClockFallbackState::Unconfigured
        );

        let rendered = report.render_json();
        assert!(rendered.contains("\"external_io_snapshot\":{"));
        assert!(rendered.contains("\"health_state\":\"Unavailable\""));
        assert!(rendered.contains("\"monitoring_state\":\"Unavailable\""));
        assert!(rendered.contains("\"loopback_state\":\"Unavailable\""));
        assert!(rendered.contains("\"linux_clocking_parity\":\"Unsupported\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_external_midi_endpoint_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(
            report.observation.external_midi_snapshot.discovery_state,
            signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
        );
        assert_eq!(
            report.observation.external_midi_snapshot.graph_state,
            signal_runtime::RuntimeExternalMidiGraphState::Empty
        );
        assert_eq!(
            report.observation.external_midi_snapshot.provider_name,
            "signal-host-server"
        );
        assert_eq!(report.observation.external_midi_snapshot.device_count, 0);
        assert_eq!(report.observation.external_midi_snapshot.endpoint_count, 0);
        assert_eq!(
            report
                .observation
                .external_midi_snapshot
                .live_ownership
                .ownership_posture,
            signal_runtime::RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
        );
        assert_eq!(
            report
                .observation
                .external_midi_snapshot
                .live_ownership
                .backend_parity,
            signal_runtime::RuntimeExternalMidiBackendParity::Guarded
        );
        assert!(report.observation.external_midi_snapshot.devices.is_empty());
        assert!(report
            .observation
            .external_midi_snapshot
            .endpoints
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"external_midi_snapshot\":{"));
        assert!(rendered.contains("\"live_ownership\":{"));
        assert!(rendered.contains("\"discovery_state\":\"Idle\""));
        assert!(rendered.contains("\"graph_state\":\"Empty\""));
        assert!(rendered.contains("\"backend_parity\":\"Guarded\""));
        assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_linux_backend_session_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        let snapshot = &report.observation.linux_backend_session_snapshot;
        assert_eq!(
            snapshot.backend_identity,
            signal_runtime::RuntimeLinuxAudioBackendIdentity::PipeWire
        );
        assert_eq!(
            snapshot.ownership,
            signal_runtime::RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
        );
        assert_eq!(
            snapshot.lifecycle_state,
            signal_runtime::RuntimeLinuxBackendSessionLifecycleState::Running
        );
        assert_eq!(
            snapshot.device_claim_posture,
            signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
        );
        assert_eq!(
            snapshot.session_role,
            signal_runtime::RuntimeLinuxBackendSessionRole::PrimaryAudioIo
        );
        assert_eq!(
            snapshot.ownership_fallback,
            signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
        );
        assert_eq!(snapshot.backend_name, "pipewire");
        assert_eq!(snapshot.device_id, "pipewire:default-graph");
        assert!(snapshot.simulated);

        let rendered = report.render_json();
        assert!(rendered.contains("\"linux_backend_session_snapshot\":{"));
        assert!(rendered.contains("\"backend_identity\":\"PipeWire\""));
        assert!(rendered.contains("\"ownership\":\"BackendManagedGraph\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_jack_coordination_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        let snapshot = &report.observation.jack_coordination_snapshot;
        assert_eq!(
            snapshot.backend_identity,
            signal_runtime::RuntimeLinuxAudioBackendIdentity::Jack
        );
        assert_eq!(snapshot.backend_name, "jack");
        assert_eq!(
            snapshot.transport_posture,
            signal_runtime::RuntimeJackTransportPosture::Detached
        );
        assert_eq!(
            snapshot.graph_state,
            signal_runtime::RuntimeJackGraphCoordinationState::AttachedGuarded
        );
        assert_eq!(
            snapshot.client_role,
            signal_runtime::RuntimeJackClientRole::PrimaryAudioIo
        );
        assert_eq!(
            snapshot.guarded_state,
            signal_runtime::RuntimeJackGuardedCoordinationState::GraphGuarded
        );
        assert_eq!(snapshot.device_id, "jack:graph-main");
        assert!(snapshot.simulated);

        let rendered = report.render_json();
        assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
        assert!(rendered.contains("\"backend_identity\":\"Jack\""));
        assert!(rendered.contains("\"transport_posture\":\"Detached\""));
        assert!(rendered.contains("\"graph_state\":\"AttachedGuarded\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_control_surface_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(
            report.observation.control_surface_snapshot.discovery_state,
            signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
        );
        assert_eq!(
            report.observation.control_surface_snapshot.graph_state,
            signal_runtime::RuntimeControlSurfaceGraphState::Empty
        );
        assert_eq!(
            report.observation.control_surface_snapshot.provider_name,
            "signal-host-server"
        );
        assert_eq!(report.observation.control_surface_snapshot.device_count, 0);
        assert!(report
            .observation
            .control_surface_snapshot
            .devices
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"control_surface_snapshot\":{"));
        assert!(rendered.contains("\"graph_state\":\"Empty\""));
        assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_advanced_hardware_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(
            report
                .observation
                .advanced_hardware_snapshot
                .discovery_state,
            signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
        );
        assert_eq!(
            report.observation.advanced_hardware_snapshot.graph_state,
            signal_runtime::RuntimeAdvancedHardwareGraphState::Empty
        );
        assert_eq!(
            report.observation.advanced_hardware_snapshot.provider_name,
            "signal-host-server"
        );
        assert_eq!(
            report.observation.advanced_hardware_snapshot.device_count,
            0
        );
        assert!(report
            .observation
            .advanced_hardware_snapshot
            .devices
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"advanced_hardware_snapshot\":{"));
        assert!(rendered.contains("\"graph_state\":\"Empty\""));
        assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_stretch_engine_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(report.observation.stretch_engine_snapshot.clip_count, 0);
        assert_eq!(
            report.observation.stretch_engine_snapshot.ready_clip_count,
            0
        );
        assert!(report.observation.stretch_engine_snapshot.clips.is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"stretch_engine_snapshot\":{"));
        assert!(rendered.contains("\"clip_count\":0"));
        assert!(rendered.contains("\"sample_domain_clip_count\":0"));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_marker_analysis_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(report.observation.marker_analysis_snapshot.clip_count, 0);
        assert_eq!(
            report.observation.marker_analysis_snapshot.ready_clip_count,
            0
        );
        assert_eq!(
            report
                .observation
                .marker_analysis_snapshot
                .tempo_assist_ready_clip_count,
            0
        );
        assert!(report.observation.marker_analysis_snapshot.clips.is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"marker_analysis_snapshot\":{"));
        assert!(rendered.contains("\"clip_count\":0"));
        assert!(rendered.contains("\"tempo_assist_ready_clip_count\":0"));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_transform_artifact_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(report.observation.transform_artifact_snapshot.clip_count, 0);
        assert_eq!(
            report
                .observation
                .transform_artifact_snapshot
                .ready_clip_count,
            0
        );
        assert_eq!(
            report
                .observation
                .transform_artifact_snapshot
                .reusable_clip_count,
            0
        );
        assert!(report
            .observation
            .transform_artifact_snapshot
            .clips
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"transform_artifact_snapshot\":{"));
        assert!(rendered.contains("\"clip_count\":0"));
        assert!(rendered.contains("\"reusable_clip_count\":0"));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_preview_transform_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let host = ServerRuntimeHost::new(runtime);
        let report = host.supervisor_report();

        assert_eq!(report.observation.preview_transform_snapshot.clip_count, 0);
        assert_eq!(
            report
                .observation
                .preview_transform_snapshot
                .active_audition_clip_count,
            0
        );
        assert_eq!(
            report
                .observation
                .preview_transform_snapshot
                .ready_clip_count,
            0
        );
        assert_eq!(
            report
                .observation
                .preview_transform_snapshot
                .artifact_backed_clip_count,
            0
        );
        assert!(report
            .observation
            .preview_transform_snapshot
            .clips
            .is_empty());

        let rendered = report.render_json();
        assert!(rendered.contains("\"preview_transform_snapshot\":{"));
        assert!(rendered.contains("\"clip_count\":0"));
        assert!(rendered.contains("\"artifact_backed_clip_count\":0"));
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_media_service_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        host.runtime
            .handshake(HandshakeRequest {
                client_version: "signal-host-server".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("handshake");
        host.runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .expect("configure");

        let imported_path = temp_media_fixture_path("server-media-service");
        fs::write(&imported_path, b"signal media fixture").expect("write media fixture");
        host.runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:server-media".into(),
                content_hash: "server-media".into(),
                source_path: imported_path.display().to_string(),
                file_name: "server-media.bin".into(),
                byte_size: fs::metadata(&imported_path)
                    .expect("fixture metadata")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 12,
            }])
            .expect("media reconcile");
        host.runtime
            .start_media_preview("asset:sha256:server-media")
            .expect("start media preview");

        let report = host.supervisor_report();
        assert_eq!(report.observation.media_pipeline_snapshot.asset_count, 1);
        assert_eq!(
            report.observation.media_pipeline_snapshot.ready_asset_count,
            1
        );
        assert_eq!(
            report
                .observation
                .media_service_snapshot
                .indexed_asset_count,
            1
        );
        assert_eq!(
            report.observation.media_service_snapshot.preview_state,
            RuntimeMediaPreviewState::Previewing
        );
        assert_eq!(
            report
                .observation
                .media_service_snapshot
                .previewing_asset_id
                .as_deref(),
            Some("asset:sha256:server-media")
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .indexed_asset_count,
            1
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .ready_descriptor_count,
            0
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .loudness_ready_descriptor_count,
            0
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .character_ready_descriptor_count,
            0
        );
        assert_eq!(
            report
                .observation
                .media_library_snapshot
                .unavailable_descriptor_count,
            1
        );

        let rendered = report.render_json();
        assert!(rendered.contains("\"media_pipeline_snapshot\":{"));
        assert!(rendered.contains("\"media_service_snapshot\":{"));
        assert!(rendered.contains("\"media_library_snapshot\":{"));
        assert!(rendered.contains("\"preview_state\":\"Previewing\""));
        assert!(rendered.contains("\"unavailable_descriptor_count\":1"));

        let _ = fs::remove_file(&imported_path);
        if let Some(path) = host
            .runtime
            .get_media_pipeline_snapshot()
            .assets
            .first()
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn server_host_shared_report_surfaces_runtime_spatial_execution_baseline() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);
        host.runtime
            .handshake(HandshakeRequest {
                client_version: "signal-host-server".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("handshake");
        host.runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .expect("configure");
        host.runtime
            .apply_graph_projection(GraphProjection {
                graph_id: "graph:host-server:spatial".into(),
                node_count: 2,
                nodes: vec![
                    GraphNodeProjection {
                        node_id: "spatial-stereo".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::StereoBalance { balance: -0.2 }],
                    },
                    GraphNodeProjection {
                        node_id: "spatial-surround".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 20,
                        stages: vec![GraphStageSpec::StereoBalance { balance: 0.35 }],
                    },
                ],
            })
            .expect("apply spatial graph");
        host.runtime
            .apply_graph_contract_projection(GraphContractProjection {
                graph_id: "graph:host-server:spatial".into(),
                contract_count: 2,
                nodes: vec![
                    GraphNodeContractProjection {
                        node_id: "spatial-stereo".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "main:in".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:spatial:stereo".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            ..GraphNodeBufferContractProjection::default()
                        },
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:stereo".into()),
                            bus_group_id: Some("bus:spatial:stereo".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    GraphNodeContractProjection {
                        node_id: "spatial-surround".into(),
                        buffer_contract: GraphNodeBufferContractProjection {
                            input: GraphNodeBusEndpointProjection {
                                bus_id: "main:surround-in".into(),
                                channels: ChannelLayout::Count(ChannelCount(6)),
                            },
                            output: GraphNodeBusEndpointProjection {
                                bus_id: "bus:spatial:surround".into(),
                                channels: ChannelLayout::Count(ChannelCount(6)),
                            },
                            ..GraphNodeBufferContractProjection::default()
                        },
                        topology: GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:surround".into()),
                            bus_group_id: Some("bus:spatial:surround".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("apply spatial contract");
        host.runtime
            .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
                graph_id: "graph:host-server:spatial".into(),
                bindings: vec![
                    PluginBackedNodeBinding {
                        node_id: "spatial-stereo".into(),
                        sandbox_id: "sandbox:spatial-stereo".into(),
                    },
                    PluginBackedNodeBinding {
                        node_id: "spatial-surround".into(),
                        sandbox_id: "sandbox:spatial-surround".into(),
                    },
                ],
            })
            .expect("bind spatial nodes");

        let report = host.supervisor_report();
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .spatial_node_count,
            2
        );
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .active_spatial_node_count,
            1
        );
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .fallback_spatial_node_count,
            1
        );
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .surround_bed_spatial_node_count,
            1
        );
        assert_eq!(
            report
                .observation
                .execution_topology_summary
                .expanded_fallback_spatial_node_count,
            1
        );
        assert!(report
            .observation
            .plugin_chain_snapshot
            .chains
            .iter()
            .flat_map(|chain| chain.stages.iter())
            .any(|stage| stage.node_id == "spatial-surround"
                && stage
                    .spatial_execution
                    .as_ref()
                    .is_some_and(|spatial| {
                        spatial.fallback_outcome
                            == Some(
                                signal_runtime::RuntimeSpatialFallbackOutcome::BypassSpatialProcessing
                            )
                            && spatial.bed_class
                                == signal_runtime::RuntimeSpatialBedClass::CanonicalSurroundBed
                            && spatial.expanded_fallback_outcome
                                == Some(
                                    signal_runtime::RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial
                                )
                    })));

        let rendered = report.render_json();
        assert!(rendered.contains("\"spatial_node_count\":2"));
        assert!(rendered.contains("\"active_spatial_node_count\":1"));
        assert!(rendered.contains("\"fallback_spatial_node_count\":1"));
        assert!(rendered.contains("\"surround_bed_spatial_node_count\":1"));
        assert!(rendered.contains("\"expanded_fallback_spatial_node_count\":1"));
        assert!(rendered.contains("\"adapter_class\":\"Balance\""));
        assert!(rendered.contains("\"bed_class\":\"CanonicalSurroundBed\""));
        assert!(rendered.contains("\"mix_policy\":\"CollapseToBaselineSpatial\""));
        assert!(rendered.contains("\"execution_mode\":\"Bypassed\""));
    }

    #[test]
    fn server_host_vst3_scan_and_sandbox_surface_linux_runtime_owned_receipts() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);

        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/.vst3".into(), "/usr/lib/vst3".into()],
            formats: vec![PluginFormat::Vst3],
        })
        .expect("server vst3 plugin scan");
        host.ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: "server-vst3-sandbox".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:linux-synth".into()),
        })
        .expect("server vst3 sandbox ensure");

        let report = host.supervisor_report();
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .discovered_type_count,
            4
        );
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .last_scan
                .as_ref()
                .map(|scan| scan.formats.clone()),
            Some(vec![PluginFormat::Vst3])
        );
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:vst3:linux-synth"
                && plugin.format == PluginFormat::Vst3
                && plugin.default_io_layout.midi_inputs == 1));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(
                |plugin| plugin.plugin_type_id == "plugin:vst3:multiout-instrument"
                    && plugin.complex_io_summary.multi_output_instrument
                    && plugin.complex_io_summary.instrument_output_group_count >= 2
            ));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:vst3:bus-fx"
                && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
        let sandbox = report
            .observation
            .plugin_lifecycle_snapshot
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "server-vst3-sandbox")
            .expect("server vst3 sandbox should be exported");
        assert_eq!(sandbox.plugin_format, Some(PluginFormat::Vst3));
        assert_eq!(
            sandbox.plugin_type_id.as_deref(),
            Some("plugin:vst3:linux-synth")
        );
        assert_eq!(
            sandbox.lifecycle_stage,
            Some(PluginSandboxLifecycleStage::TransportAttached)
        );
        assert_eq!(
            sandbox.transport_stage,
            Some(PluginSandboxTransportStage::Attached)
        );
        assert!(sandbox.active);
        assert!(sandbox.active_transport);
        let vst3_parity = report
            .observation
            .plugin_discovery_snapshot
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Vst3)
            .expect("server vst3 parity should be present");
        assert_eq!(
            vst3_parity.linux_parity_band,
            RuntimePluginParityBand::Portable
        );
        assert!(vst3_parity.linux_supported);
        assert_eq!(
            vst3_parity.linux_preferred_sandbox_outcome,
            Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
        );
        assert!(vst3_parity.linux_strict_sandbox_default);
        assert!(vst3_parity.prepare_capable_type_count >= 1);
        assert!(vst3_parity.activate_capable_type_count >= 1);

        let rendered = report.render_json();
        assert!(rendered.contains("\"plugin_format\":\"Vst3\""));
        assert!(rendered.contains("\"formats\":[\"Vst3\"]"));
        assert!(rendered.contains("\"transport_stage\":\"Attached\""));
        assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
        assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    }

    #[test]
    fn server_host_au_scan_and_sandbox_surface_runtime_owned_receipts() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);

        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/Components".into()],
            formats: vec![PluginFormat::Au],
        })
        .expect("server au plugin scan");
        host.ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: "server-au-sandbox".into(),
            plugin_format: PluginFormat::Au,
            plugin_type_id: Some("plugin:au:instrument".into()),
        })
        .expect("server au sandbox ensure");

        let report = host.supervisor_report();
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .discovered_type_count,
            4
        );
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .last_scan
                .as_ref()
                .map(|scan| scan.formats.clone()),
            Some(vec![PluginFormat::Au])
        );
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:au:instrument"
                && plugin.format == PluginFormat::Au
                && plugin.default_io_layout.midi_inputs == 1));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(
                |plugin| plugin.plugin_type_id == "plugin:au:multiout-instrument"
                    && plugin.complex_io_summary.multi_output_instrument
                    && plugin.complex_io_summary.instrument_output_group_count >= 2
            ));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:au:bus-fx"
                && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
        let sandbox = report
            .observation
            .plugin_lifecycle_snapshot
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "server-au-sandbox")
            .expect("server au sandbox should be exported");
        assert_eq!(sandbox.plugin_format, Some(PluginFormat::Au));
        assert_eq!(
            sandbox.plugin_type_id.as_deref(),
            Some("plugin:au:instrument")
        );
        assert_eq!(
            sandbox.lifecycle_stage,
            Some(PluginSandboxLifecycleStage::TransportAttached)
        );
        assert_eq!(
            sandbox.transport_stage,
            Some(PluginSandboxTransportStage::Attached)
        );
        assert!(sandbox.active);
        assert!(sandbox.active_transport);
        let au_parity = report
            .observation
            .plugin_discovery_snapshot
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Au)
            .expect("server au parity should be present");
        assert_eq!(
            au_parity.supported_platforms,
            vec![RuntimePluginHostPlatform::MacOs]
        );
        assert_eq!(
            au_parity.unsupported_platforms,
            vec![
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ]
        );
        assert_eq!(au_parity.discovered_type_count, 4);
        assert_eq!(au_parity.sandbox_count, 1);

        let rendered = report.render_json();
        assert!(rendered.contains("\"plugin_format\":\"Au\""));
        assert!(rendered.contains("\"formats\":[\"Au\"]"));
        assert!(rendered.contains("\"transport_stage\":\"Attached\""));
        assert!(rendered.contains("\"parity_coverage\":["));
        assert!(rendered.contains("\"parity_band\":\"Guarded\""));
        assert!(rendered.contains("\"supported_platforms\":[\"MacOs\"]"));
        assert!(rendered.contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));
    }

    #[test]
    fn server_host_lv2_scan_and_sandbox_surface_linux_runtime_owned_receipts() {
        let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
        let mut host = ServerRuntimeHost::new(runtime);

        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/.lv2".into(), "/usr/lib/lv2".into()],
            formats: vec![PluginFormat::Lv2],
        })
        .expect("server lv2 plugin scan");
        host.ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: "server-lv2-sandbox".into(),
            plugin_format: PluginFormat::Lv2,
            plugin_type_id: Some("plugin:lv2:linux-synth".into()),
        })
        .expect("server lv2 sandbox ensure");

        let report = host.supervisor_report();
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .discovered_type_count,
            4
        );
        assert_eq!(
            report
                .observation
                .plugin_discovery_snapshot
                .last_scan
                .as_ref()
                .map(|scan| scan.formats.clone()),
            Some(vec![PluginFormat::Lv2])
        );
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:lv2:linux-synth"
                && plugin.format == PluginFormat::Lv2
                && plugin.default_io_layout.midi_inputs == 1));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(
                |plugin| plugin.plugin_type_id == "plugin:lv2:multiout-instrument"
                    && plugin.complex_io_summary.multi_output_instrument
                    && plugin.complex_io_summary.instrument_output_group_count >= 2
            ));
        assert!(report
            .observation
            .plugin_discovery_snapshot
            .discovered_types
            .iter()
            .any(|plugin| plugin.plugin_type_id == "plugin:lv2:bus-fx"
                && plugin.complex_io_summary.bus_capable_fx_class.is_some()));
        let sandbox = report
            .observation
            .plugin_lifecycle_snapshot
            .sandboxes
            .iter()
            .find(|sandbox| sandbox.sandbox_id == "server-lv2-sandbox")
            .expect("server lv2 sandbox should be exported");
        assert_eq!(sandbox.plugin_format, Some(PluginFormat::Lv2));
        assert_eq!(
            sandbox.plugin_type_id.as_deref(),
            Some("plugin:lv2:linux-synth")
        );
        assert_eq!(
            sandbox.lifecycle_stage,
            Some(PluginSandboxLifecycleStage::TransportAttached)
        );
        assert_eq!(
            sandbox.transport_stage,
            Some(PluginSandboxTransportStage::Attached)
        );
        assert!(sandbox.active);
        assert!(sandbox.active_transport);
        let lv2_parity = report
            .observation
            .plugin_discovery_snapshot
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Lv2)
            .expect("server lv2 parity should be present");
        assert_eq!(
            lv2_parity.supported_platforms,
            vec![RuntimePluginHostPlatform::Linux]
        );
        assert_eq!(
            lv2_parity.unsupported_platforms,
            vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Windows,
            ]
        );
        assert_eq!(lv2_parity.discovered_type_count, 4);
        assert_eq!(lv2_parity.sandbox_count, 1);
        assert_eq!(
            lv2_parity.linux_parity_band,
            RuntimePluginParityBand::Portable
        );
        assert!(lv2_parity.linux_supported);
        assert_eq!(
            lv2_parity.linux_preferred_sandbox_outcome,
            Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
        );
        assert!(lv2_parity.linux_strict_sandbox_default);
        assert!(lv2_parity.prepare_capable_type_count >= 1);
        assert!(lv2_parity.activate_capable_type_count >= 1);

        let rendered = report.render_json();
        assert!(rendered.contains("\"plugin_format\":\"Lv2\""));
        assert!(rendered.contains("\"formats\":[\"Lv2\"]"));
        assert!(rendered.contains("\"transport_stage\":\"Attached\""));
        assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
        assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
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
