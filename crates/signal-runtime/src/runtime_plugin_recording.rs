pub(super) use super::*;

#[path = "runtime_plugin_recording/capability_coverage.rs"]
mod capability_coverage;
#[path = "runtime_plugin_recording/coverage.rs"]
mod coverage;
#[path = "runtime_plugin_recording/discovery.rs"]
mod discovery;
#[path = "runtime_plugin_recording/format_coverage.rs"]
mod format_coverage;
#[path = "runtime_plugin_recording/metadata.rs"]
mod metadata;
#[path = "runtime_plugin_recording/parity_coverage.rs"]
mod parity_coverage;
#[path = "runtime_plugin_recording/transport.rs"]
mod transport;

pub(crate) use capability_coverage::runtime_plugin_capability_coverage;
pub(crate) use coverage::plugin_format_sort_key;
pub(crate) use format_coverage::runtime_plugin_format_coverage;
pub(crate) use parity_coverage::runtime_plugin_parity_coverage;
