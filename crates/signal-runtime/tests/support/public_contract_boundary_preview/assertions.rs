use std::path::PathBuf;

#[path = "assertions/observation.rs"]
mod observation;
#[path = "assertions/rendering.rs"]
mod rendering;

use signal_runtime::{RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime};

pub(crate) use observation::assert_preview_transform_observation;
pub(crate) use rendering::{
    assert_preview_transform_render_and_preview, assert_preview_transform_supervisor,
    cleanup_preview_transform_runtime,
};

#[allow(dead_code)]
fn _type_anchor(
    _observation: &RuntimeObservationReport,
    _supervisor: &RuntimeSupervisorReport,
    _runtime: &SignalRuntime,
    _path: &PathBuf,
) {
}
