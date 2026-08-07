use super::*;

/// An atom-input instrument is still DISCOVERED (with the Instrument
/// feature and its event input in the io layout) — the stereo gate
/// rejects it at hosting, not at scan.
#[test]
fn atom_input_instrument_is_discovered_but_flagged_as_instrument() {
    let adapter = Lv2HostAdapter::default();
    let root = temp_plugin_root("instrument");
    write_bundle(
        &root,
        "synth.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .\n\
             @prefix atom: <http://lv2plug.in/ns/ext/atom#> .\n\
             <https://example.com/synth>\n\
             \ta lv2:Plugin ;\n\
             \tlv2:binary <synth.so> ;\n\
             \tlv2:port [ a atom:AtomPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"events\" ]\n\
             \t\t, [ a lv2:AudioPort , lv2:OutputPort ; lv2:index 1 ; lv2:symbol \"out_l\" ]\n\
             \t\t, [ a lv2:AudioPort , lv2:OutputPort ; lv2:index 2 ; lv2:symbol \"out_r\" ] .\n",
        )],
    );

    let batch = scan(&adapter, &root);
    assert_eq!(batch.diagnostics, vec![]);
    assert_eq!(batch.discovered.len(), 1);
    let synth = &batch.discovered[0];
    assert_eq!(synth.default_io_layout.midi_inputs, 1);
    assert_eq!(synth.default_io_layout.audio_inputs, 0);
    assert_eq!(synth.default_io_layout.audio_outputs, 2);
    assert!(synth
        .descriptor
        .features
        .contains(&PluginFeature::Instrument));
    let _ = fs::remove_dir_all(root);
}
