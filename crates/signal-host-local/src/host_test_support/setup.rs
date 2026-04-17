#[path = "setup/fixtures.rs"]
mod fixtures;
#[path = "setup/lifecycle.rs"]
mod lifecycle;
#[path = "setup/offline_render.rs"]
mod offline_render;
#[path = "setup/scan_roots.rs"]
mod scan_roots;

pub(crate) use fixtures::{temp_artifact_dir, unique_test_path, write_test_wav};
pub(crate) use lifecycle::{
    prepare_local_host_with_lifecycle, prepare_local_host_without_lifecycle,
};
pub(crate) use offline_render::prepare_local_host_for_offline_render;
pub(crate) use scan_roots::{temp_local_au_scan_root, temp_local_vst3_scan_root};
