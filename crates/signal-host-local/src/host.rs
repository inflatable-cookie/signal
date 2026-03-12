use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
use signal_hardware::{
    AudioSampleFormat, BackendPolicyTier, HardwareBackend, HardwareClockSource,
    HardwareConfigRequest, HardwareDiagnosticsSnapshot, HardwareLifecycleContract,
    HardwareNegotiationError, HardwareStreamConfig,
};
use signal_hardware_coreaudio::CoreAudioBackend;
use signal_ipc::{
    PluginInstanceStatePayload, PluginMessageEnvelope, PluginMessagePayload, SharedMemoryBroker,
    SharedMemoryTransportPayload,
};
use signal_plugin::{
    BlockPayload, CompletionState, LoopRange, ParameterValueEvent, PluginEvent, PluginFormat,
    PluginRenderContext, PluginSandboxRequest, SandboxPolicy, SandboxWatchdogState,
    WatchdogOutcome, WatchdogTriggerReason,
};
use signal_plugin_clap::{
    classify_sandbox_failure, sandbox_failure_event, BrokeredBlockOutcome, ClapBlockProtocol,
    ClapPluginHostAdapter, ClapSandboxFailureStage, ClapSandboxLifecycleHarness,
};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount};
use signal_runtime::{
    BackendPolicyOverride, BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage,
    CompletionSlotStage, GraphContractProjection, GraphNodeBufferContractProjection,
    GraphNodeBusEndpointProjection, GraphNodeContractProjection, GraphNodeProjection,
    GraphNodeTopologyProjection, GraphProjection, HandshakeRequest, HeartbeatCycleStage,
    LingeringCleanupMode, PluginBackedNodeBinding, PluginBackedNodeBindingProjection,
    PluginFaultKind, PluginNodeRender, PluginNodeRenderBatch, PluginSandboxInstanceFaultRecord,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent, RuntimeConfigRequest,
    RuntimeError, RuntimeEventRecorder, RuntimeExecutionTopologySummary,
    RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
    RuntimeHostClockSource, RuntimeHostClockingSummary, RuntimeHostHardwareSummary,
    RuntimeHostIoSummary, RuntimeHostLatencySummary, RuntimeHostObservationReport,
    RuntimeHostSupervisorReport, RuntimeLifecycleApi, RuntimeObservationApi,
    RuntimeObservationDiagnostics, RuntimeObservationReport, RuntimePluginDispatchState,
    RuntimePreworkServicePressure, RuntimeProjectionApi, RuntimeSupervisorApi,
    RuntimeSupervisorReport, RuntimeWatchdogTrigger, SandboxOperationFailureStage, SignalRuntime,
    StopReason, TransportAttachIntent, WatchdogRestartRecord,
};

const WATCHDOG_TRIGGER_WINDOW_BLOCKS: u64 = 3;
const STEADY_STATE_BLOCKS: u64 = 8;
const SOAK_RESTART_EPISODES: u32 = 3;
const INTER_EPISODE_CONTINUITY_BLOCKS: u64 = 2;
const LOCAL_DEMO_GRAPH_ID: &str = "signal.host.local.demo";
const LOCAL_DEMO_PLUGIN_NODE_ID: &str = "plugin-insert";
const LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES: u32 = 24;
const LOCAL_DEMO_PLUGIN_TAIL_SAMPLES: u32 = 48;
#[derive(Clone, Debug, Default)]
struct LocalSupervisorState {
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
    DeviceLoss,
    DeviceLossRestartFailure,
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
pub struct LocalPayloadSummary {
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
pub struct LocalPluginDispatchSummary {
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub render_context: PluginRenderContext,
    pub automation_value: Option<f32>,
    pub render_bypass_count: u32,
    pub last_render_bypassed: bool,
    pub last_render_latency_samples: u32,
    pub last_render_tail_samples: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalExecutionSummary {
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
pub struct LocalTransportSummary {
    pub sandbox_id: String,
    pub shared_memory_lease_id: String,
    pub shared_memory_region_id: String,
    pub shared_memory_path: String,
    pub shared_memory_bytes: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalFaultSummary {
    pub deadline_misses: u32,
    pub heartbeat_misses: u32,
    pub watchdog_triggered: bool,
    pub watchdog_trigger_reason: Option<WatchdogTriggerReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalAudioStreamState {
    Stopped,
    Running,
    Faulted,
}

impl From<LocalAudioStreamState> for RuntimeHostAudioStreamState {
    fn from(value: LocalAudioStreamState) -> Self {
        match value {
            LocalAudioStreamState::Stopped => RuntimeHostAudioStreamState::Stopped,
            LocalAudioStreamState::Running => RuntimeHostAudioStreamState::Running,
            LocalAudioStreamState::Faulted => RuntimeHostAudioStreamState::Faulted,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalAudioTransferPolicy {
    pub max_callback_frames: usize,
    pub max_transfer_channels: u16,
    pub zero_fill_unwritten_output: bool,
}

impl From<LocalAudioTransferPolicy> for RuntimeHostAudioTransferPolicy {
    fn from(value: LocalAudioTransferPolicy) -> Self {
        Self {
            max_callback_frames: value.max_callback_frames,
            max_transfer_channels: value.max_transfer_channels,
            zero_fill_unwritten_output: value.zero_fill_unwritten_output,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalAudioPumpSummary {
    pub stream_state: LocalAudioStreamState,
    pub transfer_policy: LocalAudioTransferPolicy,
    pub callback_count: u64,
    pub last_callback_index: Option<u64>,
    pub total_callback_frames: u64,
    pub total_runtime_output_frames: u64,
    pub copied_output_samples: u64,
    pub zero_filled_output_samples: u64,
    pub dropped_output_samples: u64,
    pub last_callback_output_peak: Option<f32>,
    pub last_runtime_graph_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalHardwareSummary {
    pub device_id: String,
    pub device_name: String,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub output_channels: u16,
    pub sample_format: AudioSampleFormat,
    pub lifecycle: HardwareLifecycleContract,
    pub simulated: bool,
    pub backend_diagnostics: HardwareDiagnosticsSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalRuntimeHostSummary {
    pub backend_name: &'static str,
    pub hardware: LocalHardwareSummary,
    pub audio_pump: LocalAudioPumpSummary,
    pub scan_roots: Vec<String>,
    pub execution: LocalExecutionSummary,
    pub transport: LocalTransportSummary,
    pub topology: RuntimeExecutionTopologySummary,
    pub plugin_dispatch: Option<LocalPluginDispatchSummary>,
    pub last_payload: LocalPayloadSummary,
    pub faults: LocalFaultSummary,
}

#[derive(Clone, Debug)]
struct LocalAudioPumpState {
    summary: LocalAudioPumpSummary,
}

impl Default for LocalAudioPumpState {
    fn default() -> Self {
        Self {
            summary: LocalAudioPumpSummary {
                stream_state: LocalAudioStreamState::Stopped,
                transfer_policy: LocalAudioTransferPolicy {
                    max_callback_frames: 0,
                    max_transfer_channels: 0,
                    zero_fill_unwritten_output: true,
                },
                callback_count: 0,
                last_callback_index: None,
                total_callback_frames: 0,
                total_runtime_output_frames: 0,
                copied_output_samples: 0,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: None,
                last_runtime_graph_id: None,
            },
        }
    }
}

impl LocalAudioPumpState {
    fn reset_for_stream(&mut self, stream: &HardwareStreamConfig) {
        let transfer_policy = LocalAudioTransferPolicy {
            max_callback_frames: stream.buffer_size,
            max_transfer_channels: stream.output_channels,
            zero_fill_unwritten_output: true,
        };
        self.summary = LocalAudioPumpSummary {
            stream_state: LocalAudioStreamState::Stopped,
            transfer_policy,
            ..Self::default().summary
        };
    }

    fn stop(&mut self) {
        self.summary.stream_state = LocalAudioStreamState::Stopped;
    }

    fn fault(&mut self) {
        self.summary.stream_state = LocalAudioStreamState::Faulted;
        self.summary.last_runtime_graph_id = None;
    }

    fn summary(&self) -> LocalAudioPumpSummary {
        self.summary.clone()
    }

    fn record_callback(
        &mut self,
        stream: &HardwareStreamConfig,
        callback_index: u64,
        runtime_output: &AudioBuffer,
        runtime_graph_id: Option<&str>,
    ) {
        self.summary.stream_state = LocalAudioStreamState::Running;
        self.summary.last_callback_index = Some(callback_index);
        self.summary.callback_count = self.summary.callback_count.saturating_add(1);
        self.summary.total_callback_frames = self
            .summary
            .total_callback_frames
            .saturating_add(stream.buffer_size as u64);
        self.summary.total_runtime_output_frames = self
            .summary
            .total_runtime_output_frames
            .saturating_add(runtime_output.frames().0 as u64);
        if let Some(graph_id) = runtime_graph_id {
            self.summary.last_runtime_graph_id = Some(graph_id.to_string());
        }

        let transfer = transfer_runtime_output_to_host_buffer(
            runtime_output,
            stream,
            self.summary.transfer_policy,
        );
        self.summary.copied_output_samples = self
            .summary
            .copied_output_samples
            .saturating_add(transfer.outcome.copied_samples as u64);
        self.summary.zero_filled_output_samples = self
            .summary
            .zero_filled_output_samples
            .saturating_add(transfer.outcome.zero_filled_samples as u64);
        self.summary.dropped_output_samples = self
            .summary
            .dropped_output_samples
            .saturating_add(transfer.outcome.dropped_samples as u64);
        self.summary.last_callback_output_peak = Some(transfer.output_peak);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LocalAudioTransferOutcome {
    copied_samples: usize,
    zero_filled_samples: usize,
    dropped_samples: usize,
}

struct LocalAudioTransferResult {
    outcome: LocalAudioTransferOutcome,
    output_peak: f32,
}

fn transfer_runtime_output_to_host_buffer(
    runtime_output: &AudioBuffer,
    stream: &HardwareStreamConfig,
    policy: LocalAudioTransferPolicy,
) -> LocalAudioTransferResult {
    let callback_frames = stream.buffer_size.min(policy.max_callback_frames);
    let host_channels = usize::from(stream.output_channels.min(policy.max_transfer_channels));
    let runtime_channels = runtime_output.channel_count().0;
    let copied_frames = callback_frames.min(runtime_output.frames().0);
    let copied_channels = host_channels.min(runtime_channels);
    let mut host_buffer = vec![0.0_f32; callback_frames.saturating_mul(host_channels)];
    let runtime_samples = runtime_output.samples();

    for frame_index in 0..copied_frames {
        for channel_index in 0..copied_channels {
            let runtime_index = frame_index
                .saturating_mul(runtime_channels)
                .saturating_add(channel_index);
            let host_index = frame_index
                .saturating_mul(host_channels)
                .saturating_add(channel_index);
            host_buffer[host_index] = runtime_samples[runtime_index];
        }
    }

    let copied_samples = copied_frames.saturating_mul(copied_channels);
    let callback_samples = callback_frames.saturating_mul(host_channels);
    let dropped_frame_samples = runtime_output
        .frames()
        .0
        .saturating_sub(copied_frames)
        .saturating_mul(runtime_channels);
    let dropped_channel_samples =
        copied_frames.saturating_mul(runtime_channels.saturating_sub(copied_channels));
    let output_peak = host_buffer
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()));

    LocalAudioTransferResult {
        outcome: LocalAudioTransferOutcome {
            copied_samples,
            zero_filled_samples: callback_samples.saturating_sub(copied_samples),
            dropped_samples: dropped_frame_samples.saturating_add(dropped_channel_samples),
        },
        output_peak,
    }
}

pub struct LocalRuntimeHost {
    runtime: SignalRuntime,
    coreaudio: CoreAudioBackend,
    clap: ClapPluginHostAdapter,
    broker: SharedMemoryBroker,
    active_output_stream: Option<HardwareStreamConfig>,
    audio_pump: LocalAudioPumpState,
    supervisor: LocalSupervisorState,
    events: RuntimeEventRecorder,
}

impl LocalRuntimeHost {
    pub fn new(runtime: SignalRuntime) -> Self {
        let events = RuntimeEventRecorder::default();
        let mut runtime = runtime;
        runtime.subscribe(Box::new(events.clone()));

        Self {
            runtime,
            coreaudio: CoreAudioBackend::default(),
            clap: ClapPluginHostAdapter::default(),
            broker: SharedMemoryBroker::default(),
            active_output_stream: None,
            audio_pump: LocalAudioPumpState::default(),
            supervisor: LocalSupervisorState::default(),
            events,
        }
    }

    fn prepare_default_output_hardware(&mut self) -> Result<HardwareStreamConfig, RuntimeError> {
        let stream = self
            .coreaudio
            .default_output_stream(
                self.runtime.config().sample_rate.0,
                self.runtime.config().graph.block_size,
            )
            .map_err(Self::runtime_error_from_hardware_negotiation)?;
        let hardware_request =
            HardwareConfigRequest::from_stream(&stream, self.coreaudio.policy_tier());
        self.runtime.apply_hardware_config(hardware_request)?;
        self.runtime
            .set_active_output_device(stream.device.device_id.clone());
        self.set_backend_policy(BackendPolicyOverride {
            tier: hardware_request.backend_policy,
        })?;
        self.runtime
            .set_backend_policy_tier(hardware_request.backend_policy);
        self.audio_pump.reset_for_stream(&stream);
        self.active_output_stream = Some(stream.clone());
        Ok(stream)
    }

    fn runtime_error_from_hardware_negotiation(error: HardwareNegotiationError) -> RuntimeError {
        RuntimeError::new(
            signal_runtime::RuntimeErrorKind::InvalidRequest,
            format!("hardware negotiation failed: {}", error.message),
        )
    }

    pub fn boot_default(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(None)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_timeout_recovery(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Timeout))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_crash_recovery(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::Crash))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_heartbeat_miss_recovery(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::HeartbeatMiss))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_teardown_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryTeardownFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_then_cleanup(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownThenCleanup))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_deferred_teardown_cleanup_retry(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryDeferredTeardownCleanupRetry))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_restart_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryRestartFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_overlap_contention(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryOverlapContention))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_recovery_interleaved_failures(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::RecoveryInterleavedFailures))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_escalating_heartbeat_failures(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: 2,
        }))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_watchdog_soak(&mut self) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: SOAK_RESTART_EPISODES,
        }))
    }

