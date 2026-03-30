mod continuity;
mod lifecycle;

use super::super::super::super::*;

pub(super) fn assert_timeout_prework_cache(
    summary: &LocalRuntimeHostSummary,
    supervisor: &signal_runtime::RuntimeSupervisorReport,
) {
    lifecycle::assert_timeout_prework_cache_lifecycle(summary, supervisor);
    continuity::assert_timeout_prework_cache_continuity(summary, supervisor);
}
