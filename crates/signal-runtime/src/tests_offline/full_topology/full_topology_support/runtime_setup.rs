use super::*;

mod discovery;
mod projection_setup;

pub(super) fn prepare_runtime(runtime: &mut SignalRuntime) {
    handshake_and_configure_with_disabled_forecast(runtime, true);
    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/VST3".into()],
        formats: vec![PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(scan_handle, discovery::discovered_types());
    projection_setup::apply(runtime);
}
