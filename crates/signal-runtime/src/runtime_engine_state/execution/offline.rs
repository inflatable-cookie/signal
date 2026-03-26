use super::super::super::*;

impl RuntimeEngineState {
    pub(crate) fn render_offline_block(
        &self,
        context: GraphExecutionContext,
        buffer: AudioBuffer,
        parameter_batch: Option<GraphParameterBatch>,
        node_render_overrides: &[GraphNodeRenderOverride],
        captured_bus_ids: &[String],
    ) -> Result<(AudioBuffer, Vec<GraphCapturedBusOutput>), RuntimeError> {
        let graph = self.graph.as_ref().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "no executable graph has been applied",
            )
        })?;
        let planning = graph.planning_summary(context.anticipative_enabled);
        let contract = graph.contract_summary();
        let routing = graph.routing_summary();
        let (output, _, captured_buses) = graph
            .execute_realtime_from_prepared_with_node_overrides_and_bus_captures(
                &buffer,
                peak_abs(buffer.samples()),
                None,
                context,
                parameter_batch.as_ref(),
                &planning,
                &contract,
                &routing,
                node_render_overrides,
                captured_bus_ids,
            );
        Ok((output, captured_buses))
    }
}
