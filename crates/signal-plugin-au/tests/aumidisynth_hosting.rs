#![cfg(target_os = "macos")]

use std::path::Path;

use signal_plugin::{NoteEvent, NoteEventKind, PluginEvent};
use signal_plugin_au::AuHostedInstance;

#[test]
fn system_midi_synth_accepts_instrument_layout_and_generates_audio() {
    let mut instance = AuHostedInstance::load(Path::new("au-registry.component"), "aumu:msyn:appl")
        .expect("system MIDI synth should load");
    assert_eq!(instance.port_layout().main_input_channels, 0);
    assert_eq!(instance.port_layout().main_output_channels, 2);
    instance
        .activate(48_000.0, 1, 256)
        .expect("instrument layout should activate");
    let mut session = instance
        .process_session()
        .expect("instrument process session should not install an input callback");
    session.start().expect("processing should start");

    let mut audio = vec![0.0; 256 * 2];
    let event = PluginEvent::Note(NoteEvent {
        offset_frames: 7,
        note_id: -1,
        port_index: 0,
        channel: 0,
        key: 60,
        velocity: 0.75,
        kind: NoteEventKind::NoteOn,
    });
    assert!(session.process_in_place_with_events(&mut audio, 256, &[event]));
    assert!(audio[7 * 2..].iter().any(|sample| sample.abs() > 1e-5));

    session.stop();
    drop(session);
    instance.deactivate().expect("instrument should deactivate");
}
