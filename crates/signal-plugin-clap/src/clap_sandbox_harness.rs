use signal_ipc::PluginMessageEnvelope;

mod block_read;
mod block_read_payload;
mod block_surface;
mod block_write;
mod descriptor;
mod failure;
mod lifecycle_dispatch;
mod lifecycle_setup;
mod lifecycle_setup_prepare;
mod lifecycle_teardown;
mod state;

pub use failure::{
    classify_sandbox_failure, sandbox_failure_event, ClapSandboxFailureClassification,
    ClapSandboxFailureInput, ClapSandboxFailureStage,
};
pub use state::ClapSandboxLifecycleHarness;

pub(crate) use descriptor::{
    clap_discovered_plugin_type, clap_fixture_descriptor, descriptor_payload,
};

pub type ClapHarnessError = Box<PluginMessageEnvelope>;
pub type ClapHarnessResult<T> = Result<T, ClapHarnessError>;
