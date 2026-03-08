use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_ipc::{
    PluginMessageEnvelope, PluginMessagePayload, SharedMemoryBroker, SharedMemoryTransportPayload,
};
use signal_plugin::{
    CompletionState, PluginFormat, PluginSandboxRequest, SandboxPolicy, SandboxWatchdogState,
    WatchdogOutcome, WatchdogTriggerReason,
};
use signal_plugin_clap::{
    sandbox_failure_event, BrokeredBlockOutcome, ClapBlockProtocol, ClapSandboxLifecycleHarness,
};
use signal_runtime::{
    BackendPolicyOverride, HandshakeRequest, PluginFaultKind, PluginSandboxSpec, PluginScanRequest,
    RuntimeConfigRequest, RuntimeError, RuntimeEventRecorder, RuntimeLifecycleApi,
    RuntimeObservationApi, RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimeProjectionApi, RuntimeSupervisorApi, RuntimeSupervisorReport, RuntimeWatchdogTrigger,
    SignalRuntime, WatchdogRestartRecord,
};

const WATCHDOG_TRIGGER_WINDOW_BLOCKS: u64 = 3;
const STEADY_STATE_BLOCKS: u64 = 8;
const SOAK_RESTART_EPISODES: u32 = 3;
const INTER_EPISODE_CONTINUITY_BLOCKS: u64 = 2;

