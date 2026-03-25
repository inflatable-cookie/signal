use signal_ipc::SharedMemoryTransportPayload;
use signal_plugin::{CompletionState, SandboxWatchdogState};
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    BrokerFailureStage, HeartbeatCycleStage, PluginSandboxLifecycleStage,
    PluginSandboxTransportStage, TransportAttachIntent,
};

use super::super::LocalRuntimeHost;
use super::{
    extract_prepare_metadata, lifecycle_stage_for_request,
    plugin_instance_state_record_from_response, record_broker_failure_and_convert,
    record_runtime_fault, runtime_error_from_failure, transport_attach_intent,
    LifecycleRunSummary,
};

impl LocalRuntimeHost {
    fn run_lifecycle(
        &mut self,
        protocol: &ClapBlockProtocol,
        sandbox_id: &str,
        processing_epoch: u64,
        lifecycle: &mut ClapSandboxLifecycleHarness,
    ) -> Result<LifecycleRunSummary, signal_runtime::RuntimeError> {
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
            let intent = transport_attach_intent(processing_epoch);
            if let Err(error) = self
                .runtime
                .begin_transport_session_with_metadata_for_epoch(
                    sandbox_id,
                    shared_memory_lease_id.as_str(),
                    transport.region_id.as_str(),
                    intent,
                    Some(processing_epoch),
                    match intent {
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
            self.runtime
                .end_transport_session(sandbox_id, lease_id, transport.region_id.as_str());
        }
    }
}
