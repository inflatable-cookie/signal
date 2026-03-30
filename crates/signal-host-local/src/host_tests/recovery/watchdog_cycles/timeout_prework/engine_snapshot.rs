mod prework_cache;
mod topology_dispatch;

use super::super::super::*;
use prework_cache::assert_timeout_prework_cache;
use topology_dispatch::assert_timeout_topology_dispatch;

pub(super) fn assert_timeout_engine_snapshot(
    summary: &LocalRuntimeHostSummary,
    supervisor: &signal_runtime::RuntimeSupervisorReport,
) {
    assert_timeout_topology_dispatch(supervisor);
    assert_timeout_prework_cache(summary, supervisor);
}
