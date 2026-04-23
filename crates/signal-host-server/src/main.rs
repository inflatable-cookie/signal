#![allow(missing_docs)]
use signal_host_server::ServerRuntimeHost;
use signal_runtime::{RuntimeConfig, SignalRuntime};

fn main() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = host.boot_default().expect("server host boot");
    let supervisor = host.supervisor_report();

    println!(
        "signal-host-server profile={:?} sandbox={:?} control_requests={} control_responses={} heartbeat_responses={} processed_blocks={} engine_processed_blocks={} engine_graph_id={:?} engine_output_peak={:?} engine_output_rms={:?} completion={:?} last_block_sequence={} interaction_mode={} automation_value={:?} parameter_events={} generated_event_bytes={} deadline_misses={} heartbeat_misses={} watchdog_triggered={} watchdog_reason={:?} last_control_message={:?} epoch={} lease_id={:?} region_id={:?} shared_memory_bytes={} restarts={} teardowns={} observation={}",
        host.runtime().config().profile,
        summary.transport.sandbox_id,
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.processed_blocks,
        summary.execution.engine_processed_blocks,
        summary.execution.last_engine_graph_id,
        summary.execution.last_engine_output_peak,
        summary.execution.last_engine_output_rms,
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
        supervisor.render_compact()
    );
}
