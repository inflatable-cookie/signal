use super::*;

/// rdfs:seeAlso references escaping the bundle directory are never
/// chased (packet: seeAlso resolves WITHIN the bundle only).
#[test]
fn see_also_outside_the_bundle_is_ignored() {
    let adapter = Lv2HostAdapter::default();
    let root = temp_plugin_root("escape");
    write_bundle(
        &root,
        "escapist.lv2",
        &[(
            "manifest.ttl",
            "@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             <https://example.com/escapist> a lv2:Plugin ;\n\
             \tlv2:binary <e.so> ;\n\
             \trdfs:seeAlso <../../../etc/passwd> , <https://example.com/remote.ttl> ;\n\
             \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"in\" ] .\n",
        )],
    );
    let batch = scan(&adapter, &root);
    assert_eq!(batch.diagnostics, vec![]);
    assert_eq!(batch.discovered.len(), 1);
    let _ = fs::remove_dir_all(root);
}
