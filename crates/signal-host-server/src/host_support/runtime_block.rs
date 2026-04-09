use signal_graph::synthetic_stereo_block;
use signal_plugin::{BlockDispatch, BlockPayload};
use signal_plugin_clap::{BrokeredBlockOutcome, ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_primitives::FrameCount;
use signal_runtime::RuntimeError;

use super::super::ServerRuntimeHost;
use super::{
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
        let payload = protocol.test_input_payload(block_sequence, frame_count);
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
