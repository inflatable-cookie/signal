use signal_graph::synthetic_stereo_block;
use signal_plugin::{BlockDispatch, BlockPayload, PluginEvent};
use signal_plugin_clap::{BrokeredBlockOutcome, ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_primitives::FrameCount;
use signal_runtime::RuntimeError;

use super::super::ServerRuntimeHost;
use super::{
    demo_interaction_step,
    record_broker_failure_and_convert, record_runtime_fault, runtime_error_from_failure,
    runtime_watchdog_trigger, LifecycleRunSummary,
};

impl ServerRuntimeHost {
    pub(crate) fn prepare_brokered_block_request(
        &mut self,
        protocol: &ClapBlockProtocol,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        frame_count: u32,
    ) -> Result<(BlockDispatch, BlockPayload), RuntimeError> {
        let dispatch = protocol.block_dispatch(
            run.processing_epoch,
            block_sequence,
            frame_count,
            protocol.default_render_context(frame_count),
        );
        let mut payload = protocol.test_input_payload(block_sequence, frame_count);
        let automation_parameter_id = protocol.automation_parameter_id();
        if let Some(step) = demo_interaction_step() {
            for event in &mut payload.events.events {
                if let PluginEvent::ParameterValue(existing) = event {
                    if existing.parameter_id == automation_parameter_id {
                        existing.normalized_value = step.value;
                    }
                }
            }
        }
        run.last_plugin_automation_value =
            payload.events.events.iter().find_map(|event| match event {
                PluginEvent::ParameterValue(event)
                    if event.parameter_id == automation_parameter_id =>
                {
                    Some(event.normalized_value)
                }
                _ => None,
            });
        Ok((dispatch, payload))
    }

    pub(crate) fn complete_brokered_block_engine(
        &mut self,
        run: &mut LifecycleRunSummary,
        block_sequence: u64,
        frame_count: u32,
        _stored_result: &BrokeredBlockOutcome,
    ) -> Result<signal_runtime::RuntimeEngineBlockResult, RuntimeError> {
        let _ = self
            .runtime
            .apply_forecast_state_for_block(run.processing_epoch, block_sequence)?;
        self.runtime.process_engine_block(
            run.processing_epoch,
            block_sequence,
            synthetic_stereo_block(
                self.runtime.config().sample_rate,
                FrameCount(frame_count as usize),
                block_sequence.saturating_add(17),
            ),
        )
    }
}

signal_runtime::impl_host_runtime_block_support!(ServerRuntimeHost);
