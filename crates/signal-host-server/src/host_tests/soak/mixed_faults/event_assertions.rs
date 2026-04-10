#[path = "event_assertions/dispatch_accounting.rs"]
mod dispatch_accounting;
#[path = "event_assertions/lifecycle_faults.rs"]
mod lifecycle_faults;

use signal_runtime::RuntimeSupervisorReport;

use dispatch_accounting::assert_mixed_watchdog_dispatch_accounting;
use lifecycle_faults::assert_mixed_watchdog_lifecycle_faults;

pub(super) fn assert_mixed_watchdog_event_stream(supervisor: &RuntimeSupervisorReport) {
    assert_mixed_watchdog_lifecycle_faults(supervisor);
    assert_mixed_watchdog_dispatch_accounting(supervisor);
}
