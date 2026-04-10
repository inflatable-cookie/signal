#[path = "soak_cycles/event_accounting.rs"]
mod event_accounting;
#[path = "soak_cycles/summary_assertions.rs"]
mod summary_assertions;

use super::super::super::*;
use event_accounting::assert_watchdog_soak_event_accounting;
use summary_assertions::assert_watchdog_soak_summary;

#[test]
fn local_host_soak_path_rolls_across_multiple_lease_generations() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host.boot_with_watchdog_soak().expect("watchdog soak boot");
    let supervisor = host.supervisor_report();

    assert_watchdog_soak_summary(&summary, &supervisor);
    assert_watchdog_soak_event_accounting(&supervisor);
    assert_runtime_automation_values(
        &supervisor,
        RuntimeAutomationExpectations {
            value_events: 12,
            modulation_events: 12,
            gesture_begin_events: 3,
            gesture_end_events: 9,
            first_value: 2.0 / 7.0,
            last_value: 5.0 / 7.0,
            last_modulation: 0.18,
        },
    );
    assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
    assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 13, 0, 2);
    assert_plugin_dispatch_summary(&summary, &supervisor, 0);
}
