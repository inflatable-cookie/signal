mod event_assertions;
mod summary_assertions;

use super::super::super::*;
use event_assertions::assert_mixed_watchdog_event_stream;
use summary_assertions::assert_mixed_watchdog_summary;

#[test]
fn local_host_mixed_watchdog_soak_tracks_deadlines_and_heartbeats() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host
        .boot_with_mixed_watchdog_soak()
        .expect("mixed watchdog soak boot");
    let supervisor = host.supervisor_report();

    assert_mixed_watchdog_summary(&host, &summary, &supervisor);
    assert_mixed_watchdog_event_stream(&summary, &supervisor);
}