    pub fn boot_with_device_loss_recovery(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::DeviceLoss))
    }

    pub fn boot_with_device_loss_restart_failure(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::DeviceLossRestartFailure))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn boot_with_mixed_watchdog_soak(
        &mut self,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        self.boot_with_fault_recovery(Some(FaultInjection::MixedWatchdogEpisodes {
            restart_episodes: SOAK_RESTART_EPISODES,
        }))
    }

    fn boot_with_fault_recovery(
        &mut self,
        fault: Option<FaultInjection>,
    ) -> Result<LocalRuntimeHostSummary, RuntimeError> {
        let runtime_config = RuntimeConfigRequest::new(
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size,
        );
        self.runtime.handshake(HandshakeRequest {
            client_version: "signal-host-local".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })?;
        self.runtime.configure(runtime_config)?;
        let assembly = local_demo_runtime_assembly();
        self.runtime
            .apply_graph_projection(assembly.graph.clone())?;
        self.runtime
            .apply_graph_contract_projection(assembly.graph_contracts.clone())?;

        let hardware_stream = self.prepare_default_output_hardware()?;

        self.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        })?;

        for sandbox in &assembly.plugin_sandboxes {
            self.ensure_plugin_sandbox(sandbox.spec())?;
        }
        self.runtime
            .apply_plugin_backed_node_bindings(assembly.plugin_bindings())?;
        self.runtime
            .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());
        let sandbox = assembly.primary_sandbox();

        self.runtime.set_cpu_load_percent(4.5);
        self.runtime.set_graph_latency_ms(2.7);
        self.runtime.start()?;

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:default",
            "instance:local:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
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
        let mut executed_steady_state_tail = false;
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
                                    sandbox.request.sandbox_id.as_str(),
                                    "instance:local:default",
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
                        "instance:local:default",
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
                                "instance:local:default",
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
                FaultInjection::DeviceLoss => {
                    self.execute_block_sequence(&protocol, &mut run, 2, &mut lifecycle, false)?;
                    self.handle_device_loss_transition(false)?;
                    self.execute_block_sequence(
                        &protocol,
                        &mut run,
                        STEADY_STATE_BLOCKS.saturating_sub(2),
                        &mut lifecycle,
                        false,
                    )?;
                    executed_steady_state_tail = true;
                }
                FaultInjection::DeviceLossRestartFailure => {
                    self.execute_block_sequence(&protocol, &mut run, 2, &mut lifecycle, false)?;
                    self.handle_device_loss_transition(true)?;
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
                                    "instance:local:default",
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
                                    "instance:local:default",
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
                                    "instance:local:default",
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
                                    "instance:local:default",
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
                                    "instance:local:default",
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
                                    "instance:local:default",
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
                                    "instance:local:default",
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
                                    "instance:local:default",
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
                                    "instance:local:default",
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
            if !executed_steady_state_tail {
                self.execute_block_sequence(
                    &protocol,
                    &mut run,
                    STEADY_STATE_BLOCKS,
                    &mut lifecycle,
                    false,
                )?;
            }
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
        let observation = self.observation_report();
        Ok(LocalRuntimeHostSummary {
            backend_name: self.coreaudio.backend_name(),
            hardware: LocalHardwareSummary {
                device_id: hardware_stream.device.device_id.clone(),
                device_name: hardware_stream.device.name.clone(),
                sample_rate: hardware_stream.sample_rate.0,
                buffer_size: hardware_stream.buffer_size,
                output_channels: hardware_stream.output_channels,
                sample_format: hardware_stream.sample_format,
                lifecycle: hardware_stream.lifecycle,
                simulated: hardware_stream.simulated,
                backend_diagnostics: self.coreaudio.diagnostics(),
            },
            audio_pump: self.audio_pump.summary(),
            scan_roots: self.supervisor.last_scan_roots.clone(),
            execution: LocalExecutionSummary {
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
            transport: LocalTransportSummary {
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
            topology: observation.execution_topology_summary.clone(),
            plugin_dispatch: run
                .last_plugin_render_context
                .clone()
                .map(|render_context| LocalPluginDispatchSummary {
                    processing_epoch: run.processing_epoch,
                    block_sequence: run.last_block_sequence,
                    render_context,
                    automation_value: run.last_plugin_automation_value,
                    render_bypass_count: run.plugin_render_bypass_count,
                    last_render_bypassed: run.last_plugin_render_bypassed,
                    last_render_latency_samples: run.last_plugin_render_latency_samples,
                    last_render_tail_samples: run.last_plugin_render_tail_samples,
                }),
            last_payload: LocalPayloadSummary {
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
            faults: LocalFaultSummary {
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
            last_plugin_render_context: None,
            last_plugin_automation_value: None,
            plugin_render_bypass_count: 0,
            last_plugin_render_bypassed: false,
            last_plugin_render_latency_samples: 0,
            last_plugin_render_tail_samples: 0,
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
        let plugin_dispatch_state = self
            .runtime
            .prepare_plugin_dispatch_state_for_block(run.processing_epoch, block_sequence)?;
        let (dispatch, payload) = self.build_plugin_block_request(
            protocol,
            run.processing_epoch,
            block_sequence,
            frame_count,
            &plugin_dispatch_state,
        )?;
        run.last_plugin_render_context = Some(dispatch.render_context.clone());
        run.last_plugin_automation_value =
            payload_automation_value(&payload, protocol.automation_parameter_id());
        self.runtime.record_block_dispatch(
            run.sandbox_id.as_str(),
            run.shared_memory_lease_id.as_str(),
            run.processing_epoch,
            block_sequence,
            frame_count,
            BlockDispatchStage::Requested,
            None,
        );
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
        self.apply_plugin_node_render_for_block(run, block_sequence, &stored_result)?;
        let engine_result =
            self.process_engine_block_through_output_pump(run.processing_epoch, block_sequence)?;
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

    fn build_plugin_block_request(
        &self,
        protocol: &ClapBlockProtocol,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        plugin_dispatch_state: &RuntimePluginDispatchState,
    ) -> Result<(signal_plugin::BlockDispatch, BlockPayload), RuntimeError> {
        let dispatch = protocol.block_dispatch(
            processing_epoch,
            block_sequence,
            frame_count,
            self.plugin_render_context(frame_count, plugin_dispatch_state),
        );
        let payload = self.plugin_input_payload(
            protocol,
            block_sequence,
            frame_count,
            plugin_dispatch_state.parameter_batch.as_ref(),
        )?;
        Ok((dispatch, payload))
    }

    fn plugin_render_context(
        &self,
        frame_count: u32,
        plugin_dispatch_state: &RuntimePluginDispatchState,
    ) -> PluginRenderContext {
        let transport = plugin_dispatch_state.transport;
        PluginRenderContext {
            sample_rate_hz: self.runtime.config().sample_rate.0,
            tempo_bpm: transport
                .map(|transport| transport.tempo_bpm)
                .unwrap_or(120.0),
            timeline_position_samples: transport
                .map(|transport| transport.timeline_position_samples)
                .unwrap_or(0),
            playing: transport
                .map(|transport| transport.playing)
                .unwrap_or(false),
            bypassed: false,
            loop_range: transport
                .and_then(|transport| transport.loop_state)
                .map(|loop_state| LoopRange {
                    start_samples: loop_state.start_samples,
                    end_samples: loop_state.end_samples,
                }),
            deadline_frames: frame_count,
        }
    }

    fn plugin_input_payload(
        &self,
        protocol: &ClapBlockProtocol,
        block_sequence: u64,
        frame_count: u32,
        parameter_batch: Option<&signal_runtime::ParameterBatch>,
    ) -> Result<BlockPayload, RuntimeError> {
        let mut payload = protocol.test_input_payload(block_sequence, frame_count);
        let automation_parameter_id = protocol.automation_parameter_id();
        let automation_value =
            plugin_automation_value_from_runtime_batch(automation_parameter_id, parameter_batch);
        for event in &mut payload.events.events {
            if let (PluginEvent::ParameterValue(existing), Some(automation_value)) =
                (event, automation_value)
            {
                if existing.parameter_id == automation_parameter_id {
                    *existing = automation_value;
                }
            }
        }

        let expected_audio_samples =
            payload.audio.channel_count as usize * payload.audio.frame_count as usize;
        if payload.audio.samples.len() != expected_audio_samples {
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::PluginFailure,
                "plugin input payload audio shape became invalid",
            ));
        }
        Ok(payload)
    }

    fn apply_plugin_node_render_for_block(
        &mut self,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        stored_result: &BrokeredBlockOutcome,
    ) -> Result<(), RuntimeError> {
        let render_bypassed = stored_result.result.slot.state != CompletionState::Completed;
        if render_bypassed {
            run.plugin_render_bypass_count = run.plugin_render_bypass_count.saturating_add(1);
        }
        run.last_plugin_render_bypassed = render_bypassed;
        run.last_plugin_render_latency_samples = LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES;
        run.last_plugin_render_tail_samples = LOCAL_DEMO_PLUGIN_TAIL_SAMPLES;
        let channel_layout = match stored_result.output.audio.channel_count {
            1 => ChannelLayout::Mono,
            2 => ChannelLayout::Stereo,
            count => ChannelLayout::Count(ChannelCount(count as usize)),
        };
        let output = AudioBuffer::from_interleaved(
            self.runtime.config().sample_rate,
            channel_layout,
            stored_result.output.audio.samples.clone(),
        );
        self.runtime
            .apply_plugin_node_render_batch(PluginNodeRenderBatch {
                graph_id: LOCAL_DEMO_GRAPH_ID.into(),
                processing_epoch: run.processing_epoch,
                block_sequence,
                renders: vec![PluginNodeRender {
                    node_id: LOCAL_DEMO_PLUGIN_NODE_ID.into(),
                    sandbox_id: run.sandbox_id.clone(),
                    output,
                    latency_samples: LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES,
                    tail_samples: LOCAL_DEMO_PLUGIN_TAIL_SAMPLES,
                    bypassed: render_bypassed,
                }],
            })
    }

    fn process_engine_block_through_output_pump(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Result<signal_runtime::RuntimeEngineBlockResult, RuntimeError> {
        let Some(stream) = self.active_output_stream.clone() else {
            self.audio_pump.fault();
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "local host audio pump has no negotiated output stream",
            ));
        };
        let input = AudioBuffer::new(
            self.runtime.config().sample_rate,
            ChannelLayout::Count(ChannelCount(stream.output_channels as usize)),
            FrameCount(stream.buffer_size),
        );
        let result = self
            .runtime
            .process_engine_block(processing_epoch, block_sequence, input)
            .inspect_err(|_| self.audio_pump.fault())?;
        self.audio_pump.record_callback(
            &stream,
            block_sequence,
            &result.output,
            result.snapshot.graph_id.as_deref(),
        );
        Ok(result)
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
        if let Some(transport) = replacement_run.transport.as_ref() {
            self.runtime.promote_transport_session_to_steady_state(
                sandbox_id,
                replacement_run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
            );
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
        if let Some(transport) = restarted_run.transport.as_ref() {
            self.runtime.promote_transport_session_to_steady_state(
                sandbox_id,
                restarted_run.shared_memory_lease_id.as_str(),
                transport.region_id.as_str(),
            );
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

    fn stop_runtime_with_reason(&mut self, reason: StopReason) -> Result<(), RuntimeError> {
        if self.runtime.get_control_snapshot().running {
            self.audio_pump.stop();
            self.supervisor.last_stop_reason = Some(reason);
            self.runtime.stop(reason)
        } else {
            Ok(())
        }
    }

    fn stop_runtime_for_recovery(&mut self) -> Result<(), RuntimeError> {
        self.stop_runtime_with_reason(StopReason::DegradedModeRecovery)
    }

    fn handle_device_loss_transition(
        &mut self,
        restart_should_fail: bool,
    ) -> Result<(), RuntimeError> {
        self.coreaudio
            .simulate_device_loss("simulated CoreAudio device disconnect");
        self.stop_runtime_with_reason(StopReason::DeviceReconfigure)?;
        self.audio_pump.fault();
        self.coreaudio
            .simulate_restart_attempt("simulated CoreAudio device restart attempt");

        if restart_should_fail {
            self.coreaudio
                .simulate_restart_failure("simulated CoreAudio device restart failure");
            return Err(RuntimeError::new(
                signal_runtime::RuntimeErrorKind::HardwareFailure,
                "simulated device-loss restart failure",
            ));
        }

        self.prepare_default_output_hardware()?;
        self.runtime.start()?;
        self.coreaudio.mark_recovered();
        Ok(())
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

    pub fn clap_supported(&self) -> bool {
        self.clap.supports_format(PluginFormat::Clap)
    }

    #[allow(dead_code)]
    pub fn observation_diagnostics(&self) -> RuntimeObservationDiagnostics {
        self.events.diagnostics()
    }

    #[allow(dead_code)]
    pub fn observation_report(&self) -> RuntimeObservationReport {
        RuntimeObservationReport::capture(&self.runtime, &self.events)
    }

    pub fn host_observation_report(&self) -> RuntimeHostObservationReport {
        let observation = self.observation_report();
        let host_io = self.host_io_summary(&observation);
        RuntimeHostObservationReport::new(observation, host_io)
    }

    pub fn supervisor_report(&self) -> RuntimeSupervisorReport {
        RuntimeSupervisorReport::capture(&self.runtime, &self.events)
    }

    pub fn host_supervisor_report(&self) -> RuntimeHostSupervisorReport {
        let supervisor = self.supervisor_report();
        let host_io = self.host_io_summary(&supervisor.observation);
        RuntimeHostSupervisorReport::new(supervisor, host_io)
    }

    fn host_io_summary(&self, observation: &RuntimeObservationReport) -> RuntimeHostIoSummary {
        let audio_pump = self.audio_pump.summary();
        let backend_diagnostics = self.coreaudio.diagnostics();
        let active_stream = self.active_output_stream.as_ref();
        let sample_rate = active_stream
            .map(|stream| stream.sample_rate.0)
            .unwrap_or(self.runtime.config().sample_rate.0);
        let buffer_size = active_stream
            .map(|stream| stream.buffer_size)
            .unwrap_or(self.runtime.config().graph.block_size);
        let graph_latency_samples = observation.engine_block_snapshot.total_latency_samples;
        let output_latency_samples = active_stream
            .map(|stream| stream.latency.output_latency_samples)
            .unwrap_or(buffer_size as u32);
        let input_latency_samples =
            active_stream.and_then(|stream| stream.latency.input_latency_samples);
        let round_trip_latency_samples =
            active_stream.and_then(|stream| stream.latency.round_trip_latency_samples);
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
        let callback_interval_ms = samples_to_ms(buffer_size as u32, sample_rate);
        RuntimeHostIoSummary {
            hardware: RuntimeHostHardwareSummary {
                backend_name: self.coreaudio.backend_name().into(),
                device_id: active_stream
                    .as_ref()
                    .map(|stream| stream.device.device_id.clone())
                    .unwrap_or_else(|| "coreaudio:unconfigured".into()),
                device_name: active_stream
                    .as_ref()
                    .map(|stream| stream.device.name.clone())
                    .unwrap_or_else(|| "Unconfigured Device".into()),
                sample_rate,
                buffer_size,
                output_channels: active_stream
                    .as_ref()
                    .map(|stream| stream.output_channels)
                    .unwrap_or_default(),
                sample_format: active_stream
                    .as_ref()
                    .map(|stream| stream.sample_format)
                    .unwrap_or(AudioSampleFormat::F32),
                simulated: active_stream
                    .as_ref()
                    .map(|stream| stream.simulated)
                    .unwrap_or(false),
                backend_health: backend_diagnostics.health,
                xrun_count: backend_diagnostics.xrun_count,
                callback_overrun_count: backend_diagnostics.callback_overrun_count,
                device_loss_count: backend_diagnostics.device_loss_count,
                restart_attempt_count: backend_diagnostics.restart_attempt_count,
                restart_failure_count: backend_diagnostics.restart_failure_count,
            },
            audio_pump: RuntimeHostAudioPumpSummary {
                stream_state: audio_pump.stream_state.into(),
                transfer_policy: audio_pump.transfer_policy.into(),
                callback_count: audio_pump.callback_count,
                total_callback_frames: audio_pump.total_callback_frames,
                total_runtime_output_frames: audio_pump.total_runtime_output_frames,
                copied_output_samples: audio_pump.copied_output_samples,
                zero_filled_output_samples: audio_pump.zero_filled_output_samples,
                dropped_output_samples: audio_pump.dropped_output_samples,
                last_callback_output_peak: audio_pump.last_callback_output_peak,
                last_runtime_graph_id: audio_pump.last_runtime_graph_id.clone(),
            },
            clocking: RuntimeHostClockingSummary {
                clock_source: active_stream
                    .map(|stream| RuntimeHostClockSource::from(stream.clock_source))
                    .unwrap_or(RuntimeHostClockSource::from(HardwareClockSource::Internal)),
                ownership: active_stream
                    .map(|stream| stream.lifecycle.ownership.into())
                    .unwrap_or(HardwareLifecycleContract::default().ownership.into()),
                restart_policy: active_stream
                    .map(|stream| stream.lifecycle.restart_policy.into())
                    .unwrap_or(HardwareLifecycleContract::default().restart_policy.into()),
                callback_interval_ms,
            },
            latency: RuntimeHostLatencySummary {
                input_latency_samples,
                output_latency_samples,
                round_trip_latency_samples,
                graph_latency_samples,
                estimated_output_latency_samples,
                estimated_round_trip_latency_samples,
                output_latency_ms: samples_to_ms(output_latency_samples, sample_rate),
                graph_latency_ms: samples_to_ms(graph_latency_samples, sample_rate),
                estimated_output_latency_ms: samples_to_ms(
                    estimated_output_latency_samples,
                    sample_rate,
                ),
                estimated_round_trip_latency_ms: estimated_round_trip_latency_samples
                    .map(|samples| samples_to_ms(samples, sample_rate)),
            },
            runtime_graph_id_matches_pump: audio_pump.last_runtime_graph_id.as_deref()
                == observation.engine_block_snapshot.graph_id.as_deref(),
        }
    }
}

fn samples_to_ms(samples: u32, sample_rate: u32) -> f32 {
    if sample_rate == 0 {
        return 0.0;
    }
    samples as f32 / sample_rate as f32 * 1000.0
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
    last_plugin_render_context: Option<PluginRenderContext>,
    last_plugin_automation_value: Option<f32>,
    plugin_render_bypass_count: u32,
    last_plugin_render_bypassed: bool,
    last_plugin_render_latency_samples: u32,
    last_plugin_render_tail_samples: u32,
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
    plugin_render_bypass_count: u32,
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
            plugin_render_bypass_count: self.plugin_render_bypass_count,
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
        self.plugin_render_bypass_count = self
            .plugin_render_bypass_count
            .saturating_add(history.plugin_render_bypass_count);
        self.watchdog_triggered |= history.watchdog_triggered;
        self.current_watchdog_triggered = false;
        if self.watchdog_trigger_reason.is_none() {
            self.watchdog_trigger_reason = history.watchdog_trigger_reason;
        }
    }
}

fn local_demo_graph_projection() -> GraphProjection {
    GraphProjection {
        graph_id: LOCAL_DEMO_GRAPH_ID.into(),
        node_count: 4,
        nodes: vec![
            GraphNodeProjection {
                node_id: "track-input".into(),
                execution_class: GraphNodeExecutionClass::LatencyBearing,
                latency_samples: 24,
                stages: vec![
                    GraphStageSpec::Gain { linear: 0.75 },
                    GraphStageSpec::Bias { amount: 0.05 },
                    GraphStageSpec::TanhDrive { drive: 1.35 },
                ],
            },
            GraphNodeProjection {
                node_id: "plugin-insert".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 0,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.82 }],
            },
            GraphNodeProjection {
                node_id: "bus-main".into(),
                execution_class: GraphNodeExecutionClass::Stateful,
                latency_samples: 0,
                stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
            },
            GraphNodeProjection {
                node_id: "output-main".into(),
                execution_class: GraphNodeExecutionClass::Stateful,
                latency_samples: 0,
                stages: vec![
                    GraphStageSpec::StereoBalance { balance: -0.2 },
                    GraphStageSpec::HardClip { threshold: 0.8 },
                ],
            },
        ],
    }
}

fn local_demo_graph_contract_projection(graph_id: &str) -> GraphContractProjection {
    GraphContractProjection {
        graph_id: graph_id.into(),
        contract_count: 4,
        nodes: vec![
            GraphNodeContractProjection {
                node_id: "track-input".into(),
                buffer_contract: GraphNodeBufferContractProjection {
                    input: GraphNodeBusEndpointProjection {
                        bus_id: "main:in".into(),
                        channels: ChannelLayout::Stereo,
                    },
                    output: GraphNodeBusEndpointProjection {
                        bus_id: "bus:track:lead".into(),
                        channels: ChannelLayout::Stereo,
                    },
                    ..GraphNodeBufferContractProjection::default()
                },
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                },
            },
            GraphNodeContractProjection {
                node_id: "plugin-insert".into(),
                buffer_contract: GraphNodeBufferContractProjection {
                    input: GraphNodeBusEndpointProjection {
                        bus_id: "bus:track:lead".into(),
                        channels: ChannelLayout::Stereo,
                    },
                    output: GraphNodeBusEndpointProjection {
                        bus_id: "bus:mix:tracks".into(),
                        channels: ChannelLayout::Stereo,
                    },
                    ..GraphNodeBufferContractProjection::default()
                },
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                },
            },
            GraphNodeContractProjection {
                node_id: "bus-main".into(),
                buffer_contract: GraphNodeBufferContractProjection {
                    input: GraphNodeBusEndpointProjection {
                        bus_id: "bus:mix:tracks".into(),
                        channels: ChannelLayout::Stereo,
                    },
                    output: GraphNodeBusEndpointProjection {
                        bus_id: "bus:console:main".into(),
                        channels: ChannelLayout::Stereo,
                    },
                    ..GraphNodeBufferContractProjection::default()
                },
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::Bus),
                    lane_id: None,
                    bus_group_id: Some("mix:master".into()),
                },
            },
            GraphNodeContractProjection {
                node_id: "output-main".into(),
                buffer_contract: GraphNodeBufferContractProjection {
                    input: GraphNodeBusEndpointProjection {
                        bus_id: "bus:console:main".into(),
                        channels: ChannelLayout::Stereo,
                    },
                    output: GraphNodeBusEndpointProjection {
                        bus_id: "main:out".into(),
                        channels: ChannelLayout::Stereo,
                    },
                    ..GraphNodeBufferContractProjection::default()
                },
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::ConsoleNode),
                    lane_id: None,
                    bus_group_id: Some("console:main".into()),
                },
            },
        ],
    }
}

