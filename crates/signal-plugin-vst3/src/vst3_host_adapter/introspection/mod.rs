//! VST3 bundle introspection: moduleinfo, factory class snapshots, scan helper.

mod derive;
mod factory;
mod paths;
mod scan_helper;
mod snapshot;
mod types;

#[cfg(target_os = "macos")]
mod macos_bundle;

#[cfg(not(target_os = "macos"))]
pub(crate) use paths::resolve_module_binary_path;
pub(crate) use snapshot::{metadata_descriptor, metadata_io_layout, read_vst3_bundle_snapshot};

pub(super) use scan_helper::run_vst3_scan_helper;
pub(super) use snapshot::moduleinfo_declares_component_class;
