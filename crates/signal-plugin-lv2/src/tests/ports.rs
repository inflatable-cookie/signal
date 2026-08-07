use super::*;

/// Descriptor enrichment (g12.013): portProperty
/// toggled/integer/enumeration map to step counts, scale points count
/// enumeration steps, `units:unit` maps to a display label, and
/// `lv2:designation lv2:enabled` marks the bypass parameter.
#[test]
fn control_port_descriptors_map_steps_units_and_bypass() {
    let adapter = Lv2HostAdapter::default();
    let root = temp_plugin_root("descriptors");
    write_bundle(
        &root,
        "described.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2:   <http://lv2plug.in/ns/lv2core#> .\n\
             @prefix doap:  <http://usefulinc.com/ns/doap#> .\n\
             @prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .\n\
             @prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             @prefix units: <http://lv2plug.in/ns/extensions/units#> .\n\
             <https://example.com/described>\n\
             \ta lv2:Plugin ;\n\
             \tdoap:name \"Described\" ;\n\
             \tlv2:binary <described.so> ;\n\
             \tlv2:port [ a lv2:ControlPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"cutoff\" ;\n\
             \t    lv2:name \"Cutoff\" ; lv2:default 440.0 ; lv2:minimum 20.0 ; lv2:maximum 20000.0 ;\n\
             \t    units:unit units:hz ]\n\
             \t\t, [ a lv2:ControlPort , lv2:InputPort ; lv2:index 1 ; lv2:symbol \"stages\" ;\n\
             \t    lv2:name \"Stages\" ; lv2:default 2.0 ; lv2:minimum 1.0 ; lv2:maximum 5.0 ;\n\
             \t    lv2:portProperty lv2:integer ]\n\
             \t\t, [ a lv2:ControlPort , lv2:InputPort ; lv2:index 2 ; lv2:symbol \"mode\" ;\n\
             \t    lv2:name \"Mode\" ; lv2:default 0.0 ; lv2:minimum 0.0 ; lv2:maximum 2.0 ;\n\
             \t    lv2:portProperty lv2:enumeration ;\n\
             \t    lv2:scalePoint [ rdfs:label \"Low\" ; rdf:value 0.0 ]\n\
             \t\t, [ rdfs:label \"Mid\" ; rdf:value 1.0 ]\n\
             \t\t, [ rdfs:label \"High\" ; rdf:value 2.0 ] ]\n\
             \t\t, [ a lv2:ControlPort , lv2:InputPort ; lv2:index 3 ; lv2:symbol \"enabled\" ;\n\
             \t    lv2:name \"Enabled\" ; lv2:default 1.0 ; lv2:minimum 0.0 ; lv2:maximum 1.0 ;\n\
             \t    lv2:portProperty lv2:toggled ; lv2:designation lv2:enabled ] .\n",
        )],
    );

    let batch = scan(&adapter, &root);
    assert_eq!(batch.diagnostics, vec![]);
    assert_eq!(batch.discovered.len(), 1);
    let parameters = &batch.discovered[0].descriptor.parameters;
    assert_eq!(parameters.len(), 4);

    let cutoff = &parameters[0];
    assert_eq!(cutoff.unit.as_deref(), Some("Hz"));
    assert_eq!(cutoff.step_count, None);
    assert!(!cutoff.flags.stepped);
    assert!(cutoff.is_automatable());
    assert!(!cutoff.is_bypass());

    let stages = &parameters[1];
    assert_eq!(stages.step_count, Some(4), "integer span 1..5 = 4 steps");
    assert!(stages.flags.stepped);
    assert_eq!(stages.unit, None);

    let mode = &parameters[2];
    assert_eq!(mode.step_count, Some(2), "3 scale points = 2 steps");
    assert!(mode.flags.stepped);

    let enabled = &parameters[3];
    assert_eq!(enabled.step_count, Some(1), "toggled = one step");
    assert!(enabled.is_bypass());
    assert!((enabled.default_normalized - 1.0).abs() < 1e-6);

    let _ = fs::remove_dir_all(root);
}

/// Ports declared out of `lv2:index` order sort by index; control-port
/// defaults follow the documented rule (absent default → midpoint of
/// min/max, or 0.0 when unbounded).
#[test]
fn ports_sort_by_index_and_defaults_follow_the_documented_rule() {
    let adapter = Lv2HostAdapter::default();
    let root = temp_plugin_root("ports");
    write_bundle(
        &root,
        "shuffled.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             <https://example.com/shuffled>\n\
             \ta lv2:Plugin ;\n\
             \tlv2:binary <shuffled.so> ;\n\
             \tlv2:port [ a lv2:ControlPort , lv2:InputPort ; lv2:index 2 ;\n\
             \t           lv2:symbol \"midpointed\" ; lv2:minimum 10.0 ; lv2:maximum 30.0 ]\n\
             \t\t, [ a lv2:ControlPort , lv2:InputPort ; lv2:index 1 ;\n\
             \t\t    lv2:symbol \"unbounded\" ]\n\
             \t\t, [ a lv2:ControlPort , lv2:InputPort ; lv2:index 0 ;\n\
             \t\t    lv2:symbol \"explicit\" ; lv2:default 0.25 ;\n\
             \t\t    lv2:minimum 0.0 ; lv2:maximum 1.0 ] .\n",
        )],
    );

    let batch = scan(&adapter, &root);
    assert_eq!(batch.diagnostics, vec![]);
    assert_eq!(batch.discovered.len(), 1);
    let plugin = &batch.discovered[0];
    let indices: Vec<u32> = plugin.ports.iter().map(|port| port.index).collect();
    assert_eq!(indices, vec![0, 1, 2], "ports sorted by lv2:index");
    assert!((plugin.ports[0].effective_default() - 0.25).abs() < 1e-6);
    assert!(
        (plugin.ports[1].effective_default() - 0.0).abs() < 1e-6,
        "unbounded control port defaults to 0.0",
    );
    assert!(
        (plugin.ports[2].effective_default() - 20.0).abs() < 1e-6,
        "absent default falls back to the min/max midpoint",
    );
    let _ = fs::remove_dir_all(root);
}
