mod broker;
mod child_gui;

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

    // g13.027 child thread posture: the MAIN thread is reserved for the
    // GUI service loop (macOS requires AppKit on the main thread; AppKit
    // initializes lazily on the first `open-editor`), the stdio protocol
    // moves to a dedicated CONTROL thread, and the RT audio loop stays on
    // its own thread spawned by `start-processing` — the audio path never
    // touches AppKit.
    let (gui_handle, gui_requests) = child_gui::channel();
    let writer = child_gui::SharedLineWriter::new(Box::new(io::stdout()));
    let control_writer = writer.clone();
    let control = std::thread::Builder::new()
        .name("sandbox-control".into())
        .spawn(move || {
            let mut broker = SandboxBrokerProcess::default();
            broker.set_gui_handle(gui_handle);
            let stdin = io::stdin();
            broker
                .serve(stdin.lock(), control_writer)
                .expect("sandbox broker serve");
        })
        .expect("sandbox control thread should spawn");

    // Runs until the control thread drops its GUI handle (serve returned).
    child_gui::run_gui_service(gui_requests, writer, "plugin-sandbox-broker");
    control.join().expect("sandbox control thread panicked");
}
