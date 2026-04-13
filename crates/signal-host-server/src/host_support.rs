#[path = "host_support/boot_entrypoints.rs"]
mod boot_entrypoints;
#[path = "host_support/boot_recovery.rs"]
mod boot_recovery;
#[path = "host_support/boot_recovery_helpers.rs"]
mod boot_recovery_helpers;
#[path = "host_support/boot_summary.rs"]
mod boot_summary;
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
#[path = "host_support/runtime_cycle.rs"]
mod runtime_cycle;
#[path = "host_support/sandbox_sessions.rs"]
mod sandbox_sessions;
#[path = "host_support/summary_types.rs"]
mod summary_types;

pub use demo::ensure_default_demo_plugin_override;
pub(crate) use demo::{
    demo_interaction_step, server_demo_runtime_assembly, ServerDemoPluginSandboxAssembly,
};
pub(crate) use discovery::discovered_plugins_for_scan;
pub(crate) use faults::{
    build_fault_envelope, extract_prepare_metadata, lifecycle_stage_for_request,
    record_broker_failure_and_convert, record_runtime_fault, runtime_error_from_failure,
    runtime_error_from_io, runtime_watchdog_trigger, transport_attach_intent,
};
pub(crate) use host_types::{
    samples_to_ms, FaultInjection, RecoveryFailureInjection, ServerSupervisorState,
    INTER_EPISODE_CONTINUITY_BLOCKS, SOAK_RESTART_EPISODES, STEADY_STATE_BLOCKS,
    WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};
pub(crate) use instance_state::plugin_instance_state_record_from_response;
pub(crate) use lifecycle_admission::LifecycleAdmissionRollback;
pub(crate) use lifecycle_run::{LifecycleRunSummary, RecoveryHistory};
pub(crate) use metadata::{
    runtime_host_clock_source, runtime_host_lifecycle_ownership, runtime_host_restart_policy,
    runtime_plugin_format_platform_coverage,
};
pub(crate) use signal_runtime::RepeatedWatchdogRecoveryPlan;
pub(crate) type TimeoutRecoveryRetryPlan<'a> =
    signal_runtime::TimeoutRecoveryRetryPlan<'a, RecoveryFailureInjection>;
pub(crate) use recovery_overlap_finish::RecoveryOverlapTransition;
pub(crate) use recovery_runtime::LingeringSessionRecovery;
pub(crate) use sandbox_sessions::{
    ensure_au_sandbox_session, ensure_clap_sandbox_session, ensure_lv2_sandbox_session,
    ensure_vst3_sandbox_session, teardown_broker_sandbox_session, SandboxBrokerSession,
};
pub use summary_types::{
    ServerExecutionSummary, ServerFaultSummary, ServerPayloadSummary, ServerPluginDispatchSummary,
    ServerRuntimeHostSummary, ServerTransportSummary,
};
