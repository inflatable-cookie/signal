use super::*;

/// Multi-plugin manifest: one manifest.ttl declares two plugins (one
/// with a full `<>` URI reference, one via a prefixed name) whose port
/// models live in separate rdfs:seeAlso files.
#[test]
fn multi_plugin_manifest_with_split_see_also_files_discovers_both() {
    let adapter = Lv2HostAdapter::default();
    let root = temp_plugin_root("multi");
    write_bundle(
        &root,
        "duo.lv2",
        &[
            (
                "manifest.ttl",
                "@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .\n\
                 @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                 @prefix ex:   <https://example.com/plugins/> .\n\
                 <https://example.com/plugins/alpha>\n\
                 \ta lv2:Plugin ;\n\
                 \tlv2:binary <duo.so> ;\n\
                 \trdfs:seeAlso <alpha.ttl> .\n\
                 ex:beta\n\
                 \ta lv2:Plugin ;\n\
                 \tlv2:binary <duo.so> ;\n\
                 \trdfs:seeAlso <beta.ttl> .\n",
            ),
            (
                "alpha.ttl",
                "@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .\n\
                 @prefix doap: <http://usefulinc.com/ns/doap#> .\n\
                 <https://example.com/plugins/alpha>\n\
                 \tdoap:name \"Alpha Gain\" ;\n\
                 \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"in_l\" ]\n\
                 \t\t, [ a lv2:AudioPort , lv2:InputPort ; lv2:index 1 ; lv2:symbol \"in_r\" ]\n\
                 \t\t, [ a lv2:AudioPort , lv2:OutputPort ; lv2:index 2 ; lv2:symbol \"out_l\" ]\n\
                 \t\t, [ a lv2:AudioPort , lv2:OutputPort ; lv2:index 3 ; lv2:symbol \"out_r\" ]\n\
                 \t\t, [ a lv2:ControlPort , lv2:InputPort ; lv2:index 4 ; lv2:symbol \"gain\" ;\n\
                 \t\t    lv2:name \"Gain\" ; lv2:default 0.5 ; lv2:minimum 0.0 ; lv2:maximum 1.0 ] .\n",
            ),
            (
                "beta.ttl",
                "@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .\n\
                 @prefix doap: <http://usefulinc.com/ns/doap#> .\n\
                 <https://example.com/plugins/beta>\n\
                 \tdoap:name \"Beta Meter\" ;\n\
                 \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"in\" ]\n\
                 \t\t, [ a lv2:ControlPort , lv2:OutputPort ; lv2:index 1 ; lv2:symbol \"level\" ] .\n",
            ),
        ],
    );

    let batch = scan(&adapter, &root);
    assert_eq!(batch.diagnostics, vec![]);
    assert_eq!(batch.discovered.len(), 2);

    let alpha = batch
        .discovered
        .iter()
        .find(|plugin| plugin.plugin_uri == "https://example.com/plugins/alpha")
        .expect("alpha discovered");
    assert_eq!(
        alpha.plugin_type_id.0,
        "plugin:lv2:https://example.com/plugins/alpha",
    );
    assert_eq!(alpha.descriptor.name, "Alpha Gain");
    assert_eq!(alpha.descriptor.format, PluginFormat::Lv2);
    assert_eq!(alpha.default_io_layout.audio_inputs, 2);
    assert_eq!(alpha.default_io_layout.audio_outputs, 2);
    assert!(alpha.binary_path.ends_with("duo.so"));
    assert_eq!(alpha.descriptor.parameters.len(), 1);
    let gain = &alpha.descriptor.parameters[0];
    assert_eq!(gain.parameter_id, 4, "parameter_id = control port index");
    assert_eq!(gain.name, "Gain");
    assert!((gain.min_plain - 0.0).abs() < 1e-6);
    assert!((gain.max_plain - 1.0).abs() < 1e-6);
    assert!((gain.default_normalized - 0.5).abs() < 1e-6);

    let beta = batch
        .discovered
        .iter()
        .find(|plugin| plugin.plugin_uri == "https://example.com/plugins/beta")
        .expect("beta discovered via prefixed-name subject");
    assert_eq!(beta.descriptor.name, "Beta Meter");
    // Control OUTPUT ports are not parameters.
    assert!(beta.descriptor.parameters.is_empty());

    let _ = fs::remove_dir_all(root);
}
