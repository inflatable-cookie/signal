use signal_host_local::LocalRuntimeHost;
use signal_runtime::{RuntimeConfig, SignalRuntime};

fn main() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host.boot_default().expect("local host boot");
    let report = host.host_supervisor_report();
    let topology = &report.observation.observation.execution_topology_summary;

    println!(
        "signal-host-local profile={:?} backend={} clap_supported={} sandbox={:?} control_requests={} control_responses={} heartbeat_responses={} processed_blocks={} completion={:?} last_block_sequence={} interaction_mode={} automation_value={:?} parameter_events={} generated_event_bytes={} deadline_misses={} heartbeat_misses={} watchdog_triggered={} watchdog_reason={:?} last_control_message={:?} epoch={} lease_id={:?} region_id={:?} shared_memory_bytes={} restarts={} teardowns={} audio_state={:?} audio_callbacks={} audio_frames={} audio_copied_samples={} audio_zero_filled_samples={} audio_dropped_samples={} audio_peak={:?} audio_graph={:?} topology_nodes={} topology_roles={}/{}/{} topology_groups={}/{}/{} observation={}",
        host.runtime().config().profile,
        summary.backend_name,
        host.clap_supported(),
        summary.transport.sandbox_id,
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.processed_blocks,
        summary.execution.last_completion_state,
        summary.execution.last_block_sequence,
        if std::env::var_os("SIGNAL_HOST_DEMO_INTERACTION_MODE").is_some() {
            "parameter-step"
        } else {
            "none"
        },
        summary
            .plugin_dispatch
            .as_ref()
            .and_then(|dispatch| dispatch.automation_value),
        summary.last_payload.parameter_event_count,
        summary.last_payload.generated_event_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        summary.faults.watchdog_trigger_reason,
        summary.execution.last_control_message,
        summary.execution.processing_epoch,
        summary.transport.shared_memory_lease_id,
        summary.transport.shared_memory_region_id,
        summary.transport.shared_memory_bytes,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        summary.audio_pump.stream_state,
        summary.audio_pump.callback_count,
        summary.audio_pump.total_callback_frames,
        summary.audio_pump.copied_output_samples,
        summary.audio_pump.zero_filled_output_samples,
        summary.audio_pump.dropped_output_samples,
        summary.audio_pump.last_callback_output_peak,
        summary.audio_pump.last_runtime_graph_id,
        topology.node_count,
        topology.track_lane_node_count,
        topology.bus_node_count,
        topology.console_node_count,
        topology.track_lane_group_count,
        topology.bus_group_count,
        topology.console_group_count,
        report.render_compact()
    );
}
