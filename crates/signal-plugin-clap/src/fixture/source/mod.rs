mod extensions;
mod state;
mod statics;
mod types;

use extensions::extensions_fragment;
use state::state_fragment;
use statics::statics_fragment;
use types::types_fragment;

/// Full Rust source of the fixture cdylib.
pub fn clap_fixture_source(plugin_type_id: &str, plugin_name: &str, midi_outputs: u16) -> String {
    clap_fixture_source_for_layout(plugin_type_id, plugin_name, midi_outputs, false, 1)
}

pub(crate) fn clap_fixture_source_for_layout(
    plugin_type_id: &str,
    plugin_name: &str,
    midi_outputs: u16,
    instrument: bool,
    audio_output_count: u32,
) -> String {
    format!(
        "{}{}{}{}",
        types_fragment(instrument),
        statics_fragment(plugin_type_id, plugin_name, instrument, audio_output_count),
        state_fragment(),
        extensions_fragment(instrument, midi_outputs, audio_output_count),
    )
}
