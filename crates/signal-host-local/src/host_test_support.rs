#![allow(dead_code)]

#[path = "host_test_support/assertions.rs"]
mod assertions;
#[path = "host_test_support/setup.rs"]
mod setup;

pub(crate) use assertions::{
    assert_local_plugin_topology, assert_plugin_dispatch_summary,
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
    RuntimeAutomationExpectations,
};
pub(crate) use setup::{
    prepare_local_host_for_offline_render, prepare_local_host_with_lifecycle,
    prepare_local_host_without_lifecycle, temp_artifact_dir, temp_local_au_scan_root,
    temp_local_vst3_scan_root, unique_test_path, write_test_wav,
};
