use super::*;

pub(crate) fn json_runtime_block_execution_summary(
    summary: &RuntimeBlockExecutionSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"processed_blocks\":{},",
            "\"last_processing_epoch\":{},",
            "\"last_block_sequence\":{},",
            "\"last_frame_count\":{},",
            "\"last_channel_count\":{},",
            "\"last_block_execution_time_ns\":{},",
            "\"last_block_deadline_budget_ns\":{},",
            "\"last_block_budget_utilization_percent\":{},",
            "\"last_block_budget_overrun_ns\":{},",
            "\"last_block_deadline_pressure\":\"{:?}\",",
            "\"budget_overrun_count\":{},",
            "\"prework_cache_state\":{},",
            "\"prework_cache_freshness_state\":{},",
            "\"last_prework_invalidation_reason\":{},",
            "\"total_latency_samples\":{},",
            "\"total_tail_samples\":{},",
            "\"output_tail_samples\":{},",
            "\"max_bus_tail_samples\":{},",
            "\"last_input_peak\":{},",
            "\"last_output_peak\":{},",
            "\"last_output_rms\":{},",
            "\"transport_epoch\":{},",
            "\"transport_transition\":{},",
            "\"transport_loop_wrapped\":{},",
            "\"context_anticipative\":{},",
            "\"transport_playing\":{},",
            "\"transport_tempo_bpm\":{},",
            "\"timeline_position_samples\":{}",
            "}}"
        ),
        summary.processed_blocks,
        json_option_u64(summary.last_processing_epoch),
        json_option_u64(summary.last_block_sequence),
        summary.last_frame_count,
        summary.last_channel_count,
        json_option_u64(summary.last_block_execution_time_ns),
        json_option_u64(summary.last_block_deadline_budget_ns),
        json_option_f32(summary.last_block_budget_utilization_percent),
        json_option_u64(summary.last_block_budget_overrun_ns),
        summary.last_block_deadline_pressure,
        summary.budget_overrun_count,
        json_escape_string(&format!("{:?}", summary.prework_cache_state)),
        json_escape_string(&format!("{:?}", summary.prework_cache_freshness_state)),
        json_option_string(
            summary
                .last_prework_invalidation_reason
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        summary.total_latency_samples,
        summary.total_tail_samples,
        summary.output_tail_samples,
        summary.max_bus_tail_samples,
        json_option_f32(summary.last_input_peak),
        json_option_f32(summary.last_output_peak),
        json_option_f32(summary.last_output_rms),
        summary.transport_epoch,
        json_option_string(
            summary
                .transport_transition
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        summary.transport_loop_wrapped,
        match summary.context_anticipative {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        match summary.transport_playing {
            Some(value) => value.to_string(),
            None => "null".into(),
        },
        json_option_f64(summary.transport_tempo_bpm),
        json_option_i64(summary.timeline_position_samples),
    )
}
