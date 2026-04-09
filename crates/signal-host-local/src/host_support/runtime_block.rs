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

signal_runtime::impl_host_runtime_block_support!(LocalRuntimeHost);
