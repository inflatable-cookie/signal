mod dispatch_accounting;
mod lifecycle_faults;

use signal_runtime::RuntimeHostSupervisorReport;

use dispatch_accounting::assert_mixed_watchdog_dispatch_accounting;
use lifecycle_faults::assert_mixed_watchdog_lifecycle_faults;

pub(super) fn assert_mixed_watchdog_event_stream(supervisor: &RuntimeHostSupervisorReport) {
    assert_mixed_watchdog_lifecycle_faults(supervisor);
    assert_mixed_watchdog_dispatch_accounting(supervisor);
}
