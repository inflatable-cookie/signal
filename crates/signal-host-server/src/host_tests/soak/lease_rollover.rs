mod event_accounting;
mod summary_assertions;

use super::*;
use event_accounting::assert_lease_rollover_event_accounting;
use summary_assertions::assert_lease_rollover_summary;

#[test]
fn server_host_soak_path_rolls_across_multiple_lease_generations() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = host.boot_with_watchdog_soak().expect("watchdog soak boot");
    let supervisor = host.supervisor_report();

    assert_lease_rollover_summary(&summary, &supervisor);
    assert_lease_rollover_event_accounting(&supervisor);
    assert_runtime_automation_values(
        &supervisor,
        RuntimeAutomationExpectations {
            value_events: 12,
            modulation_events: 12,
            gesture_begin_events: 2,
            gesture_end_events: 10,
            first_value: 0.2,
            last_value: 0.95,
            last_modulation: 0.26,
        },
    );
    assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
    assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
}
