#[path = "event_assertions/dispatch_accounting.rs"]
mod dispatch_accounting;
#[path = "event_assertions/lifecycle_faults.rs"]
mod lifecycle_faults;

use crate::LocalRuntimeHostSummary;
use signal_runtime::RuntimeSupervisorReport;

pub(super) fn assert_mixed_watchdog_event_stream(
    summary: &LocalRuntimeHostSummary,
    supervisor: &RuntimeSupervisorReport,
) {
    lifecycle_faults::assert_mixed_watchdog_lifecycle_faults(summary, supervisor);
    dispatch_accounting::assert_mixed_watchdog_dispatch_accounting(summary, supervisor);
}
