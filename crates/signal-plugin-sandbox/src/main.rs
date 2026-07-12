mod broker;

use std::io;

use broker::SandboxBrokerProcess;

#[derive(serde::Serialize)]
struct ClapProbeReceipt {
    plugin_type_id: String,
    audio_inputs: u16,
    audio_outputs: u16,
    midi_inputs: u16,
    midi_outputs: u16,
}

fn main() {
    let mut args = std::env::args_os();
    let _program = args.next();
    if args.next().as_deref() == Some(std::ffi::OsStr::new("probe-clap")) {
        let path = args.next().expect("probe-clap requires a plugin path");
        let path = std::path::PathBuf::from(path);
        let plugins = signal_plugin_clap::ClapPluginHostAdapter::default()
            .discover_plugins_for_roots_with_options(&[path.display().to_string()], true);
        let receipts = plugins
            .into_iter()
            .map(|plugin| ClapProbeReceipt {
                plugin_type_id: plugin.plugin_type_id.0,
                audio_inputs: plugin.default_io_layout.audio_inputs,
                audio_outputs: plugin.default_io_layout.audio_outputs,
                midi_inputs: plugin.default_io_layout.midi_inputs,
                midi_outputs: plugin.default_io_layout.midi_outputs,
            })
            .collect::<Vec<_>>();
        serde_json::to_writer(io::stdout().lock(), &receipts).expect("serialize probe receipts");
        return;
    }
    let stdin = io::stdin();
    let mut broker = SandboxBrokerProcess::default();
    broker
        .serve(stdin.lock(), io::stdout().lock())
        .expect("sandbox broker serve");
}