#[derive(Clone, Debug)]
struct LocalDemoPluginSandboxAssembly {
    request: PluginSandboxRequest,
    plugin_format: &'static str,
    bound_node_ids: Vec<&'static str>,
}

impl LocalDemoPluginSandboxAssembly {
    fn spec(&self) -> PluginSandboxSpec {
        PluginSandboxSpec {
            sandbox_id: self.request.sandbox_id.clone(),
            plugin_format: self.plugin_format,
        }
    }
}

#[derive(Clone, Debug)]
struct LocalDemoRuntimeAssembly {
    graph: GraphProjection,
    graph_contracts: GraphContractProjection,
    plugin_sandboxes: Vec<LocalDemoPluginSandboxAssembly>,
}

impl LocalDemoRuntimeAssembly {
    fn primary_sandbox(&self) -> &LocalDemoPluginSandboxAssembly {
        self.plugin_sandboxes
            .first()
            .expect("local demo assembly should define a primary sandbox")
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

fn local_demo_runtime_assembly() -> LocalDemoRuntimeAssembly {
    let graph = local_demo_graph_projection();
    LocalDemoRuntimeAssembly {
        graph_contracts: local_demo_graph_contract_projection(&graph.graph_id),
        graph,
        plugin_sandboxes: vec![LocalDemoPluginSandboxAssembly {
            request: PluginSandboxRequest::new(
                "local-default-sandbox",
                PluginFormat::Clap,
                SandboxPolicy::Strict,
            ),
            plugin_format: "clap",
            bound_node_ids: vec![LOCAL_DEMO_PLUGIN_NODE_ID],
        }],
    }
}

fn plugin_automation_value_from_runtime_batch(
    automation_parameter_id: u32,
    parameter_batch: Option<&signal_runtime::ParameterBatch>,
) -> Option<ParameterValueEvent> {
    let parameter_batch = parameter_batch?;
    let value = parameter_batch.events.last()?.normalized_value;
    Some(ParameterValueEvent {
        offset_frames: 0,
        parameter_id: automation_parameter_id,
        normalized_value: value,
    })
}

fn payload_automation_value(payload: &BlockPayload, automation_parameter_id: u32) -> Option<f32> {
    payload.events.events.iter().find_map(|event| match event {
        PluginEvent::ParameterValue(event) if event.parameter_id == automation_parameter_id => {
            Some(event.normalized_value)
        }
        _ => None,
    })
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
        | FaultInjection::DeviceLoss
        | FaultInjection::DeviceLossRestartFailure
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

impl RuntimeSupervisorApi for LocalRuntimeHost {
    fn start_plugin_scan(
        &mut self,
        request: PluginScanRequest,
    ) -> Result<signal_runtime::ScanHandle, RuntimeError> {
        self.supervisor.scans_started = self.supervisor.scans_started.saturating_add(1);
        self.supervisor.last_scan_roots = request.roots;
        Ok(signal_runtime::ScanHandle(self.supervisor.scans_started))
    }

    fn ensure_plugin_sandbox(
        &mut self,
        request: PluginSandboxSpec,
    ) -> Result<signal_runtime::SandboxHandle, RuntimeError> {
        self.supervisor.sandboxes = self.supervisor.sandboxes.saturating_add(1);
        self.runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );
        self.supervisor.last_sandbox_id = Some(request.sandbox_id);
        Ok(signal_runtime::SandboxHandle(self.supervisor.sandboxes))
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
    use super::{
        local_demo_runtime_assembly, LifecycleRunSummary, LocalAudioStreamState,
        LocalAudioTransferPolicy, LocalRuntimeHost, LocalRuntimeHostSummary, LOCAL_DEMO_GRAPH_ID,
        LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES, LOCAL_DEMO_PLUGIN_NODE_ID,
        LOCAL_DEMO_PLUGIN_TAIL_SAMPLES,
    };
    use signal_graph::GraphNodeTopologyRole;
    use signal_hardware::{
        AudioDeviceDescriptor, AudioSampleFormat, AudioStreamDirection, BackendHealth,
        HardwareClockSource, HardwareLatencyProfile, HardwareLifecycleContract,
        HardwareLifecycleOwnership, HardwareRestartPolicy, HardwareStreamConfig,
    };
    use signal_plugin::{CompletionState, LoopRange, PluginEvent, WatchdogTriggerReason};
    use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
    use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, SampleRate};
    use signal_runtime::RuntimeHostClockSource;
    use signal_runtime::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode, PluginSandboxLifecycleStage,
        PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent, RuntimeConfig,
        RuntimeConfigRequest, RuntimeErrorKind, RuntimeHostAudioStreamState, RuntimeLifecycleApi,
        RuntimeObservationApi, RuntimeProjectionApi, RuntimeReadiness, RuntimeSupervisorApi,
        RuntimeSupervisorReport, SandboxOperationFailureStage, SignalRuntime, StopReason,
        TransportAttachIntent,
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
        assert_eq!(timeline.segment_count(), epochs.len(), "{timeline:?}");
        assert_eq!(timeline.segment_epochs(), epochs, "{timeline:?}");
        assert_eq!(
            timeline.first_block_sequence(),
            Some(first_block_sequence),
            "{timeline:?}"
        );
        assert_eq!(
            timeline.last_block_sequence(),
            Some(last_block_sequence),
            "{timeline:?}"
        );
        assert_eq!(timeline.sequence_gaps, sequence_gaps, "{timeline:?}");
        assert_eq!(timeline.lease_rollovers, lease_rollovers, "{timeline:?}");
    }

    fn assert_plugin_dispatch_summary(
        summary: &LocalRuntimeHostSummary,
        supervisor: &RuntimeSupervisorReport,
        expected_bypass_count: u32,
    ) {
        let dispatch = summary
            .plugin_dispatch
            .as_ref()
            .expect("plugin dispatch summary");
        let expected_timeline = ((dispatch.block_sequence as i64) * 512).rem_euclid(16 * 512);
        let expected_automation = ((dispatch.block_sequence % 8) as f32) / 7.0;

        assert_eq!(
            dispatch.processing_epoch,
            summary.execution.processing_epoch
        );
        assert_eq!(
            dispatch.block_sequence,
            summary.execution.last_block_sequence
        );
        assert_eq!(dispatch.render_context.sample_rate_hz, 48_000);
        assert_eq!(dispatch.render_context.tempo_bpm, 126.0);
        assert_eq!(
            dispatch.render_context.timeline_position_samples,
            expected_timeline
        );
        assert!(dispatch.render_context.playing);
        assert!(!dispatch.render_context.bypassed);
        assert_eq!(
            dispatch.render_context.loop_range,
            Some(LoopRange {
                start_samples: 0,
                end_samples: 16 * 512,
            })
        );
        assert_eq!(dispatch.render_context.deadline_frames, 512);
        assert!(dispatch
            .automation_value
            .is_some_and(|value| (value - expected_automation).abs() < 1.0e-6));
        assert_eq!(dispatch.render_bypass_count, expected_bypass_count);
        assert!(!dispatch.last_render_bypassed);
        assert_eq!(
            dispatch.last_render_latency_samples,
            LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES
        );
        assert_eq!(
            dispatch.last_render_tail_samples,
            LOCAL_DEMO_PLUGIN_TAIL_SAMPLES
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .planned_nodes
            .iter()
            .any(|node| node.node_id == LOCAL_DEMO_PLUGIN_NODE_ID
                && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")));
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            Some(dispatch.render_context.playing)
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(dispatch.render_context.timeline_position_samples)
        );
    }

    fn assert_local_plugin_topology(summary: &LocalRuntimeHostSummary) {
        let topology = &summary.topology;
        assert_eq!(topology.node_count, 4);
        assert_eq!(topology.track_lane_node_count, 2);
        assert_eq!(topology.bus_node_count, 1);
        assert_eq!(topology.console_node_count, 1);
        assert_eq!(topology.track_lane_group_count, 1);
        assert_eq!(topology.bus_group_count, 1);
        assert_eq!(topology.console_group_count, 1);
        assert!(topology.nodes.iter().any(|node| {
            node.node_id == "track-input"
                && node.lane_id.as_deref() == Some("track:lead")
                && node.bus_group_id.as_deref() == Some("mix:tracks")
                && node.input_bus_id == "main:in"
                && node.output_bus_id == "bus:track:lead"
        }));
        assert!(topology.nodes.iter().any(|node| {
            node.node_id == LOCAL_DEMO_PLUGIN_NODE_ID
                && node.topology_role == GraphNodeTopologyRole::TrackLane
                && node.lane_id.as_deref() == Some("track:lead")
                && node.bus_group_id.as_deref() == Some("mix:tracks")
                && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")
                && node.input_bus_id == "bus:track:lead"
                && node.output_bus_id == "bus:mix:tracks"
        }));
        assert!(topology.nodes.iter().any(|node| {
            node.node_id == "bus-main"
                && node.topology_role == GraphNodeTopologyRole::Bus
                && node.bus_group_id.as_deref() == Some("mix:master")
                && node.input_bus_id == "bus:mix:tracks"
                && node.output_bus_id == "bus:console:main"
        }));
        assert!(topology.nodes.iter().any(|node| {
            node.node_id == "output-main"
                && node.topology_role == GraphNodeTopologyRole::ConsoleNode
                && node.bus_group_id.as_deref() == Some("console:main")
                && node.input_bus_id == "bus:console:main"
                && node.output_bus_id == "main:out"
        }));
    }

    fn prepare_local_host_with_lifecycle() -> (
        LocalRuntimeHost,
        ClapBlockProtocol,
        ClapSandboxLifecycleHarness,
        LifecycleRunSummary,
    ) {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        let runtime_config = RuntimeConfigRequest::new(
            host.runtime.config().sample_rate.0,
            host.runtime.config().graph.block_size,
        );
        host.runtime
            .handshake(HandshakeRequest {
                client_version: "signal-host-local".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(192_000),
            })
            .expect("handshake");
        host.runtime.configure(runtime_config).expect("configure");
        let assembly = local_demo_runtime_assembly();
        host.runtime
            .apply_graph_projection(assembly.graph.clone())
            .expect("graph projection");
        host.runtime
            .apply_graph_contract_projection(assembly.graph_contracts.clone())
            .expect("graph contract projection");

        host.prepare_default_output_hardware()
            .expect("hardware config");
        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
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
        host.runtime.set_cpu_load_percent(4.5);
        host.runtime.set_graph_latency_ms(2.7);
        host.runtime.start().expect("start runtime");

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:default",
            "instance:local:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
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

    fn prepare_local_host_without_lifecycle() -> (LocalRuntimeHost, ClapBlockProtocol) {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        let runtime_config = RuntimeConfigRequest::new(
            host.runtime.config().sample_rate.0,
            host.runtime.config().graph.block_size,
        );
        host.runtime
            .handshake(HandshakeRequest {
                client_version: "signal-host-local".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(192_000),
            })
            .expect("handshake");
        host.runtime.configure(runtime_config).expect("configure");
        let assembly = local_demo_runtime_assembly();
        host.runtime
            .apply_graph_projection(assembly.graph.clone())
            .expect("graph projection");
        host.runtime
            .apply_graph_contract_projection(assembly.graph_contracts.clone())
            .expect("graph contract projection");

        host.prepare_default_output_hardware()
            .expect("hardware config");
        host.start_plugin_scan(PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
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
        host.runtime.set_cpu_load_percent(4.5);
        host.runtime.set_graph_latency_ms(2.7);
        host.runtime.start().expect("start runtime");

        let protocol = ClapBlockProtocol::new(
            "plugin:clap:default",
            "instance:local:default",
            signal_plugin::PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            2048,
        );
        (host, protocol)
    }

    #[test]
    fn local_host_builds_plugin_block_request_from_runtime_transport_and_parameter_truth() {
        let (mut host, protocol, _lifecycle, run) = prepare_local_host_with_lifecycle();
        let frame_count = host.runtime.config().graph.block_size as u32;
        let plugin_dispatch_state = host
            .runtime
            .prepare_plugin_dispatch_state_for_block(run.processing_epoch, 7)
            .expect("prepare plugin dispatch state");
        let (dispatch, payload) = host
            .build_plugin_block_request(
                &protocol,
                run.processing_epoch,
                7,
                frame_count,
                &plugin_dispatch_state,
            )
            .expect("build plugin block request");

        assert_eq!(dispatch.render_context.sample_rate_hz, 48_000);
        assert_eq!(dispatch.render_context.tempo_bpm, 126.0);
        assert_eq!(dispatch.render_context.timeline_position_samples, 7 * 512);
        assert!(dispatch.render_context.playing);
        assert_eq!(
            dispatch.render_context.loop_range,
            Some(LoopRange {
                start_samples: 0,
                end_samples: 16 * 512,
            })
        );
        let automation_value = payload
            .events
            .events
            .iter()
            .find_map(|event| match event {
                PluginEvent::ParameterValue(event)
                    if event.parameter_id == protocol.automation_parameter_id() =>
                {
                    Some(event.normalized_value)
                }
                _ => None,
            })
            .expect("automation value event");
        assert!((automation_value - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn local_host_routes_sandbox_plugin_audio_through_bound_engine_node() {
        let (mut host, protocol, mut lifecycle, mut run) = prepare_local_host_with_lifecycle();

        let outcome = host
            .execute_block(&protocol, &mut run, 1, &mut lifecycle, false)
            .expect("execute realtime block");
        let snapshot = host.runtime.get_engine_block_snapshot();

        assert_eq!(outcome.output.audio.first_sample(), Some(1.0));
        assert_eq!(
            run.last_engine_graph_id.as_deref(),
            Some(LOCAL_DEMO_GRAPH_ID)
        );
        assert_eq!(snapshot.graph_id.as_deref(), Some(LOCAL_DEMO_GRAPH_ID));
        assert_eq!(snapshot.output_tail_samples, LOCAL_DEMO_PLUGIN_TAIL_SAMPLES);
        assert_eq!(snapshot.last_first_output_sample, Some(0.8));
        assert!(run.last_engine_output_peak.unwrap_or_default() >= 0.79);
    }

    #[test]
    fn local_host_timeout_block_bypasses_plugin_node_without_detaching_graph_binding() {
        let (mut host, protocol, mut lifecycle, mut run) = prepare_local_host_with_lifecycle();

        let outcome = host
            .execute_block(&protocol, &mut run, 1, &mut lifecycle, true)
            .expect("execute timeout block");
        let snapshot = host.runtime.get_engine_block_snapshot();

        assert_eq!(outcome.result.slot.state, CompletionState::TimedOut);
        assert_eq!(run.last_completion_state, CompletionState::TimedOut);
        assert_eq!(
            run.last_engine_graph_id.as_deref(),
            Some(LOCAL_DEMO_GRAPH_ID)
        );
        assert!(snapshot.planned_nodes.iter().any(|node| {
            node.node_id == "plugin-insert"
                && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")
        }));
        assert_eq!(
            run.last_plugin_render_context
                .as_ref()
                .map(|context| context.tempo_bpm),
            Some(126.0)
        );
        assert_eq!(
            run.last_plugin_render_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(512)
        );
        assert_eq!(run.last_plugin_automation_value, Some(1.0 / 7.0));
        assert_eq!(run.plugin_render_bypass_count, 1);
        assert!(run.last_plugin_render_bypassed);
        assert_eq!(
            run.last_plugin_render_latency_samples,
            LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES
        );
        assert_eq!(
            run.last_plugin_render_tail_samples,
            LOCAL_DEMO_PLUGIN_TAIL_SAMPLES
        );
        assert!(run.last_engine_output_peak.unwrap_or_default() > 0.05);
        assert!(run.last_engine_output_peak.unwrap_or_default() < 0.1);
    }

    #[test]
    fn local_host_rolls_leases_forward_after_timeout() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
        assert_eq!(
            summary.execution.last_block_sequence,
            supervisor
                .observation
                .timeline_snapshot
                .block_sequence_continuity
                .last_block_sequence()
                .expect("last block sequence")
        );
        assert_eq!(
            summary.execution.last_engine_graph_id.as_deref(),
            Some("signal.host.local.demo")
        );
        assert!(
            summary
                .execution
                .last_engine_output_peak
                .unwrap_or_default()
                <= 0.8
        );
        assert!(summary.execution.last_engine_output_rms.is_some());
        assert!(summary.audio_pump.last_callback_output_peak.is_some());
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            Some(2)
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
        assert_eq!(supervisor.observation.engine_block_snapshot.node_count, 4);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .stateful_node_count,
            4
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
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_planning_enabled
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .inline_realtime_node_count,
            0
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .stateful_realtime_node_count,
            3
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_eligible_node_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_semantic_policy,
            signal_runtime::RuntimePreworkServiceSemanticPolicy::PluginConstrained
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
        assert!(
            !supervisor
                .observation
                .engine_block_snapshot
                .prework_service_plugin_gate_active
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .planned_nodes
            .iter()
            .any(|node| node.node_id == "plugin-insert"
                && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")));
        assert_eq!(supervisor.observation.engine_block_snapshot.phase_count, 2);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_phase_count,
            1
        );
        assert_eq!(supervisor.observation.engine_block_snapshot.lane_count, 2);
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .anticipative_lane_count,
            1
        );
        assert_eq!(
            supervisor.observation.engine_block_snapshot.dispatch_count,
            2
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .dispatch_boundary_count,
            1
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prepared_dispatch_count,
            1
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
            1
        );
        assert!(
            supervisor
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
            signal_runtime::RuntimePreworkForecastMode::RuntimeRoleDefault
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_forecast_profile,
            Some(signal_runtime::RuntimePreworkForecastProfile::Local)
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_forecast_profile_source,
            Some(signal_runtime::RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_forecast_policy_target_window_blocks,
            Some(2)
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_pressure,
            signal_runtime::RuntimePreworkServicePressure::Elevated
        );
        assert!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_yield_count
                >= 1
        );
        assert!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_service_throttle_count
                >= 1
        );
        assert!(matches!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_state,
            signal_runtime::RuntimePreworkCacheState::Consumed
                | signal_runtime::RuntimePreworkCacheState::Admitted
        ));
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_freshness_state,
            signal_runtime::RuntimePreworkFreshnessState::Fresh
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_queue_capacity,
            3
        );
        assert!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_queue_depth
                > 0
        );
        assert!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_queue_depth
                <= 3
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_peak_queue_depth,
            3
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_window_target_count,
            3
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_window_target_block_sequences,
            vec![
                summary.execution.last_block_sequence,
                summary.execution.last_block_sequence + 1,
                summary.execution.last_block_sequence + 2,
            ]
        );
        let engine_snapshot = &supervisor.observation.engine_block_snapshot;
        assert!(
            engine_snapshot.prework_cache_admissions >= engine_snapshot.prework_cache_consumptions
        );
        assert!(
            engine_snapshot.prework_cache_queued_admissions
                >= engine_snapshot.prework_cache_window_target_count as u64
        );
        assert!(
            engine_snapshot.prework_cache_queued_consumptions
                <= engine_snapshot.prework_cache_consumptions
        );
        assert_eq!(
            engine_snapshot.prework_cache_retirement_count,
            engine_snapshot.prework_cache_unconsumed_retirement_count
                + engine_snapshot.prework_cache_consumed_retirement_count
        );
        assert!(engine_snapshot.prework_cache_retirement_count > 0);
        assert_eq!(
            engine_snapshot.prework_cache_hits + engine_snapshot.prework_cache_misses,
            engine_snapshot.prework_cache_consumptions
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .last_prework_output_peak
            .is_some());
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_output_peak,
            supervisor
                .observation
                .engine_block_snapshot
                .last_realtime_input_peak
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_admission_processing_epoch,
            Some(2)
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .last_prework_admission_block_sequence
            .is_some_and(|sequence| sequence >= summary.execution.last_block_sequence));
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .last_prework_admitted_from_block_sequence
            .is_some_and(|sequence| sequence <= summary.execution.last_block_sequence));
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_consumption_processing_epoch,
            Some(2)
        );
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_consumption_block_sequence,
            Some(summary.execution.last_block_sequence)
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .last_prework_consumed_from_block_sequence
            .is_some_and(|sequence| sequence <= summary.execution.last_block_sequence));
        assert!(
            matches!(
                supervisor
                    .observation
                    .engine_block_snapshot
                    .last_prework_retirement_reason,
                Some(signal_runtime::RuntimePreworkRetirementReason::PlanningWindowRevised)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::TransportStarted)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::TransportStopped)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::TransportSeeked)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::TransportTempoChanged)
                    | Some(
                        signal_runtime::RuntimePreworkRetirementReason::TransportLoopStateChanged
                    )
                    | Some(signal_runtime::RuntimePreworkRetirementReason::TransportLoopWrapped)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::ParameterBatchApplied)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::InputSignatureChanged)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::ProcessingEpochExpired)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::BlockSequenceExpired)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::SupersededByAdmission)
                    | Some(signal_runtime::RuntimePreworkRetirementReason::QueueCapacityExceeded)
            ),
            "unexpected prework retirement reason: {:?}",
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_retirement_reason
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .last_prework_retired_unconsumed
            .is_some());
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .prework_cache_valid_until_processing_epoch,
            Some(3)
        );
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_valid_until_block_sequence
            .is_some_and(|sequence| sequence >= summary.execution.last_block_sequence));
        assert!(supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_remaining_valid_blocks
            .is_some_and(|remaining| remaining > 0));
        assert_eq!(
            supervisor
                .observation
                .engine_block_snapshot
                .total_latency_samples,
            24
        );
        assert_eq!(summary.last_payload.event_count, 11);
        assert_eq!(summary.last_payload.parameter_event_count, 2);
        assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
        assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
        assert_eq!(summary.last_payload.note_event_count, 1);
        assert_eq!(summary.last_payload.note_expression_event_count, 3);
        assert_eq!(summary.last_payload.midi_event_count, 1);
        assert_eq!(summary.last_payload.generated_event_bytes, 268);
        assert_eq!(
            summary.last_payload.first_output_sample,
            Some(summary.execution.last_block_sequence as f32)
        );
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
        assert!(summary
            .transport
            .shared_memory_region_id
            .starts_with("region-"));
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
            0
        );
        assert_eq!(
            supervisor
                .observation
                .transport_concurrency_snapshot
                .last_admitted_sandbox_id
                .as_deref(),
            Some("local-default-sandbox")
        );
        let automation = &supervisor.observation.automation_snapshot;
        assert_eq!(automation.parameter_id, 4096);
        assert_eq!(automation.value_events, 8);
        assert_eq!(automation.modulation_events, 8);
        assert_eq!(automation.gesture_begin_events, 2);
        assert_eq!(automation.gesture_end_events, 6);
        assert!(automation.first_value.is_some());
        assert!(automation.last_value.is_some(), "{automation:?}");
        assert!(automation.last_modulation.is_some());
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        let timeline = &supervisor
            .observation
            .timeline_snapshot
            .block_sequence_continuity;
        assert!(timeline.segment_count() >= 2);
        assert!(timeline.first_block_sequence().is_some());
        assert!(timeline
            .last_block_sequence()
            .is_some_and(|last| last >= summary.execution.last_block_sequence));
        assert!(timeline.sequence_gaps <= 1, "{timeline:?}");
        assert_eq!(timeline.lease_rollovers, 1);
        assert_local_plugin_topology(&summary);
        assert_plugin_dispatch_summary(&summary, &supervisor, 2);
    }

    #[test]
    fn local_host_rolls_back_replacement_transport_when_recovery_teardown_fails() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
    fn local_host_exposes_lingering_detach_fault_state_after_deferred_recovery_teardown_failure() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
    fn local_host_recovers_after_lingering_deferred_teardown_cleanup() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
    fn local_host_recovers_after_lingering_cleanup_fails_once_more() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
    fn local_host_sweeps_orphan_lingering_sessions_before_overlap_recovery() {
        let (mut host, protocol, mut lifecycle, run) = prepare_local_host_with_lifecycle();
        let orphan_region = host
            .broker
            .create_region("local-orphan-lingering", 256)
            .expect("orphan region");
        let orphan_transport = orphan_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "local-default-sandbox",
                "lease-orphan",
                orphan_transport.region_id.as_str(),
                TransportAttachIntent::RecoveryOverlap,
                Some(orphan_transport.backing_path.clone()),
                Some(orphan_transport.total_bytes),
            )
            .expect("orphan transport session");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            "lease-orphan",
            orphan_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("replacement rollback linger".into()),
        );

        let recovered = host
            .recover_sandbox(
                &protocol,
                "local-default-sandbox",
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
    fn local_host_aborts_when_orphan_lingering_cleanup_fails_before_overlap_recovery() {
        let (mut host, protocol, mut lifecycle, run) = prepare_local_host_with_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "local-default-sandbox",
                "lease-orphan",
                "region-orphan-failure",
                TransportAttachIntent::RecoveryOverlap,
                None,
                None,
            )
            .expect("orphan transport session");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            "lease-orphan",
            "region-orphan-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("replacement rollback linger".into()),
        );

        let error = host
            .recover_sandbox(
                &protocol,
                "local-default-sandbox",
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
    fn local_host_cleans_multiple_orphan_lingering_sessions_for_same_sandbox() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        let orphan_region_a = host
            .broker
            .create_region("local-orphan-a", 256)
            .expect("orphan region a");
        let orphan_transport_a = orphan_region_a.metadata().clone();
        let orphan_region_b = host
            .broker
            .create_region("local-orphan-b", 256)
            .expect("orphan region b");
        let orphan_transport_b = orphan_region_b.metadata().clone();

        host.runtime
            .begin_transport_session_with_metadata(
                "local-default-sandbox",
                "lease-orphan-a",
                orphan_transport_a.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(orphan_transport_a.backing_path.clone()),
                Some(orphan_transport_a.total_bytes),
            )
            .expect("orphan session a");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            "lease-orphan-a",
            orphan_transport_a.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("orphan a lingering".into()),
        );
        host.runtime
            .begin_transport_session_with_metadata(
                "local-default-sandbox",
                "lease-orphan-b",
                orphan_transport_b.region_id.as_str(),
                TransportAttachIntent::RecoveryOverlap,
                Some(orphan_transport_b.backing_path.clone()),
                Some(orphan_transport_b.total_bytes),
            )
            .expect("orphan session b");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            "lease-orphan-b",
            orphan_transport_b.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("orphan b lingering".into()),
        );

        host.cleanup_orphan_lingering_sessions_for_sandbox(
            "local-default-sandbox",
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
    fn local_host_reconciles_late_lingering_completion_without_disturbing_active_replacement() {
        let (mut host, protocol) = prepare_local_host_without_lifecycle();
        let late_region = host
            .broker
            .create_region("local-late-lingering", 256)
            .expect("late lingering region");
        let late_transport = late_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "local-default-sandbox",
                "lease-late-origin",
                late_transport.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(late_transport.backing_path.clone()),
                Some(late_transport.total_bytes),
            )
            .expect("late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            "lease-late-origin",
            late_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late origin teardown completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered = host
            .run_lifecycle(&protocol, "local-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        host.reconcile_late_lingering_sessions_after_start("local-default-sandbox", &recovered);

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
    fn local_host_keeps_active_replacement_running_when_late_lingering_cleanup_fails() {
        let (mut host, protocol) = prepare_local_host_without_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "local-default-sandbox",
                "lease-late-origin",
                "region-late-origin-failure",
                TransportAttachIntent::SteadyState,
                None,
                None,
            )
            .expect("late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            "lease-late-origin",
            "region-late-origin-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("late origin teardown completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered = host
            .run_lifecycle(&protocol, "local-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        host.reconcile_late_lingering_sessions_after_start("local-default-sandbox", &recovered);

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
    fn local_host_sweeps_prior_late_lingering_before_next_overlap_recovery() {
        let (mut host, protocol) = prepare_local_host_without_lifecycle();
        let late_region = host
            .broker
            .create_region("local-adjacent-lingering", 256)
            .expect("late lingering region");
        let late_transport = late_region.metadata().clone();
        host.runtime
            .begin_transport_session_with_metadata(
                "local-default-sandbox",
                "lease-prior-lingering",
                late_transport.region_id.as_str(),
                TransportAttachIntent::SteadyState,
                Some(late_transport.backing_path.clone()),
                Some(late_transport.total_bytes),
            )
            .expect("prior late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            "lease-prior-lingering",
            late_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("prior late completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered_epoch2 = host
            .run_lifecycle(&protocol, "local-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");
        let recovered_transport = recovered_epoch2
            .transport
            .as_ref()
            .expect("recovered transport");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            recovered_epoch2.shared_memory_lease_id.as_str(),
            recovered_transport.region_id.as_str(),
            PluginSandboxTransportStage::DetachFault,
            Some(recovered_epoch2.processing_epoch),
            Some("current replacement became lingering before adjacent recovery".into()),
        );

        let recovered_epoch3 = host
            .recover_sandbox(
                &protocol,
                "local-default-sandbox",
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
    fn local_host_aborts_adjacent_overlap_recovery_when_prior_late_lingering_lacks_metadata() {
        let (mut host, protocol) = prepare_local_host_without_lifecycle();
        host.runtime
            .begin_transport_session_with_metadata(
                "local-default-sandbox",
                "lease-prior-lingering",
                "region-prior-lingering-failure",
                TransportAttachIntent::SteadyState,
                None,
                None,
            )
            .expect("prior late lingering session");
        host.runtime.record_plugin_sandbox_transport(
            "local-default-sandbox",
            "lease-prior-lingering",
            "region-prior-lingering-failure",
            PluginSandboxTransportStage::DetachFault,
            Some(1),
            Some("prior late completion".into()),
        );
        let mut lifecycle = ClapSandboxLifecycleHarness::default();
        let recovered_epoch2 = host
            .run_lifecycle(&protocol, "local-default-sandbox", 2, &mut lifecycle)
            .expect("replacement lifecycle");

        let error = host
            .recover_sandbox(
                &protocol,
                "local-default-sandbox",
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
    fn local_host_overlap_recovery_keeps_bound_plugin_dispatch_truth() {
        let (mut host, protocol, mut lifecycle, mut run) = prepare_local_host_with_lifecycle();

        host.execute_block(&protocol, &mut run, 1, &mut lifecycle, false)
            .expect("initial realtime block");
        let mut recovered = host
            .recover_sandbox(
                &protocol,
                "local-default-sandbox",
                &mut lifecycle,
                &run,
                RecoveryRestartIntent::WatchdogRecovery,
                None,
            )
            .expect("overlap recovery");
        let block_sequence = host.runtime.allocate_block_sequence();
        host.execute_block(
            &protocol,
            &mut recovered,
            block_sequence,
            &mut lifecycle,
            false,
        )
        .expect("replacement realtime block");

        let snapshot = host.runtime.get_engine_block_snapshot();
        let concurrency = host.runtime.get_transport_concurrency_snapshot();

        assert_eq!(recovered.processing_epoch, 2);
        assert_eq!(
            recovered
                .last_plugin_render_context
                .as_ref()
                .map(|context| context.tempo_bpm),
            Some(126.0)
        );
        assert_eq!(
            recovered
                .last_plugin_render_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(((block_sequence as i64) * 512).rem_euclid(16 * 512))
        );
        assert_eq!(
            recovered.last_plugin_automation_value,
            Some(((block_sequence % 8) as f32) / 7.0)
        );
        assert_eq!(recovered.plugin_render_bypass_count, 0);
        assert!(!recovered.last_plugin_render_bypassed);
        assert_eq!(
            recovered.last_engine_graph_id.as_deref(),
            Some(LOCAL_DEMO_GRAPH_ID)
        );
        assert!(snapshot.planned_nodes.iter().any(|node| {
            node.node_id == LOCAL_DEMO_PLUGIN_NODE_ID
                && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")
        }));
        assert_eq!(
            snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            Some(2)
        );
        assert_eq!(
            snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            Some(((block_sequence as i64) * 512).rem_euclid(16 * 512))
        );
        assert_eq!(concurrency.current_attached_sessions, 1);
        assert_eq!(concurrency.current_recovery_overlap_sessions, 0);
        assert_eq!(concurrency.peak_attached_sessions, 2);
    }

    #[test]
    fn local_host_rolls_back_replacement_transport_when_recovery_start_fails() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
    fn local_host_rolls_back_partial_overlap_when_competing_recovery_attach_is_rejected() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
            Some("local-default-sandbox")
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
    fn local_host_handles_interleaved_recovery_failures_across_retries() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
            Some("local-default-sandbox")
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
    fn local_host_recovers_after_crash() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
            .shared_memory_path
            .ends_with(".signal-shm"));
        assert_runtime_automation_values(&supervisor, 9, 9, 3, 6, 0.0, 0.0, 0.08);
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 8, 0, 1);
    }

    #[test]
    fn local_host_recovers_after_heartbeat_watchdog_trigger() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
        assert_runtime_automation_values(&supervisor, 8, 8, 2, 6, 2.0 / 7.0, 1.0 / 7.0, 0.10);
        assert_runtime_automation_continuity(&supervisor, 2, 2, &[2], 0);
        assert_runtime_sequence_continuity(&supervisor, &[2], 2, 9, 0, 0);
        assert_local_plugin_topology(&summary);
        assert_plugin_dispatch_summary(&summary, &supervisor, 0);
    }

    #[test]
    fn local_host_enters_safe_mode_after_repeated_watchdog_restarts() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
        assert_eq!(
            summary.execution.last_block_sequence, 11,
            "unexpected escalating heartbeat summary: {summary:?}"
        );
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
        assert_runtime_automation_values(&supervisor, 10, 10, 2, 8, 2.0 / 7.0, 3.0 / 7.0, 0.14);
        assert_runtime_automation_continuity(&supervisor, 2, 3, &[2, 3], 1);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3], 2, 11, 0, 1);
        assert_plugin_dispatch_summary(&summary, &supervisor, 0);
    }

    #[test]
    fn local_host_soak_path_rolls_across_multiple_lease_generations() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
        assert_eq!(
            summary.execution.last_block_sequence, 13,
            "unexpected watchdog soak summary: {summary:?}"
        );
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
        assert_eq!(summary.last_payload.first_output_sample, Some(13.0));
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
        assert_eq!(supervisor.transport_fault_event_count(), 15);
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
        assert_runtime_automation_values(&supervisor, 12, 12, 3, 9, 2.0 / 7.0, 5.0 / 7.0, 0.18);
        assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 13, 0, 2);
        assert_plugin_dispatch_summary(&summary, &supervisor, 0);
    }

    #[test]
    fn local_host_boot_summary_exposes_negotiated_hardware_contract() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        let summary = host.boot_default().expect("default local host boot");
        let supervisor = host.supervisor_report();

        assert_eq!(summary.backend_name, "coreaudio");
        assert_eq!(summary.hardware.device_id, "coreaudio:default-output");
        assert_eq!(summary.hardware.device_name, "CoreAudio Default Output");
        assert_eq!(summary.hardware.sample_rate, 48_000);
        assert_eq!(summary.hardware.buffer_size, 512);
        assert_eq!(summary.hardware.output_channels, 2);
        assert_eq!(summary.hardware.sample_format, AudioSampleFormat::F32);
        assert_eq!(
            summary.hardware.lifecycle,
            HardwareLifecycleContract {
                ownership: signal_hardware::HardwareLifecycleOwnership::HostDrivenCallback,
                restart_policy: signal_hardware::HardwareRestartPolicy::HostMustRestart,
            }
        );
        assert!(!summary.hardware.simulated);
        assert_eq!(
            supervisor
                .observation
                .effective_config
                .active_output_device
                .as_deref(),
            Some("coreaudio:default-output")
        );
        assert_eq!(summary.hardware.backend_diagnostics.xrun_count, 0);
        assert_eq!(summary.hardware.backend_diagnostics.device_loss_count, 0);
        assert_eq!(
            summary.hardware.backend_diagnostics.health,
            signal_hardware::BackendHealth::Healthy
        );
        assert_eq!(
            summary.audio_pump.stream_state,
            LocalAudioStreamState::Running
        );
        assert_eq!(
            summary.audio_pump.transfer_policy,
            LocalAudioTransferPolicy {
                max_callback_frames: 512,
                max_transfer_channels: 2,
                zero_fill_unwritten_output: true,
            }
        );
        assert_eq!(summary.audio_pump.callback_count, 8);
        assert_eq!(summary.audio_pump.total_callback_frames, 8 * 512);
        assert_eq!(summary.audio_pump.total_runtime_output_frames, 8 * 512);
        assert_eq!(summary.audio_pump.copied_output_samples, 8 * 512 * 2);
        assert_eq!(summary.audio_pump.zero_filled_output_samples, 0);
        assert_eq!(summary.audio_pump.dropped_output_samples, 0);
        assert!(summary.audio_pump.last_callback_output_peak.is_some());
        assert_eq!(
            summary.audio_pump.last_runtime_graph_id.as_deref(),
            Some("signal.host.local.demo")
        );
        let plugin_state = summary
            .execution
            .last_plugin_state
            .as_ref()
            .expect("plugin instance state should be projected into local summary");
        assert_eq!(plugin_state.plugin_type_id, "plugin:clap:default");
        assert_eq!(plugin_state.instance_id, "instance:local:default");
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
        assert_eq!(observed_plugin_state.instance_id, "instance:local:default");
        assert_eq!(observed_plugin_state.lifecycle_state, "Active");
        assert_eq!(observed_plugin_state.readiness_state, "Ready");
        assert!(supervisor
            .render_json()
            .contains("\"plugin_instance_state_events\":"));
    }

    #[test]
    fn local_host_executes_track_bus_output_topology_through_audio_pump() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        let summary = host.boot_default().expect("default local host boot");
        let supervisor = host.supervisor_report();
        let topology = &supervisor.observation.execution_topology_summary;

        assert_eq!(
            summary.audio_pump.stream_state,
            LocalAudioStreamState::Running
        );
        assert_eq!(summary.audio_pump.callback_count, 8);
        assert_local_plugin_topology(&summary);
        assert_eq!(summary.topology, *topology);
        assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_node_3=output-main"));
    }

    #[test]
    fn local_host_shared_report_surfaces_topology_aware_host_io() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        host.boot_default().expect("default local host boot");
        let report = host.host_supervisor_report();

        assert_eq!(
            report.observation.host_io.hardware.backend_name,
            "coreaudio"
        );
        assert_eq!(
            report.observation.host_io.hardware.device_id,
            "coreaudio:default-output"
        );
        assert_eq!(report.observation.host_io.hardware.sample_rate, 48_000);
        assert_eq!(report.observation.host_io.hardware.buffer_size, 512);
        assert_eq!(report.observation.host_io.hardware.output_channels, 2);
        assert_eq!(
            report.observation.host_io.clocking.clock_source,
            RuntimeHostClockSource::Internal
        );
        assert_eq!(
            report.observation.host_io.clocking.ownership,
            signal_runtime::RuntimeHostLifecycleOwnership::HostDrivenCallback
        );
        assert_eq!(
            report.observation.host_io.clocking.restart_policy,
            signal_runtime::RuntimeHostRestartPolicy::HostMustRestart
        );
        assert!(
            (report.observation.host_io.clocking.callback_interval_ms - 10.666667).abs() < 0.001
        );
        assert_eq!(
            report.observation.host_io.latency.output_latency_samples,
            512
        );
        assert_eq!(report.observation.host_io.latency.graph_latency_samples, 24);
        assert_eq!(
            report
                .observation
                .host_io
                .latency
                .estimated_output_latency_samples,
            536
        );
        assert_eq!(
            report.observation.host_io.audio_pump.stream_state,
            RuntimeHostAudioStreamState::Running
        );
        assert_eq!(report.observation.host_io.audio_pump.callback_count, 8);
        assert!(report.observation.host_io.runtime_graph_id_matches_pump);
        assert_eq!(
            report
                .observation
                .observation
                .execution_topology_summary
                .node_count,
            4
        );
        assert_eq!(
            report
                .observation
                .observation
                .execution_topology_summary
                .track_lane_node_count,
            2
        );
        assert_eq!(
            report
                .observation
                .observation
                .execution_topology_summary
                .bus_node_count,
            1
        );
        assert_eq!(
            report
                .observation
                .observation
                .execution_topology_summary
                .console_node_count,
            1
        );
        assert!(report
            .render_json()
            .contains("\"node_id\":\"plugin-insert\""));
        assert!(report
            .render_json()
            .contains("\"plugin_sandbox_id\":\"local-default-sandbox\""));
        assert!(report
            .render_json()
            .contains("\"input_bus_id\":\"bus:track:lead\""));
        assert!(report
            .render_json()
            .contains("\"output_bus_id\":\"bus:mix:tracks\""));
        assert!(report
            .render_compact()
            .contains("host_audio_graph_matches_runtime=true"));
        assert!(report.render_multiline().contains("host_backend=coreaudio"));
        assert!(report.render_json().contains("\"device_loss_count\":0"));
        assert!(report
            .render_json()
            .contains("\"clock_source\":\"Internal\""));
        assert!(report
            .render_json()
            .contains("\"estimated_output_latency_samples\":536"));
    }

    #[test]
    fn local_host_shared_report_tracks_timeout_recovery_without_losing_topology() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        host.boot_with_timeout_recovery()
            .expect("timeout recovery local host boot");
        let report = host.host_supervisor_report();

        assert_eq!(
            report.observation.host_io.audio_pump.stream_state,
            RuntimeHostAudioStreamState::Running
        );
        assert!(report.observation.host_io.runtime_graph_id_matches_pump);
        assert_eq!(
            report
                .observation
                .observation
                .degradation_summary
                .xrun_count,
            1
        );
        assert_eq!(
            report
                .observation
                .observation
                .execution_topology_summary
                .track_lane_node_count,
            2
        );
        assert_eq!(
            report
                .observation
                .observation
                .execution_topology_summary
                .bus_node_count,
            1
        );
        assert_eq!(
            report
                .observation
                .observation
                .execution_topology_summary
                .console_node_count,
            1
        );
        assert!(report
            .render_json()
            .contains("\"node_id\":\"plugin-insert\""));
        assert!(report
            .render_json()
            .contains("\"plugin_sandbox_id\":\"local-default-sandbox\""));
        assert!(report.render_json().contains("\"lane_id\":\"track:lead\""));
        assert!(report
            .render_json()
            .contains("\"bus_group_id\":\"mix:tracks\""));
        assert!(report.render_compact().contains("xruns=1"));
        assert!(report
            .render_json()
            .contains("\"runtime_graph_id_matches_pump\":true"));
    }

    #[test]
    fn local_host_shared_report_tracks_device_loss_recovery() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        let summary = host
            .boot_with_device_loss_recovery()
            .expect("device loss recovery local host boot");
        let report = host.host_supervisor_report();

        assert_eq!(
            summary.execution.last_stop_reason,
            Some(StopReason::DeviceReconfigure)
        );
        assert_eq!(
            report.observation.host_io.audio_pump.stream_state,
            RuntimeHostAudioStreamState::Running
        );
        assert_eq!(
            report.observation.host_io.hardware.backend_health,
            BackendHealth::Healthy
        );
        assert_eq!(report.observation.host_io.hardware.device_loss_count, 1);
        assert_eq!(report.observation.host_io.hardware.restart_attempt_count, 1);
        assert_eq!(report.observation.host_io.hardware.restart_failure_count, 0);
        assert_eq!(
            report.observation.host_io.latency.output_latency_samples,
            512
        );
        assert!(report.observation.host_io.runtime_graph_id_matches_pump);
        assert_eq!(
            report
                .observation
                .observation
                .execution_topology_summary
                .track_lane_node_count,
            2
        );
        assert!(report
            .render_compact()
            .contains("host_backend_device_losses=1"));
        assert!(report.render_json().contains("\"restart_attempt_count\":1"));
    }

    #[test]
    fn local_host_shared_report_tracks_device_loss_restart_failure() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
        let error = host
            .boot_with_device_loss_restart_failure()
            .expect_err("device loss restart should fail");
        let report = host.host_supervisor_report();

        assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
        assert_eq!(
            report.observation.host_io.audio_pump.stream_state,
            RuntimeHostAudioStreamState::Faulted
        );
        assert_eq!(
            report.observation.host_io.hardware.backend_health,
            BackendHealth::Degraded
        );
        assert_eq!(report.observation.host_io.hardware.device_loss_count, 1);
        assert_eq!(report.observation.host_io.hardware.restart_attempt_count, 1);
        assert_eq!(report.observation.host_io.hardware.restart_failure_count, 1);
        assert_eq!(
            report.observation.host_io.clocking.clock_source,
            RuntimeHostClockSource::Internal
        );
        assert!(!report.observation.host_io.runtime_graph_id_matches_pump);
        assert_eq!(
            report
                .observation
                .observation
                .control_snapshot
                .last_stop_reason,
            Some(StopReason::DeviceReconfigure)
        );
        assert!(report
            .render_compact()
            .contains("host_backend_restart_failures=1"));
        assert!(report.render_json().contains("\"device_loss_count\":1"));
    }

    #[test]
    fn host_audio_transfer_bounds_channels_and_zero_fills_unwritten_output() {
        let runtime_output = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Count(ChannelCount(4)),
            vec![0.5, 0.4, 0.3, 0.2, 0.6, 0.5, 0.4, 0.3, 0.7, 0.6, 0.5, 0.4],
        );
        let stream = HardwareStreamConfig {
            device: AudioDeviceDescriptor {
                backend_name: "coreaudio",
                device_id: "coreaudio:default-output".into(),
                name: "CoreAudio Default Output".into(),
                default_input: false,
                default_output: true,
                max_input_channels: 0,
                max_output_channels: 2,
                nominal_sample_rate: SampleRate(48_000),
                preferred_buffer_sizes: vec![3],
            },
            direction: AudioStreamDirection::Output,
            sample_rate: SampleRate(48_000),
            buffer_size: 4,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            interleaved: true,
            clock_source: HardwareClockSource::Internal,
            lifecycle: HardwareLifecycleContract {
                ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                restart_policy: HardwareRestartPolicy::HostMustRestart,
            },
            latency: HardwareLatencyProfile::output_only(4),
            simulated: false,
        };
        let policy = LocalAudioTransferPolicy {
            max_callback_frames: 4,
            max_transfer_channels: 2,
            zero_fill_unwritten_output: true,
        };

        let transfer =
            super::transfer_runtime_output_to_host_buffer(&runtime_output, &stream, policy);

        assert_eq!(
            transfer.outcome,
            super::LocalAudioTransferOutcome {
                copied_samples: 6,
                zero_filled_samples: 2,
                dropped_samples: 6,
            }
        );
        assert!(transfer.output_peak >= 0.7);
    }

    #[test]
    fn local_host_mixed_watchdog_soak_tracks_deadlines_and_heartbeats() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut host = LocalRuntimeHost::new(runtime);
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
        assert_eq!(
            summary.execution.last_block_sequence, 13,
            "unexpected mixed watchdog soak summary: {summary:?}"
        );
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
        assert_eq!(supervisor.transport_fault_event_count(), 19);
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
        assert_runtime_automation_values(&supervisor, 14, 14, 3, 11, 2.0 / 7.0, 5.0 / 7.0, 0.18);
        assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
        assert_runtime_sequence_continuity(&supervisor, &[2, 2, 3, 4], 2, 13, 1, 2);
        assert_plugin_dispatch_summary(&summary, &supervisor, 2);
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
