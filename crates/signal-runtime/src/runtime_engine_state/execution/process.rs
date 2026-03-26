#[path = "process_prepare.rs"]
mod process_prepare;
#[path = "process_report.rs"]
mod process_report;

use super::super::super::*;

impl RuntimeEngineState {
    pub(crate) fn process_block(
        &mut self,
        context: GraphExecutionContext,
        transport: Option<TransportProjection>,
        buffer: AudioBuffer,
        parameter_batch: Option<GraphParameterBatch>,
    ) -> Result<RuntimeEngineBlockResult, RuntimeError> {
        let graph_id = self
            .graph
            .as_ref()
            .ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidState,
                    "no executable graph has been applied",
                )
            })?
            .graph_id()
            .to_string();
        self.retire_stale_plugin_node_renders(context.processing_epoch, context.block_sequence);

        let input_signature = hash_audio_buffer(&buffer);
        self.retire_unready_or_mismatched_prework_for_current_block(
            graph_id.as_str(),
            &context,
            &buffer,
            input_signature,
        );

        let (prepared, cache_hit, prepared_was_used) = self.prepare_dispatch_for_block(
            &graph_id,
            &context,
            transport,
            &buffer,
            input_signature,
        )?;
        let plugin_node_renders =
            self.realize_plugin_node_renders(context.processing_epoch, context.block_sequence);
        let graph = self.graph.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "no executable graph has been applied",
            )
        })?;
        let planning = graph.planning_summary(context.anticipative_enabled);
        let contract = graph.contract_summary();
        let routing = graph.routing_summary();

        let (output, report) = graph.execute_realtime_from_prepared_with_node_overrides(
            &buffer,
            peak_abs(buffer.samples()),
            prepared,
            context,
            parameter_batch.as_ref(),
            &planning,
            &contract,
            &routing,
            &plugin_node_renders,
        );

        let meter_sources =
            self.apply_graph_block_report(report, &contract, cache_hit, prepared_was_used);
        Ok(RuntimeEngineBlockResult {
            snapshot: self.snapshot.clone(),
            output,
            meter_sources,
        })
    }
}
