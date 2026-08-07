use super::*;

/// The scan pre-filter allowlist is urid#map ONLY: any other required
/// feature yields a typed UnsupportedRequiredFeature diagnostic, while
/// urid:map itself (and any optionalFeature) passes.
#[test]
fn required_features_beyond_urid_map_are_rejected_at_scan() {
    let adapter = Lv2HostAdapter::default();
    let root = temp_plugin_root("features");
    write_bundle(
        &root,
        "workered.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             <https://example.com/workered>\n\
             \ta lv2:Plugin ;\n\
             \tlv2:binary <workered.so> ;\n\
             \tlv2:requiredFeature <http://lv2plug.in/ns/ext/urid#map> ,\n\
             \t\t<http://lv2plug.in/ns/ext/worker#schedule> ;\n\
             \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"in\" ] .\n",
        )],
    );
    write_bundle(
        &root,
        "urid-only.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             <https://example.com/urid-only>\n\
             \ta lv2:Plugin ;\n\
             \tlv2:binary <urid-only.so> ;\n\
             \tlv2:requiredFeature <http://lv2plug.in/ns/ext/urid#map> ;\n\
             \tlv2:optionalFeature <http://lv2plug.in/ns/ext/worker#schedule> ;\n\
             \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"in\" ] .\n",
        )],
    );

    let batch = scan(&adapter, &root);
    assert_eq!(batch.discovered.len(), 1);
    assert_eq!(
        batch.discovered[0].plugin_uri,
        "https://example.com/urid-only",
    );
    assert_eq!(batch.diagnostics.len(), 1);
    let diagnostic = &batch.diagnostics[0];
    assert_eq!(
        diagnostic.kind,
        Lv2DiscoveryDiagnosticKind::UnsupportedRequiredFeature,
    );
    assert_eq!(
        diagnostic.plugin_type_id.as_deref(),
        Some("plugin:lv2:https://example.com/workered"),
    );
    assert!(diagnostic
        .detail
        .contains("http://lv2plug.in/ns/ext/worker#schedule"));
    assert!(!diagnostic.detail.contains("urid#map"));
    let _ = fs::remove_dir_all(root);
}

/// Malformed inputs produce MalformedManifest diagnostics — never a
/// panic, never a silent misparse. Per-plugin failures in a
/// multi-plugin bundle leave the healthy sibling discoverable.
#[test]
fn malformed_manifests_yield_diagnostics_not_panics() {
    let adapter = Lv2HostAdapter::default();
    let root = temp_plugin_root("malformed");
    // Bundle-level: unparseable manifest (missing '.').
    write_bundle(
        &root,
        "syntax.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             <https://example.com/broken> a lv2:Plugin\n",
        )],
    );
    // Bundle-level: exotic syntax outside the subset (collection).
    write_bundle(
        &root,
        "exotic.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             <https://example.com/exotic> a lv2:Plugin ;\n\
             \tlv2:binary <exotic.so> ;\n\
             \tlv2:port ( 1 2 ) .\n",
        )],
    );
    // Per-plugin: no lv2:binary.
    write_bundle(
        &root,
        "binaryless.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             <https://example.com/binaryless> a lv2:Plugin .\n",
        )],
    );
    // Per-plugin: port without lv2:index, and a duplicate-index pair.
    write_bundle(
        &root,
        "portless.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
             <https://example.com/no-index> a lv2:Plugin ;\n\
             \tlv2:binary <a.so> ;\n\
             \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:symbol \"in\" ] .\n\
             <https://example.com/dup-index> a lv2:Plugin ;\n\
             \tlv2:binary <a.so> ;\n\
             \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:index 0 ]\n\
             \t\t, [ a lv2:AudioPort , lv2:OutputPort ; lv2:index 0 ] .\n",
        )],
    );
    // Multi-plugin: one plugin's seeAlso file is broken, its sibling
    // stays discoverable.
    write_bundle(
        &root,
        "sibling.lv2",
        &[
            (
                "manifest.ttl",
                "@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .\n\
                 @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                 <https://example.com/healthy> a lv2:Plugin ;\n\
                 \tlv2:binary <s.so> ;\n\
                 \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"in\" ] .\n\
                 <https://example.com/sick> a lv2:Plugin ;\n\
                 \tlv2:binary <s.so> ;\n\
                 \trdfs:seeAlso <sick.ttl> .\n",
            ),
            ("sick.ttl", "this is not turtle at all {"),
        ],
    );

    let batch = scan(&adapter, &root);
    assert_eq!(
        batch.discovered.len(),
        1,
        "only the healthy sibling survives: {:?}",
        batch.discovered,
    );
    assert_eq!(
        batch.discovered[0].plugin_uri,
        "https://example.com/healthy"
    );
    assert_eq!(batch.diagnostics.len(), 6, "{:?}", batch.diagnostics);
    assert!(batch
        .diagnostics
        .iter()
        .all(|d| d.kind == Lv2DiscoveryDiagnosticKind::MalformedManifest));
    assert!(batch.diagnostics.iter().any(|d| {
        d.bundle_root.ends_with("sibling.lv2")
            && d.plugin_type_id.as_deref() == Some("plugin:lv2:https://example.com/sick")
    }));
    assert!(batch
        .diagnostics
        .iter()
        .any(|d| d.plugin_type_id.as_deref() == Some("plugin:lv2:https://example.com/binaryless")));
    assert!(batch.diagnostics.iter().any(|d| {
        d.plugin_type_id.as_deref() == Some("plugin:lv2:https://example.com/no-index")
            && d.detail.contains("lv2:index")
    }));
    assert!(batch.diagnostics.iter().any(|d| {
        d.plugin_type_id.as_deref() == Some("plugin:lv2:https://example.com/dup-index")
            && d.detail.contains("duplicate")
    }));
    let _ = fs::remove_dir_all(root);
}
