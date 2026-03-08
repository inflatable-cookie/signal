use signal_host_local::LocalRuntimeHost;
use signal_runtime::{RuntimeConfig, SignalRuntime};

fn main() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host.boot_default().expect("local host boot");
    let supervisor = host.supervisor_report();

    println!(
        "signal-host-local profile={:?} backend={} clap_supported={} sandbox={:?} control_requests={} control_responses={} heartbeat_responses={} processed_blocks={} completion={:?} last_block_sequence={} deadline_misses={} heartbeat_misses={} watchdog_triggered={} watchdog_reason={:?} last_control_message={:?} epoch={} lease_id={:?} region_id={:?} shared_memory_bytes={} restarts={} teardowns={} observation={}",
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