#[derive(Clone, Debug, Default)]
struct ServerSupervisorState {
    scans_started: u64,
    sandboxes: u64,
    restarts: u64,
    teardowns: u64,
    backend_policy: Option<BackendPolicyTier>,
    last_scan_roots: Vec<String>,
    last_sandbox_id: Option<String>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultInjection {
    Timeout,
    Crash,
    HeartbeatMiss,
    EscalatingHeartbeatMisses { restart_episodes: u32 },
    MixedWatchdogEpisodes { restart_episodes: u32 },
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
    pub last_control_message: String,
    pub last_completion_state: CompletionState,
    pub last_block_sequence: u64,
    pub processing_epoch: u64,
    pub restart_count: u64,
    pub teardown_count: u64,
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
        let runtime_config = RuntimeConfigRequest::new(
            self.runtime.config().sample_rate.0,
            self.runtime.config().graph.block_size,
        );
        self.runtime.handshake(HandshakeRequest {
            client_version: "signal-host-server".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })?;
        self.runtime.configure(runtime_config)?;

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
        })?;

        let sandbox = PluginSandboxRequest::new(
            "server-default-sandbox",
            PluginFormat::Clap,
            SandboxPolicy::Strict,
        );
        self.ensure_plugin_sandbox(PluginSandboxSpec {
            sandbox_id: sandbox.sandbox_id.clone(),
            plugin_format: "clap",
        })?;
        self.runtime.set_active_plugin_sandboxes(1);
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
        let mut run = self.run_lifecycle(&protocol, &sandbox.sandbox_id, 1, &mut lifecycle)?;
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
                                    &sandbox.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    fault,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.runtime.increment_xruns();
                                self.handle_watchdog_recovery(
                                    &sandbox.sandbox_id,
                                    &mut lifecycle,
                                    &run,
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
                        &sandbox.sandbox_id,
                        "instance:server:default",
                        &run.shared_memory_lease_id,
                        run.processing_epoch,
                        fault,
                    );
                    record_runtime_fault(&mut self.runtime, &failure);
                    self.recover_sandbox(&sandbox.sandbox_id, &mut lifecycle, &run)?;
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
                                &sandbox.sandbox_id,
                                "instance:server:default",
                                &run.shared_memory_lease_id,
                                run.processing_epoch,
                                fault,
                            );
                            record_runtime_fault(&mut self.runtime, &failure);
                            self.handle_watchdog_recovery(
                                &sandbox.sandbox_id,
                                &mut lifecycle,
                                &run,
                            )?;
                            break;
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
                                    &sandbox.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    FaultInjection::HeartbeatMiss,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                self.handle_watchdog_recovery(
                                    &sandbox.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                )?;
                                if restart_episode + 1 < restart_episodes {
                                    let prior_history = run.recovery_history();
                                    run = self.run_lifecycle(
                                        &protocol,
                                        &sandbox.sandbox_id,
                                        restart_episode as u64 + 2,
                                        &mut lifecycle,
                                    )?;
                                    run.apply_recovery_history(prior_history);
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
                                    &sandbox.sandbox_id,
                                    "instance:server:default",
                                    &run.shared_memory_lease_id,
                                    run.processing_epoch,
                                    episode_fault,
                                );
                                record_runtime_fault(&mut self.runtime, &failure);
                                if matches!(episode_fault, FaultInjection::Timeout) {
                                    self.runtime.increment_xruns();
                                }
                                self.handle_watchdog_recovery(
                                    &sandbox.sandbox_id,
                                    &mut lifecycle,
                                    &run,
                                )?;
                                if restart_episode + 1 < restart_episodes {
                                    let prior_history = run.recovery_history();
                                    run = self.run_lifecycle(
                                        &protocol,
                                        &sandbox.sandbox_id,
                                        restart_episode as u64 + 2,
                                        &mut lifecycle,
                                    )?;
                                    run.apply_recovery_history(prior_history);
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
            let prior_history = run.recovery_history();
            let next_epoch = match fault {
                FaultInjection::EscalatingHeartbeatMisses { restart_episodes } => {
                    restart_episodes as u64 + 1
                }
                FaultInjection::MixedWatchdogEpisodes { restart_episodes } => {
                    restart_episodes as u64 + 1
                }
                _ => 2,
            };
            run = self.run_lifecycle(&protocol, &sandbox.sandbox_id, next_epoch, &mut lifecycle)?;
            run.apply_recovery_history(prior_history);
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
                last_control_message: run.last_control_message.clone(),
                last_completion_state: run.last_completion_state,
                last_block_sequence: run.last_block_sequence,
                processing_epoch: run.processing_epoch,
                restart_count: self.supervisor.restarts,
                teardown_count: self.supervisor.teardowns,
            },
            transport: ServerTransportSummary {
                sandbox_id: self
                    .supervisor
                    .last_sandbox_id
                    .clone()
                    .unwrap_or_else(|| sandbox.sandbox_id.clone()),
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
            .map_err(runtime_error_from_io)?;
        let mut responses = Vec::with_capacity(control_sequence.len());
        for request in control_sequence.iter().cloned() {
            match lifecycle.handle(request) {
                Ok(response) => responses.push(response),
                Err(failure) => {
                    record_runtime_fault(&mut self.runtime, &failure);
                    return Err(runtime_error_from_failure(&failure));
                }
            }
        }
        let heartbeat = lifecycle
            .handle(protocol.heartbeat_request(sandbox_id, Some(processing_epoch)))
            .map_err(|failure| {
                record_runtime_fault(&mut self.runtime, &failure);
                runtime_error_from_failure(&failure)
            })?;
        responses.push(heartbeat);

        let (shared_memory_lease_id, transport) = extract_prepare_metadata(&responses);

        Ok(LifecycleRunSummary {
            sandbox_id: sandbox_id.to_string(),
            control_requests: control_sequence.len() + 1,
            control_responses: responses.len(),
            heartbeat_responses: 1,
            processed_blocks: 0,
            last_control_message: responses
                .last()
                .map(|response| response.message.name.clone())
                .unwrap_or_default(),
            last_completion_state: CompletionState::Idle,
            last_block_sequence: 0,
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
        for block_offset in 0..block_count {
            let block_sequence = self.runtime.allocate_block_sequence();
            let should_timeout = simulate_timeout && block_offset == 0;
            let _ = self.run_realtime_cycle(
                protocol,
                run,
                block_sequence,
                lifecycle,
                false,
                should_timeout,
            )?;
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
        if !self.poll_heartbeat(
            protocol,
            run,
            lifecycle,
            simulate_heartbeat_miss,
            Some(block_sequence),
        )? {
            return Ok(None);
        }

        self.execute_block(protocol, run, block_sequence, lifecycle, simulate_timeout)
            .map(Some)
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

        let heartbeat = lifecycle
            .handle(protocol.heartbeat_request(run.sandbox_id.as_str(), Some(run.processing_epoch)))
            .map_err(|failure| {
                record_runtime_fault(&mut self.runtime, &failure);
                runtime_error_from_failure(&failure)
            })?;
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
        let payload = protocol.test_input_payload(block_sequence, frame_count);
        protocol
            .write_block_payload(&self.broker, &transport, &dispatch, &payload)
            .map_err(runtime_error_from_io)?;
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
            .map_err(runtime_error_from_io)?;
        let event_summary = stored_result.output.events.summary();
        run.processed_blocks = run.processed_blocks.saturating_add(1);
        run.last_completion_state = stored_result.result.slot.state;
        run.last_block_sequence = block_sequence;
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
        self.runtime.record_block_sequence(
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
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
    ) -> Result<(), RuntimeError> {
        let current_transport = run.transport.clone().ok_or_else(|| {
            RuntimeError::new(
                signal_runtime::RuntimeErrorKind::ResourceUnavailable,
                "lifecycle completed without brokered shared-memory transport",
            )
        })?;
        self.runtime.set_active_plugin_sandboxes(0);
        self.teardown_plugin_sandbox(sandbox_id)?;
        self.broker
            .destroy_region(&current_transport)
            .map_err(runtime_error_from_io)?;
        lifecycle
            .teardown_active_transport()
            .map_err(runtime_error_from_io)?;
        self.restart_plugin_sandbox(sandbox_id)?;
        self.runtime.set_active_plugin_sandboxes(1);
        Ok(())
    }

    fn handle_watchdog_recovery(
        &mut self,
        sandbox_id: &str,
        lifecycle: &mut ClapSandboxLifecycleHarness,
        run: &LifecycleRunSummary,
    ) -> Result<(), RuntimeError> {
        self.recover_sandbox(sandbox_id, lifecycle, run)
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
    last_control_message: String,
    last_completion_state: CompletionState,
    last_block_sequence: u64,
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
}

#[derive(Clone, Debug, Default)]
struct RecoveryHistory {
    control_requests: usize,
    control_responses: usize,
    heartbeat_responses: usize,
    processed_blocks: usize,
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

fn runtime_watchdog_trigger(reason: WatchdogTriggerReason) -> RuntimeWatchdogTrigger {
    match reason {
        WatchdogTriggerReason::DeadlineMisses => RuntimeWatchdogTrigger::DeadlineMisses,
        WatchdogTriggerReason::HeartbeatMisses => RuntimeWatchdogTrigger::HeartbeatMisses,
    }
}

fn record_runtime_fault(runtime: &mut SignalRuntime, failure: &signal_ipc::PluginMessageEnvelope) {
    if let signal_ipc::PluginMessagePayload::SandboxFailure {
        sandbox_id,
        error_kind,
        detail,
        processing_epoch,
        ..
    } = &failure.payload
    {
        let kind = match error_kind.as_str() {
            "timeout" => PluginFaultKind::Timeout,
            "crash" => PluginFaultKind::Crash,
            _ => PluginFaultKind::ProtocolViolation,
        };
        runtime.record_plugin_sandbox_fault(
            sandbox_id.clone(),
            kind,
            detail.clone(),
            *processing_epoch,
        );
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
        signal_ipc::PluginMessagePayload::SandboxFailure { detail, .. } => RuntimeError {
            kind: signal_runtime::RuntimeErrorKind::PluginFailure,
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

impl RuntimeSupervisorApi for ServerRuntimeHost {
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
        self.supervisor.last_sandbox_id = Some(request.sandbox_id);
        Ok(signal_runtime::SandboxHandle(self.supervisor.sandboxes))
    }

    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError> {
        self.supervisor.teardowns = self.supervisor.teardowns.saturating_add(1);
        self.supervisor.last_sandbox_id = Some(sandbox_id.to_string());
        Ok(())
    }

    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError> {
        self.supervisor.restarts = self.supervisor.restarts.saturating_add(1);
        self.supervisor.last_sandbox_id = Some(sandbox_id.to_string());
        Ok(())
    }

    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError> {
        self.supervisor.backend_policy = Some(request.tier);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ServerRuntimeHost;
    use signal_plugin::{CompletionState, WatchdogTriggerReason};
    use signal_runtime::{
        RuntimeConfig, RuntimeObservationApi, RuntimeSupervisorReport, SignalRuntime,
    };

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
            summary.execution.last_completion_state,
            CompletionState::Completed
        );
        assert_eq!(summary.execution.processed_blocks, 10);
        assert_eq!(summary.execution.last_block_sequence, 9);
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
        assert_runtime_automation_values(&supervisor, 8, 8, 2, 6, 0.2, 0.55, 0.10);
        assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
        assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 9, 0, 1);
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
        assert_runtime_automation_values(&supervisor, 14, 14, 2, 12, 0.2, 0.95, 0.26);
        assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
        assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
        assert_eq!(supervisor.event_count(), 24);
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
