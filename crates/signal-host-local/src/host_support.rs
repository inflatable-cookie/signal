#[path = "host_support/audio_pump.rs"]
mod audio_pump;
#[path = "host_support/boot_entrypoints.rs"]
mod boot_entrypoints;
#[path = "host_support/boot_recovery.rs"]
mod boot_recovery;
#[path = "host_support/boot_recovery_helpers.rs"]
mod boot_recovery_helpers;
#[path = "host_support/boot_summary.rs"]
mod boot_summary;
#[path = "host_support/broker.rs"]
mod broker;
#[path = "host_support/clocking.rs"]
mod clocking;
#[path = "host_support/demo.rs"]
mod demo;
#[path = "host_support/demo_graph.rs"]
mod demo_graph;
#[path = "host_support/discovery.rs"]
mod discovery;
#[path = "host_support/faults.rs"]
mod faults;
#[path = "host_support/host_types.rs"]
mod host_types;
#[path = "host_support/instance_state.rs"]
mod instance_state;
#[path = "host_support/lifecycle_admission.rs"]
mod lifecycle_admission;
#[path = "host_support/lifecycle_control.rs"]
mod lifecycle_control;
#[path = "host_support/lifecycle_run.rs"]
mod lifecycle_run;
#[path = "host_support/metadata.rs"]
mod metadata;
#[path = "host_support/observation.rs"]
mod observation;
#[path = "host_support/observation_clock_transition.rs"]
mod observation_clock_transition;
#[path = "host_support/observation_host_io.rs"]
mod observation_host_io;
#[path = "host_support/offline_render.rs"]
mod offline_render;
#[path = "host_support/recovery_cleanup.rs"]
mod recovery_cleanup;
#[path = "host_support/recovery_cleanup_transport.rs"]
mod recovery_cleanup_transport;
#[path = "host_support/recovery_overlap_finish.rs"]
mod recovery_overlap_finish;
#[path = "host_support/recovery_overlap_prepare.rs"]
mod recovery_overlap_prepare;
#[path = "host_support/recovery_overlap_restart.rs"]
mod recovery_overlap_restart;
#[path = "host_support/recovery_runtime.rs"]
mod recovery_runtime;
#[path = "host_support/recovery_sandbox.rs"]
mod recovery_sandbox;
#[path = "host_support/recovery_teardown.rs"]
mod recovery_teardown;
#[path = "host_support/runtime_block.rs"]
mod runtime_block;
#[path = "host_support/runtime_block_render.rs"]
mod runtime_block_render;
#[path = "host_support/runtime_cycle.rs"]
mod runtime_cycle;
#[path = "host_support/sandbox_sessions.rs"]
mod sandbox_sessions;
#[path = "host_support/summary_types.rs"]
mod summary_types;
#[path = "host_support/transfer.rs"]
mod transfer;

pub(crate) use audio_pump::LocalAudioPumpState;
pub(crate) use boot_recovery_helpers::{RepeatedWatchdogRecoveryPlan, TimeoutRecoveryRetryPlan};
pub(crate) use broker::{record_broker_failure_and_convert, runtime_error_from_io};
pub(crate) use clocking::{
    host_clock_discontinuity_state, host_clock_domain, host_clock_drift_state,
    host_clock_fallback_state, host_duplex_mismatch_state, host_endpoint_topology,
    host_partial_availability, samples_to_ms,
};
pub(crate) use demo::{
    local_demo_runtime_assembly, payload_automation_value,
    plugin_automation_value_from_runtime_batch, runtime_watchdog_trigger, transport_attach_intent,
};
pub(crate) use discovery::discovered_plugins_for_scan;
pub(crate) use faults::{
    build_fault_envelope, extract_prepare_metadata, lifecycle_stage_for_request,
    record_runtime_fault, runtime_error_from_failure,
};
pub(crate) use host_types::{
    FaultInjection, LocalClockTransitionMemory, LocalSupervisorState, RecoveryFailureInjection,
    INTER_EPISODE_CONTINUITY_BLOCKS, LOCAL_DEMO_GRAPH_ID, LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES,
    LOCAL_DEMO_PLUGIN_NODE_ID, LOCAL_DEMO_PLUGIN_TAIL_SAMPLES, SOAK_RESTART_EPISODES,
    STEADY_STATE_BLOCKS, WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};
pub(crate) use instance_state::plugin_instance_state_record_from_response;
pub(crate) use lifecycle_admission::LifecycleAdmissionRollback;
pub(crate) use lifecycle_run::{LifecycleRunSummary, RecoveryHistory};
pub(crate) use metadata::{
    runtime_au_discovered_type_record, runtime_plugin_discovered_type_record,
    runtime_plugin_format_platform_coverage, runtime_vst3_discovered_type_record,
};
pub(crate) use recovery_overlap_finish::RecoveryOverlapTransition;
pub(crate) use recovery_runtime::LingeringSessionRecovery;
pub(crate) use sandbox_sessions::{ensure_au_sandbox_session, ensure_vst3_sandbox_session};
pub use summary_types::{
    LocalAudioPumpSummary, LocalAudioStreamState, LocalAudioTransferPolicy, LocalExecutionSummary,
    LocalFaultSummary, LocalHardwareSummary, LocalPayloadSummary, LocalPluginDispatchSummary,
    LocalRuntimeHostSummary, LocalTransportSummary,
};
pub(crate) use transfer::transfer_runtime_output_to_host_buffer;
#[cfg(test)]
pub(crate) use transfer::LocalAudioTransferOutcome;
