#[path = "support/public_contract_boundary_media.rs"]
mod public_contract_boundary_media_support;
#[path = "support/public_contract_boundary_preview.rs"]
mod public_contract_boundary_preview_support;

use public_contract_boundary_preview_support::{
    assert_preview_transform_observation, assert_preview_transform_render_and_preview,
    assert_preview_transform_supervisor, cleanup_preview_transform_runtime,
    configured_preview_transform_runtime,
};
use signal_runtime::{RuntimeEventRecorder, RuntimeObservationReport, RuntimeSupervisorReport};

#[test]
fn public_runtime_preview_transform_boundary_reports_runtime_owned_preview_truth() {
    let (runtime, ready_path) = configured_preview_transform_runtime();

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_preview_transform_observation(&observation);

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    assert_preview_transform_render_and_preview(&runtime);
    assert_preview_transform_supervisor(&supervisor);
    cleanup_preview_transform_runtime(&runtime, &ready_path);
}
