use signal_plugin::{BlockDispatch, BlockPayload};
use signal_plugin_clap::{BrokeredBlockOutcome, ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::RuntimeError;

use super::super::LocalRuntimeHost;
use super::{
    payload_automation_value, record_broker_failure_and_convert, record_runtime_fault,
    runtime_error_from_failure, runtime_watchdog_trigger, LifecycleRunSummary,
};

impl LocalRuntimeHost {
    pub(crate) fn prepare_brokered_block_request(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        frame_count: u32,
    ) -> Result<(BlockDispatch, BlockPayload), RuntimeError> {
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
        run.last_plugin_render_context = Some(dispatch.render_context);
        run.last_plugin_automation_value =
            payload_automation_value(&payload, protocol.automation_parameter_id());
        Ok((dispatch, payload))
    }

    pub(crate) fn complete_brokered_block_engine(
        &mut self,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        _frame_count: u32,
        stored_result: &BrokeredBlockOutcome,
    ) -> Result<signal_runtime::RuntimeEngineBlockResult, RuntimeError> {
        self.apply_plugin_node_render_for_block(run, block_sequence, stored_result)?;
        self.process_engine_block_through_output_pump(run.processing_epoch, block_sequence)
    }
}

impl LocalRuntimeHost {
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
