use std::path::PathBuf;

use signal_plugin_clap::{fixture::compile_clap_instrument_fixture, ClapPluginHostAdapter};

fn main() -> Result<(), String> {
    let output_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join("Library/Audio/Plug-Ins/CLAP"))
        })
        .ok_or_else(|| "output directory required when HOME is unavailable".to_string())?;
    let installed = compile_clap_instrument_fixture(
        &output_dir,
        "audio.infiniteloop.loophole.controlled-instrument",
        "Loophole Controlled Instrument",
    )?;
    let discovered = ClapPluginHostAdapter::default()
        .discover_plugins_for_roots_with_options(&[installed.display().to_string()], true);
    let plugin = discovered
        .iter()
        .find(|plugin| {
            plugin.plugin_type_id.0 == "audio.infiniteloop.loophole.controlled-instrument"
        })
        .ok_or_else(|| "installed fixture was not discoverable".to_string())?;
    println!("{}", installed.display());
    println!(
        "io={}x{} midi={}x{}",
        plugin.default_io_layout.audio_inputs,
        plugin.default_io_layout.audio_outputs,
        plugin.default_io_layout.midi_inputs,
        plugin.default_io_layout.midi_outputs,
    );
    Ok(())
}
