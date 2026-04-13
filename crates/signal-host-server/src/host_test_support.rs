#![allow(dead_code)]

#[path = "host_test_support/assertions.rs"]
mod assertions;
#[path = "host_test_support/setup.rs"]
mod setup;

pub(crate) use assertions::{
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
    RuntimeAutomationExpectations,
};
pub(crate) use setup::{
    prepare_server_host_with_lifecycle, prepare_server_host_without_lifecycle,
    temp_media_fixture_path, temp_server_au_scan_root, temp_server_lv2_scan_root,
    temp_server_vst3_scan_root,
};
