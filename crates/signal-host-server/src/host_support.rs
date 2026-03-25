#[path = "host_support/faults.rs"]
mod faults;
#[path = "host_support/instance_state.rs"]
mod instance_state;
#[path = "host_support/metadata.rs"]
mod metadata;

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
