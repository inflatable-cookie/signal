// Graph execution engine for real-time and anticipative processing
use crate::{
    bus, AudioBuffer, ExecutableGraph, GraphBlockReport, GraphExecutionContext,
    GraphExecutionRequest, GraphNodeRenderOverride, GraphParameterBatch,
    GraphRealtimeExecutionRequest,
};

mod anticipative;
mod realtime;

impl ExecutableGraph {
    /// Execute the graph with a full execution request.
    pub fn execute(&self, request: GraphExecutionRequest) -> (AudioBuffer, GraphBlockReport) {
        let GraphExecutionRequest {
            context,
            mut buffer,
            parameter_batch,
        } = request;
        let report =
            self.process_with_parameter_batch(&mut buffer, context, parameter_batch.as_ref());
        (buffer, report)
    }

    /// Process the graph with the given buffer.
    pub fn process(&self, buffer: &mut AudioBuffer) -> GraphBlockReport {
        self.process_with_context(buffer, GraphExecutionContext::default())
    }

    /// Process with a specific execution context.
    pub fn process_with_context(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
    ) -> GraphBlockReport {
        self.process_with_parameter_batch(buffer, context, None)
    }

    /// Process with parameter batch and optional node render overrides.
    pub fn process_with_parameter_batch_and_node_overrides(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        node_render_overrides: &[GraphNodeRenderOverride],
    ) -> GraphBlockReport {
        let input_peak = bus::peak_abs(buffer.samples());
        let planning = self.planning_summary(context.anticipative_enabled);
        let contract = self.contract_summary();
        let routing = self.routing_summary();
        let prepared = self.prepare_anticipative(buffer, &context, parameter_batch);
        let (working_buffer, report) = self.execute_realtime_from_prepared_with_node_overrides(
            GraphRealtimeExecutionRequest {
                input: buffer,
                input_peak,
                prepared,
                context,
                parameter_batch,
                planning: &planning,
                contract: &contract,
                routing: &routing,
                node_render_overrides,
                captured_bus_ids: &[],
            },
        );
        *buffer = working_buffer;
        report
    }

    /// Process with parameter batch (no node overrides).
    pub fn process_with_parameter_batch(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
    ) -> GraphBlockReport {
        self.process_with_parameter_batch_and_node_overrides(buffer, context, parameter_batch, &[])
    }
}
