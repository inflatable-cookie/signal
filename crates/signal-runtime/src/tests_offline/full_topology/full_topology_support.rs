use super::*;

mod request;
mod runtime_setup;

pub(super) fn prepare_full_topology_preview_runtime() -> (SignalRuntime, RuntimeOfflineRenderRequest)
{
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    runtime_setup::prepare_runtime(&mut runtime);
    let request = request::build_request(&runtime);
    (runtime, request)
}
