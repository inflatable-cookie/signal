use crate::{
    bus, ExecutableGraph, GraphExecutionContext, GraphExecutionLane, GraphParameterBatch,
    GraphPreparedBus, GraphPreparedDispatch,
};

impl ExecutableGraph {
    /// Prepare anticipative dispatches (pre-work) for the graph.
    pub fn prepare_anticipative(
        &self,
        buffer: &crate::AudioBuffer,
        context: &GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
    ) -> Option<GraphPreparedDispatch> {
        let planning = self.planning_summary(context.anticipative_enabled);
        let anticipative_dispatches = planning
            .dispatches
            .iter()
            .filter(|dispatch| dispatch.lane == GraphExecutionLane::Anticipative)
            .collect::<Vec<_>>();
        if anticipative_dispatches.is_empty() {
            return None;
        }

        let mut prepared = bus::seeded_bus_state(buffer);
        self.execute_dispatches(
            &mut prepared,
            &anticipative_dispatches,
            context.anticipative_enabled,
            parameter_batch,
            &[],
        );
        let latencies = prepared.latencies.clone();
        let tails = prepared.tails.clone();

        Some(GraphPreparedDispatch {
            output_peak: bus::peak_abs_across_buses(&prepared),
            buses: prepared
                .buses
                .into_iter()
                .map(|(bus_id, buffer)| GraphPreparedBus {
                    latency_samples: latencies.get(&bus_id).copied().unwrap_or(0),
                    tail_samples: tails.get(&bus_id).copied().unwrap_or(0),
                    bus_id,
                    buffer,
                })
                .collect(),
            dispatch_count: anticipative_dispatches.len(),
        })
    }
}
