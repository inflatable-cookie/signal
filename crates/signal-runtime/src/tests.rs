// Tests for signal-runtime
use crate::interfaces::{RuntimeEvent, RuntimeEventSink};

#[derive(Default)]
struct TestSink {
    events: Vec<RuntimeEvent>,
}

impl RuntimeEventSink for TestSink {
    fn push(&mut self, event: RuntimeEvent) {
        self.events.push(event);
    }
}

#[path = "tests/fixtures.rs"]
mod fixtures;
#[path = "tests/support.rs"]
mod support;
use support::*;
#[path = "tests/clip_processing.rs"]
mod clip_processing;
#[path = "tests/core_runtime.rs"]
mod core_runtime;
#[path = "tests/discovery_parity.rs"]
mod discovery_parity;
#[path = "tests/engine_execution.rs"]
mod engine_execution;
#[path = "tests/event_lifecycle.rs"]
mod event_lifecycle;
#[path = "tests/forecast_override_lifecycle.rs"]
mod forecast_override_lifecycle;
#[path = "tests/forecast_profile_queueing.rs"]
mod forecast_profile_queueing;
#[path = "tests/forecast_profile_rebuilds.rs"]
mod forecast_profile_rebuilds;
#[path = "tests/forecast_profile_selection.rs"]
mod forecast_profile_selection;
#[path = "tests/forecast_windows.rs"]
mod forecast_windows;
#[path = "tests/graph_projection.rs"]
mod graph_projection;
#[path = "tests/lifecycle_guards.rs"]
mod lifecycle_guards;
#[path = "tests/media_service.rs"]
mod media_service;
#[path = "tests/metering_automation.rs"]
mod metering_automation;
#[path = "tests/observation_transform_receipts.rs"]
mod observation_transform_receipts;
#[path = "tests/performance_receipts.rs"]
mod performance_receipts;
#[path = "tests/plugin_binding.rs"]
mod plugin_binding;
#[path = "tests/plugin_chain_receipts.rs"]
mod plugin_chain_receipts;
#[path = "tests/plugin_chain_recovery.rs"]
mod plugin_chain_recovery;
#[path = "tests/plugin_placement.rs"]
mod plugin_placement;
#[path = "tests/pressure_policies.rs"]
mod pressure_policies;
#[path = "tests/preview_transform_reports.rs"]
mod preview_transform_reports;
#[path = "tests/prework_cache_invalidation.rs"]
mod prework_cache_invalidation;
#[path = "tests/prework_queue.rs"]
mod prework_queue;
#[path = "tests/realtime_prework_service.rs"]
mod realtime_prework_service;
#[path = "tests/realtime_scheduler_recovery.rs"]
mod realtime_scheduler_recovery;
#[path = "tests/recall_handoff.rs"]
mod recall_handoff;
#[path = "tests/recording_capture.rs"]
mod recording_capture;
#[path = "tests/routing_receipts.rs"]
mod routing_receipts;
#[path = "tests/scheduler_state.rs"]
mod scheduler_state;
#[path = "tests/scheduler_topology.rs"]
mod scheduler_topology;
#[path = "tests/transport_sessions.rs"]
mod transport_sessions;
#[path = "tests/transport_state.rs"]
mod transport_state;
#[path = "tests/watchdog_faults.rs"]
mod watchdog_faults;
