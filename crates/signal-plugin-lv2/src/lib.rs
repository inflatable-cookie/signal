//! LV2 plugin format adapter for Signal: real Turtle-manifest discovery
//! and dlopen-based hosting (g11.033).
//!
//! Discovery is pure file parsing over a handwritten Turtle subset
//! ([`turtle`]) — no lilv/serd/RDF dependencies and no plugin binary is
//! opened at scan time. Hosting ([`Lv2HostedInstance`]) re-parses the
//! bundle TTL at load (library path = the `.lv2` bundle directory, load
//! key = the bare plugin URI) and drives the plain LV2 C ABI: dlopen +
//! `lv2_descriptor(index)` walk, instantiate at activate, connected ports,
//! `run(n)` per block.

#![warn(missing_docs)]

#[doc(hidden)]
pub mod fixture;
mod lv2_host_adapter;

pub use lv2_host_adapter::*;

#[cfg(test)]
mod tests {
    use super::{Lv2DiscoveryDiagnosticKind, Lv2HostAdapter, Lv2HostPlatform};
    use signal_plugin::{PluginFeature, PluginFormat};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_plugin_root(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("signal-lv2-{label}-{unique}"));
        fs::create_dir_all(&root).expect("temp lv2 root should be created");
        root
    }

    fn write_bundle(root: &std::path::Path, bundle: &str, files: &[(&str, &str)]) {
        let bundle_root = root.join(bundle);
        fs::create_dir_all(&bundle_root).expect("lv2 bundle should be created");
        for (name, contents) in files {
            fs::write(bundle_root.join(name), contents).expect("bundle file should be written");
        }
    }

    fn scan(adapter: &Lv2HostAdapter, root: &std::path::Path) -> super::Lv2DiscoveryBatch {
        adapter.discover_plugins_for_roots_with_diagnostics(
            super::current_lv2_platform(),
            &[root.display().to_string()],
        )
    }

    #[test]
    fn lv2_adapter_reports_supported_format_and_capabilities() {
        let adapter = Lv2HostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Lv2));
        assert!(!adapter.supports_format(PluginFormat::Clap));
        assert!(adapter.strict_sandbox_default());
        assert!(adapter.advertised_capabilities(2048).supports_state);
    }

    #[test]
    fn default_scan_roots_cover_macos_and_linux() {
        let adapter = Lv2HostAdapter::default();
        let macos: Vec<_> = adapter
            .default_scan_roots(Lv2HostPlatform::MacOs)
            .into_iter()
            .map(|root| root.root)
            .collect();
        assert!(macos.contains(&"~/Library/Audio/Plug-Ins/LV2".to_string()));
        assert!(macos.contains(&"/Library/Audio/Plug-Ins/LV2".to_string()));
        let linux: Vec<_> = adapter
            .default_scan_roots(Lv2HostPlatform::Linux)
            .into_iter()
            .map(|root| root.root)
            .collect();
        assert!(linux.contains(&"~/.lv2".to_string()));
        assert!(linux.contains(&"/usr/lib/lv2".to_string()));
        assert!(linux.contains(&"/usr/local/lib/lv2".to_string()));
    }

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
            .any(|d| d.plugin_type_id.as_deref()
                == Some("plugin:lv2:https://example.com/binaryless")));
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
}
