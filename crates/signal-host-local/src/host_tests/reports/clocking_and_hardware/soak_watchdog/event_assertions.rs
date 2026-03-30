mod dispatch_accounting;
mod lifecycle_faults;

use super::super::super::super::*;

pub(super) fn assert_mixed_watchdog_event_stream(
    summary: &LocalRuntimeHostSummary,
    supervisor: &RuntimeHostSupervisorReport,
) {
    lifecycle_faults::assert_mixed_watchdog_lifecycle_faults(summary, supervisor);
    dispatch_accounting::assert_mixed_watchdog_dispatch_accounting(summary, supervisor);
}
