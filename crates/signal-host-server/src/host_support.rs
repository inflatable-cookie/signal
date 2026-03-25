#[path = "host_support/faults.rs"]
mod faults;
#[path = "host_support/instance_state.rs"]
mod instance_state;
#[path = "host_support/lifecycle_admission.rs"]
mod lifecycle_admission;
#[path = "host_support/lifecycle_control.rs"]
mod lifecycle_control;
#[path = "host_support/metadata.rs"]
mod metadata;
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

pub(crate) use faults::{
    build_fault_envelope, extract_prepare_metadata, lifecycle_stage_for_request,
    record_broker_failure_and_convert, record_runtime_fault, runtime_error_from_failure,
    runtime_error_from_io, runtime_watchdog_trigger, transport_attach_intent,
};
pub(crate) use instance_state::plugin_instance_state_record_from_response;
pub(crate) use metadata::{
    runtime_au_discovered_type_record, runtime_host_clock_source, runtime_host_lifecycle_ownership,
    runtime_host_restart_policy, runtime_lv2_discovered_type_record,
    runtime_plugin_discovered_type_record, runtime_plugin_format_platform_coverage,
    runtime_vst3_discovered_type_record,
};
