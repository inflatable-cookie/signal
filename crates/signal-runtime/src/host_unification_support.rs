/// Plan for exercising retrying timeout recovery in a test harness.
///
/// Describes a sequence of injected failures and whether to attempt a clean
/// recovery after all failures have been exhausted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeoutRecoveryRetryPlan<'a, Failure> {
    /// Sequence of failure modes to inject, applied in order.
    pub failures: &'a [Failure],
    /// Detail message used when the terminal failure is reached.
    pub terminal_detail: &'a str,
    /// Whether to attempt a clean recovery after all failures have been exhausted.
    pub recover_after_failures: bool,
}

/// Plan for exercising repeated watchdog recovery cycles in a test harness.
///
/// `restart_episodes` controls how many watchdog-trigger → recover cycles to
/// run; `mixed_faults` alternates between heartbeat-miss and timeout faults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepeatedWatchdogRecoveryPlan {
    /// Number of watchdog-trigger → recover cycles to run.
    pub restart_episodes: u32,
    /// Whether to alternate between heartbeat-miss and timeout faults across episodes.
    pub mixed_faults: bool,
}

/// Implements the realtime block-cycle execution helpers (`execute_block_sequence`,
/// `run_realtime_cycle`, `prework_service_pressure`, `poll_heartbeat`) on a host
/// harness type that owns a [`SignalRuntime`] field named `runtime`.
#[macro_export]
macro_rules! impl_host_runtime_cycle_support {
    ($host_ty:path) => {
        impl $host_ty {
            pub(crate) fn execute_block_sequence(
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

            pub(crate) fn run_realtime_cycle(
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

            pub(crate) fn prework_service_pressure(
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

            pub(crate) fn poll_heartbeat(
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
                    .handle(
                        protocol
                            .heartbeat_request(run.sandbox_id.as_str(), Some(run.processing_epoch)),
                    )
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
        }
    };
}

/// Implements the boot-time watchdog and timeout recovery helpers
/// (`apply_timeout_recovery_failure`, `apply_retrying_timeout_recovery`,
/// `apply_interleaved_timeout_recovery`, `apply_repeated_watchdog_recovery`,
/// `walk_timeout_watchdog`) on a host harness type.
#[macro_export]
macro_rules! impl_host_boot_recovery_helpers {
    ($host_ty:path, $sandbox_ty:path, $timeout_plan_ty:ty, $instance_id:expr) => {
        impl $host_ty {
            pub(super) fn apply_timeout_recovery_failure(
                &mut self,
                protocol: &ClapBlockProtocol,
                sandbox: &$sandbox_ty,
                run: &mut LifecycleRunSummary,
                lifecycle: &mut ClapSandboxLifecycleHarness,
                failure: RecoveryFailureInjection,
                detail: &str,
            ) -> Result<(), RuntimeError> {
                self.walk_timeout_watchdog(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    |this, run, lifecycle| match this.handle_watchdog_recovery(
                        protocol,
                        &sandbox.request.sandbox_id,
                        lifecycle,
                        run,
                        Some(failure),
                    ) {
                        Ok(_) => Err(RuntimeError::new(
                            RuntimeErrorKind::ResourceUnavailable,
                            detail,
                        )),
                        Err(error) => Err(error),
                    },
                )
            }

            pub(super) fn apply_retrying_timeout_recovery(
                &mut self,
                protocol: &ClapBlockProtocol,
                sandbox: &$sandbox_ty,
                run: &mut LifecycleRunSummary,
                lifecycle: &mut ClapSandboxLifecycleHarness,
                plan: $timeout_plan_ty,
            ) -> Result<(), RuntimeError> {
                let TimeoutRecoveryRetryPlan {
                    failures,
                    terminal_detail,
                    recover_after_failures,
                } = plan;
                self.walk_timeout_watchdog(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    |this, run, lifecycle| {
                        for (index, failure) in failures.iter().copied().enumerate() {
                            let result = this.handle_watchdog_recovery(
                                protocol,
                                &sandbox.request.sandbox_id,
                                lifecycle,
                                run,
                                Some(failure),
                            );
                            if result.is_ok() {
                                let detail = if index + 1 == failures.len() {
                                    terminal_detail
                                } else {
                                    "expected deferred teardown recovery failure"
                                };
                                return Err(RuntimeError::new(
                                    RuntimeErrorKind::ResourceUnavailable,
                                    detail,
                                ));
                            }
                        }
                        if recover_after_failures {
                            *run = this.handle_watchdog_recovery(
                                protocol,
                                &sandbox.request.sandbox_id,
                                lifecycle,
                                run,
                                None,
                            )?;
                        }
                        Ok(())
                    },
                )
            }

            pub(super) fn apply_interleaved_timeout_recovery(
                &mut self,
                protocol: &ClapBlockProtocol,
                sandbox: &$sandbox_ty,
                run: &mut LifecycleRunSummary,
                lifecycle: &mut ClapSandboxLifecycleHarness,
            ) -> Result<(), RuntimeError> {
                self.walk_timeout_watchdog(
                    protocol,
                    sandbox,
                    run,
                    lifecycle,
                    |this, run, lifecycle| {
                        let first_error = this.handle_watchdog_recovery(
                            protocol,
                            &sandbox.request.sandbox_id,
                            lifecycle,
                            run,
                            Some(RecoveryFailureInjection::DeferredOldTransportTeardown),
                        );
                        if first_error.is_ok() {
                            return Err(RuntimeError::new(
                                RuntimeErrorKind::ResourceUnavailable,
                                "expected deferred teardown recovery failure",
                            ));
                        }
                        match this.handle_watchdog_recovery(
                            protocol,
                            &sandbox.request.sandbox_id,
                            lifecycle,
                            run,
                            Some(RecoveryFailureInjection::CompetingOverlapAttach),
                        ) {
                            Ok(_) => Err(RuntimeError::new(
                                RuntimeErrorKind::ResourceUnavailable,
                                "expected interleaved recovery failures",
                            )),
                            Err(error) => Err(error),
                        }
                    },
                )
            }

            pub(super) fn apply_repeated_watchdog_recovery(
                &mut self,
                protocol: &ClapBlockProtocol,
                sandbox: &$sandbox_ty,
                run: &mut LifecycleRunSummary,
                lifecycle: &mut ClapSandboxLifecycleHarness,
                plan: RepeatedWatchdogRecoveryPlan,
            ) -> Result<(), RuntimeError> {
                let RepeatedWatchdogRecoveryPlan {
                    restart_episodes,
                    mixed_faults,
                } = plan;
                for restart_episode in 0..restart_episodes {
                    let episode_fault = if mixed_faults && restart_episode % 2 == 1 {
                        FaultInjection::Timeout
                    } else {
                        FaultInjection::HeartbeatMiss
                    };
                    for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                        let block_sequence = self.runtime.allocate_block_sequence();
                        let (simulate_heartbeat_miss, simulate_timeout) = match episode_fault {
                            FaultInjection::HeartbeatMiss => (true, false),
                            FaultInjection::Timeout => (false, true),
                            _ => (false, false),
                        };
                        let outcome = self.run_realtime_cycle(
                            protocol,
                            run,
                            block_sequence,
                            lifecycle,
                            simulate_heartbeat_miss,
                            simulate_timeout,
                        )?;
                        let watchdog_fired = match episode_fault {
                            FaultInjection::HeartbeatMiss => {
                                outcome.is_none() && run.current_watchdog_triggered
                            }
                            FaultInjection::Timeout => outcome.as_ref().is_some_and(|outcome| {
                                outcome.result.slot.state == CompletionState::TimedOut
                                    && run.current_watchdog_triggered
                            }),
                            _ => false,
                        };
                        if watchdog_fired {
                            let failure = build_fault_envelope(
                                &sandbox.request.sandbox_id,
                                $instance_id,
                                &run.shared_memory_lease_id,
                                run.processing_epoch,
                                episode_fault,
                            );
                            record_runtime_fault(&mut self.runtime, &failure);
                            if matches!(episode_fault, FaultInjection::Timeout) {
                                self.runtime.increment_xruns();
                            }
                            *run = self.handle_watchdog_recovery(
                                protocol,
                                &sandbox.request.sandbox_id,
                                lifecycle,
                                run,
                                None,
                            )?;
                            if restart_episode + 1 < restart_episodes {
                                self.execute_block_sequence(
                                    protocol,
                                    run,
                                    INTER_EPISODE_CONTINUITY_BLOCKS,
                                    lifecycle,
                                    false,
                                )?;
                            }
                            break;
                        }
                    }
                }
                Ok(())
            }

            pub(super) fn walk_timeout_watchdog<F>(
                &mut self,
                protocol: &ClapBlockProtocol,
                sandbox: &$sandbox_ty,
                run: &mut LifecycleRunSummary,
                lifecycle: &mut ClapSandboxLifecycleHarness,
                on_trigger: F,
            ) -> Result<(), RuntimeError>
            where
                F: FnOnce(
                    &mut $host_ty,
                    &mut LifecycleRunSummary,
                    &mut ClapSandboxLifecycleHarness,
                ) -> Result<(), RuntimeError>,
            {
                let mut on_trigger = Some(on_trigger);
                for _ in 0..WATCHDOG_TRIGGER_WINDOW_BLOCKS {
                    let block_sequence = self.runtime.allocate_block_sequence();
                    let timeout_result = self.run_realtime_cycle(
                        protocol,
                        run,
                        block_sequence,
                        lifecycle,
                        false,
                        true,
                    )?;
                    if let Some(outcome) = timeout_result {
                        if outcome.result.slot.state == CompletionState::TimedOut
                            && run.current_watchdog_triggered
                        {
                            let failure = build_fault_envelope(
                                &sandbox.request.sandbox_id,
                                $instance_id,
                                &run.shared_memory_lease_id,
                                run.processing_epoch,
                                FaultInjection::Timeout,
                            );
                            record_runtime_fault(&mut self.runtime, &failure);
                            self.runtime.increment_xruns();
                            if let Some(on_trigger) = on_trigger.take() {
                                return on_trigger(self, run, lifecycle);
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    };
}

/// Implements the brokered block execution helper (`execute_block`) on a host harness
/// type, handling payload dispatch, completion-slot tracking, and engine result
/// recording via the embedded [`SignalRuntime`].
#[macro_export]
macro_rules! impl_host_runtime_block_support {
    ($host_ty:path) => {
        impl $host_ty {
            pub(crate) fn execute_block(
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
                let (dispatch, payload) = self.prepare_brokered_block_request(
                    protocol,
                    run,
                    block_sequence,
                    frame_count,
                )?;
                self.runtime
                    .record_block_dispatch(signal_runtime::BlockDispatchRecord {
                        sandbox_id: run.sandbox_id.clone(),
                        lease_id: run.shared_memory_lease_id.clone(),
                        processing_epoch: run.processing_epoch,
                        block_sequence,
                        frame_count,
                        stage: signal_runtime::BlockDispatchStage::Requested,
                        completion_state: None,
                    });
                protocol
                    .write_block_payload(&self.broker, &transport, &dispatch, &payload)
                    .map_err(|error| {
                        record_broker_failure_and_convert(
                            &mut self.runtime,
                            run.sandbox_id.as_str(),
                            Some(run.shared_memory_lease_id.clone()),
                            Some(run.processing_epoch),
                            Some(block_sequence),
                            signal_runtime::BrokerFailureStage::PayloadWrite,
                            error,
                        )
                    })?;
                self.runtime.record_completion_slot_transition(
                    run.sandbox_id.as_str(),
                    run.shared_memory_lease_id.as_str(),
                    run.processing_epoch,
                    block_sequence,
                    signal_runtime::CompletionSlotStage::ReadyForProcessing,
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
                            signal_runtime::BrokerFailureStage::PayloadRead,
                            error,
                        )
                    })?;
                if simulate_timeout {
                    self.runtime.record_completion_slot_transition(
                        run.sandbox_id.as_str(),
                        run.shared_memory_lease_id.as_str(),
                        run.processing_epoch,
                        block_sequence,
                        signal_runtime::CompletionSlotStage::TimedOut,
                    );
                    if stored_result.result.fallback_applied {
                        self.runtime.record_completion_slot_transition(
                            run.sandbox_id.as_str(),
                            run.shared_memory_lease_id.as_str(),
                            run.processing_epoch,
                            block_sequence,
                            signal_runtime::CompletionSlotStage::FallbackApplied,
                        );
                    }
                } else {
                    self.runtime.record_completion_slot_transition(
                        run.sandbox_id.as_str(),
                        run.shared_memory_lease_id.as_str(),
                        run.processing_epoch,
                        block_sequence,
                        signal_runtime::CompletionSlotStage::Processing,
                    );
                    if stored_result.result.slot.state == signal_plugin::CompletionState::Completed
                    {
                        self.runtime.record_completion_slot_transition(
                            run.sandbox_id.as_str(),
                            run.shared_memory_lease_id.as_str(),
                            run.processing_epoch,
                            block_sequence,
                            signal_runtime::CompletionSlotStage::Completed,
                        );
                    }
                }
                let event_summary = stored_result.output.events.summary();
                let engine_result = self.complete_brokered_block_engine(
                    run,
                    block_sequence,
                    frame_count,
                    &stored_result,
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
                run.last_parameter_modulation_event_count =
                    event_summary.parameter_modulation_events;
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
                let dispatch_stage = if stored_result.result.slot.state
                    == signal_plugin::CompletionState::TimedOut
                {
                    signal_runtime::BlockDispatchStage::TimedOut
                } else {
                    signal_runtime::BlockDispatchStage::Completed
                };
                self.runtime
                    .record_block_dispatch(signal_runtime::BlockDispatchRecord {
                        sandbox_id: run.sandbox_id.clone(),
                        lease_id: run.shared_memory_lease_id.clone(),
                        processing_epoch: run.processing_epoch,
                        block_sequence,
                        frame_count,
                        stage: dispatch_stage,
                        completion_state: Some(stored_result.result.slot.state),
                    });
                self.runtime.record_block_sequence(
                    run.sandbox_id.as_str(),
                    run.processing_epoch,
                    run.shared_memory_lease_id.as_str(),
                    block_sequence,
                );
                run.last_output_first_sample = stored_result.output.audio.first_sample();
                if stored_result.result.slot.state == signal_plugin::CompletionState::TimedOut {
                    run.deadline_misses = run.deadline_misses.saturating_add(1);
                }
                if let signal_plugin::WatchdogOutcome::RestartRequired {
                    reason,
                    consecutive_misses: _,
                } = run
                    .watchdog
                    .record_block_completion(stored_result.result.slot.state)
                {
                    run.watchdog_triggered = true;
                    run.watchdog_trigger_reason = Some(reason);
                    run.current_watchdog_triggered = true;
                    self.runtime
                        .record_watchdog_restart(signal_runtime::WatchdogRestartRecord {
                            sandbox_id: run.sandbox_id.clone(),
                            trigger: runtime_watchdog_trigger(reason),
                            processing_epoch: run.processing_epoch,
                        });
                } else {
                    ()
                }
                Ok(stored_result)
            }
        }
    };
}
