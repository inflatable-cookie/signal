use super::*;

pub(super) fn handshake_and_configure_with_disabled_forecast(
    runtime: &mut SignalRuntime,
    anticipative_enabled: bool,
) {
    handshake_and_configure_with_anticipative(runtime, anticipative_enabled);
    runtime
        .set_prework_forecast_mode(RuntimePreworkForecastMode::Disabled)
        .unwrap();
}

pub(super) fn seed_pending_prework_targets(
    runtime: &mut SignalRuntime,
    admitted_from_block_sequence: u64,
    target_block_sequences: &[u64],
) {
    runtime.engine.pending_prework_targets.clear();
    let targets = target_block_sequences
        .iter()
        .map(|target_block_sequence| RuntimePreworkWindowTarget {
            target_block_sequence: *target_block_sequence,
            admitted_from_block_sequence,
            buffer: synthetic_stereo_block(
                runtime.config.sample_rate,
                FrameCount(runtime.config.graph.block_size),
                *target_block_sequence,
            ),
            parameter_epoch_override: None,
            transport_override: None,
        })
        .collect::<Vec<_>>();
    let graph_id = runtime
        .engine
        .graph
        .as_ref()
        .map(|graph| graph.graph_id().to_string());
    runtime.engine.reconcile_pending_prework_targets(
        &targets,
        graph_id.as_deref(),
        runtime.projection_epoch,
        runtime.latest_parameter_epoch,
        runtime.applied_transport,
        runtime.config.graph.block_size,
    );
}

pub(super) fn apply_current_forecast_block_state(runtime: &mut SignalRuntime, block_sequence: u64) {
    let policy = runtime
        .prework_forecast_policy
        .clone()
        .expect("forecast policy configured");
    runtime
        .apply_forecast_transport_projection(
            runtime.forecast_transport_projection_for_block(block_sequence, &policy),
        )
        .expect("apply forecast transport projection");
    runtime
        .apply_parameter_batch(runtime.forecast_parameter_batch_for_block(block_sequence, &policy))
        .expect("apply forecast parameter batch");
}
