//! Bundle-level LV2 introspection: parse a `.lv2` bundle's real Turtle
//! manifests into plugin models (URI, binary, ports, features).
//!
//! `manifest.ttl` names the plugins (`a lv2:Plugin`), their `lv2:binary`,
//! and `rdfs:seeAlso` data files; the port/feature model usually lives in
//! the seeAlso files. seeAlso references are chased WITHIN the bundle
//! directory only (a lexical containment check) and each file parses once
//! per bundle. Multi-plugin manifests are first-class: one `manifest.ttl`
//! may list several plugins with separate seeAlso files, and a failure in
//! one plugin's data leaves the others discoverable.
//!
//! These functions are reused verbatim by the hosting side: the sandbox
//! child re-parses the bundle TTL at load time (library_path = the bundle
//! directory, load key = the plugin URI), paralleling AU's
//! rebuild-description-from-load-key posture.

use super::turtle::{TurtleDocument, TurtleTerm};
use super::*;
use signal_plugin::{
    PluginDescriptor, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessingContract, PluginStateContract,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

// ── Vocabulary ──────────────────────────────────────────────────────────────

const LV2_CORE: &str = "http://lv2plug.in/ns/lv2core#";
const RDFS_SEE_ALSO: &str = "http://www.w3.org/2000/01/rdf-schema#seeAlso";
const DOAP_NAME: &str = "http://usefulinc.com/ns/doap#name";
const ATOM_PORT: &str = "http://lv2plug.in/ns/ext/atom#AtomPort";
const EVENT_PORT: &str = "http://lv2plug.in/ns/ext/event#EventPort";

/// The only feature phase-1 hosting provides (packet g11.033 decision 3);
/// the scan pre-filter allowlist is exactly this set.
pub(super) const URID_MAP_FEATURE: &str = "http://lv2plug.in/ns/ext/urid#map";

fn lv2(term: &str) -> String {
    format!("{LV2_CORE}{term}")
}

// ── Bundle model ────────────────────────────────────────────────────────────

/// One plugin's model parsed from the bundle TTL.
#[derive(Clone, Debug, PartialEq)]
pub struct Lv2PluginModel {
    /// The canonical LV2 plugin URI.
    pub plugin_uri: String,
    /// `doap:name`, when declared.
    pub name: Option<String>,
    /// Resolved absolute path of the `lv2:binary` shared library.
    pub binary_path: PathBuf,
    /// `lv2:requiredFeature` IRIs.
    pub required_features: Vec<String>,
    /// `lv2:optionalFeature` IRIs.
    pub optional_features: Vec<String>,
    /// Ports sorted by `lv2:index` (declaration order does not matter).
    pub ports: Vec<Lv2Port>,
}

impl Lv2PluginModel {
    /// Audio input ports in `lv2:index` order.
    pub fn audio_inputs(&self) -> Vec<&Lv2Port> {
        self.ports
            .iter()
            .filter(|port| port.classes.audio && port.classes.input)
            .collect()
    }

    /// Audio output ports in `lv2:index` order.
    pub fn audio_outputs(&self) -> Vec<&Lv2Port> {
        self.ports
            .iter()
            .filter(|port| port.classes.audio && port.classes.output)
            .collect()
    }

    /// Control input ports (the phase-1 parameter surface).
    pub fn control_inputs(&self) -> Vec<&Lv2Port> {
        self.ports
            .iter()
            .filter(|port| port.classes.control && port.classes.input)
            .collect()
    }

    /// Atom/event INPUT ports that are not `lv2:connectionOptional` — the
    /// stereo gate requires zero of these (they mean the plugin needs an
    /// event stream, e.g. instruments needing MIDI).
    pub fn required_event_inputs(&self) -> usize {
        self.ports
            .iter()
            .filter(|port| {
                (port.classes.atom || port.classes.event)
                    && port.classes.input
                    && !port.connection_optional
            })
            .count()
    }

    /// Required features the phase-1 host does not provide.
    pub fn unsupported_required_features(&self) -> Vec<String> {
        self.required_features
            .iter()
            .filter(|feature| feature.as_str() != URID_MAP_FEATURE)
            .cloned()
            .collect()
    }
}

/// The result of parsing one `.lv2` bundle: complete plugin models plus
/// per-plugin failures (multi-plugin bundles degrade per plugin).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lv2BundleModel {
    /// Plugins whose model parsed completely.
    pub plugins: Vec<Lv2PluginModel>,
    /// Per-plugin failures: `(plugin_uri if known, detail)`.
    pub plugin_faults: Vec<(Option<String>, String)>,
}

/// Parse a `.lv2` bundle directory. A bundle-level error (unreadable or
/// malformed `manifest.ttl`) fails the whole bundle; per-plugin problems
/// (missing binary, bad ports, broken seeAlso file) land in
/// `plugin_faults` and leave sibling plugins intact.
pub fn parse_lv2_bundle(bundle_root: &Path) -> Result<Lv2BundleModel, String> {
    let manifest_path = bundle_root.join("manifest.ttl");
    let manifest_text = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest =
        TurtleDocument::parse(&manifest_text).map_err(|error| format!("manifest.ttl: {error}"))?;

    let plugin_uris = manifest.iri_subjects_of_type(&lv2("Plugin"));
    if plugin_uris.is_empty() {
        return Err("manifest.ttl declares no lv2:Plugin subjects".into());
    }

    // Each seeAlso file parses once per bundle, even when several plugins
    // share it (the multi-plugin layout).
    let mut file_cache: BTreeMap<PathBuf, Result<TurtleDocument, String>> = BTreeMap::new();
    let mut model = Lv2BundleModel::default();

    for plugin_uri in plugin_uris {
        let subject = TurtleTerm::Iri(plugin_uri.clone());

        // Merge the manifest with the plugin's local seeAlso files.
        let mut doc = manifest.clone();
        let mut see_also_failure: Option<String> = None;
        for object in manifest.objects(&subject, RDFS_SEE_ALSO) {
            let Some(reference) = object.as_iri() else {
                continue;
            };
            let Some(path) = resolve_bundle_local(bundle_root, reference) else {
                // seeAlso outside the bundle directory: out of scope by
                // packet decision — skipped, never chased.
                continue;
            };
            let parsed = file_cache.entry(path.clone()).or_insert_with(|| {
                let text = fs::read_to_string(&path)
                    .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
                TurtleDocument::parse(&text).map_err(|error| format!("{}: {error}", path.display()))
            });
            match parsed {
                Ok(extra) => doc.merge(extra),
                Err(error) => {
                    see_also_failure = Some(error.clone());
                    break;
                }
            }
        }
        if let Some(detail) = see_also_failure {
            model.plugin_faults.push((Some(plugin_uri), detail));
            continue;
        }

        match plugin_model_from_document(bundle_root, &doc, &plugin_uri) {
            Ok(plugin) => model.plugins.push(plugin),
            Err(detail) => model.plugin_faults.push((Some(plugin_uri), detail)),
        }
    }
    Ok(model)
}

/// Resolve a (usually relative) IRI reference against the bundle directory,
/// returning `None` when the reference escapes the bundle or carries a
/// scheme (absolute URI).
fn resolve_bundle_local(bundle_root: &Path, reference: &str) -> Option<PathBuf> {
    if reference.contains("://") || reference.starts_with('/') || reference.contains("..") {
        return None;
    }
    Some(bundle_root.join(reference))
}

fn plugin_model_from_document(
    bundle_root: &Path,
    doc: &TurtleDocument,
    plugin_uri: &str,
) -> Result<Lv2PluginModel, String> {
    let subject = TurtleTerm::Iri(plugin_uri.to_string());

    let binary_reference = doc
        .object(&subject, &lv2("binary"))
        .and_then(TurtleTerm::as_iri)
        .ok_or_else(|| "plugin declares no lv2:binary".to_string())?;
    let binary_path = resolve_bundle_local(bundle_root, binary_reference)
        .ok_or_else(|| format!("lv2:binary <{binary_reference}> is not bundle-local"))?;

    let name = doc
        .object(&subject, DOAP_NAME)
        .and_then(TurtleTerm::as_literal)
        .map(str::to_string);

    let required_features = iri_objects(doc, &subject, &lv2("requiredFeature"));
    let optional_features = iri_objects(doc, &subject, &lv2("optionalFeature"));

    let mut ports = Vec::new();
    for port_term in doc.objects(&subject, &lv2("port")) {
        ports.push(port_from_term(doc, port_term)?);
    }
    ports.sort_by_key(|port: &Lv2Port| port.index);
    for pair in ports.windows(2) {
        if pair[0].index == pair[1].index {
            return Err(format!("duplicate lv2:index {}", pair[0].index));
        }
    }

    Ok(Lv2PluginModel {
        plugin_uri: plugin_uri.to_string(),
        name,
        binary_path,
        required_features,
        optional_features,
        ports,
    })
}

fn iri_objects(doc: &TurtleDocument, subject: &TurtleTerm, predicate: &str) -> Vec<String> {
    doc.objects(subject, predicate)
        .filter_map(|object| object.as_iri().map(str::to_string))
        .collect()
}

fn port_from_term(doc: &TurtleDocument, port: &TurtleTerm) -> Result<Lv2Port, String> {
    let index = doc
        .object(port, &lv2("index"))
        .and_then(TurtleTerm::as_number)
        .ok_or_else(|| "port declares no lv2:index".to_string())?;
    if index < 0.0 || index.fract() != 0.0 || index > u32::MAX as f64 {
        return Err(format!("invalid lv2:index {index}"));
    }
    let classes = Lv2PortClasses {
        audio: doc.has_type(port, &lv2("AudioPort")),
        control: doc.has_type(port, &lv2("ControlPort")),
        atom: doc.has_type(port, ATOM_PORT),
        event: doc.has_type(port, EVENT_PORT),
        input: doc.has_type(port, &lv2("InputPort")),
        output: doc.has_type(port, &lv2("OutputPort")),
    };
    let number = |predicate: &str| -> Option<f32> {
        doc.object(port, &lv2(predicate))
            .and_then(TurtleTerm::as_number)
            .map(|value| value as f32)
    };
    let connection_optional = doc
        .objects(port, &lv2("portProperty"))
        .any(|object| object.as_iri() == Some(lv2("connectionOptional").as_str()));
    Ok(Lv2Port {
        index: index as u32,
        symbol: doc
            .object(port, &lv2("symbol"))
            .and_then(TurtleTerm::as_literal)
            .map(str::to_string),
        name: doc
            .object(port, &lv2("name"))
            .and_then(TurtleTerm::as_literal)
            .map(str::to_string),
        classes,
        default: number("default"),
        minimum: number("minimum"),
        maximum: number("maximum"),
        connection_optional,
    })
}

// ── Discovery mapping ───────────────────────────────────────────────────────

/// Control input ports as the phase-1 parameter inventory:
/// `parameter_id` = the port's `lv2:index`, name/range from the TTL,
/// `default_normalized` = the effective default's normalized position in
/// `[minimum, maximum]` (0.0 for unbounded ports).
pub fn parameter_descriptors_from_model(model: &Lv2PluginModel) -> Vec<PluginParameterDescriptor> {
    model
        .control_inputs()
        .iter()
        .map(|port| {
            let default_plain = port.effective_default();
            let (min_plain, max_plain) = match (port.minimum, port.maximum) {
                (Some(minimum), Some(maximum)) if maximum > minimum => (minimum, maximum),
                _ => (0.0, 1.0),
            };
            let default_normalized = if max_plain > min_plain {
                ((default_plain - min_plain) / (max_plain - min_plain)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            PluginParameterDescriptor {
                parameter_id: port.index,
                name: port
                    .name
                    .clone()
                    .or_else(|| port.symbol.clone())
                    .unwrap_or_else(|| format!("Port {}", port.index)),
                unit: None,
                domain: PluginParameterDomain::GenericNormalized,
                default_normalized,
                min_plain,
                max_plain,
                flags: PluginParameterFlags::automatable(),
            }
        })
        .collect()
}

/// Build the discovered plugin type (descriptor + coordinates) from a
/// parsed model.
pub(super) fn discovered_plugin_from_model(
    bundle_root: &Path,
    model: &Lv2PluginModel,
) -> Lv2DiscoveredPluginType {
    let default_io_layout = PluginIoLayout {
        audio_inputs: model.audio_inputs().len() as u16,
        audio_outputs: model.audio_outputs().len() as u16,
        midi_inputs: model
            .ports
            .iter()
            .filter(|port| (port.classes.atom || port.classes.event) && port.classes.input)
            .count() as u16,
        midi_outputs: model
            .ports
            .iter()
            .filter(|port| (port.classes.atom || port.classes.event) && port.classes.output)
            .count() as u16,
    };
    let plugin_type_id = format!("plugin:lv2:{}", model.plugin_uri);
    let display_name = model.name.clone().unwrap_or_else(|| {
        model
            .plugin_uri
            .rsplit(['/', '#'])
            .next()
            .unwrap_or(&model.plugin_uri)
            .to_string()
    });

    // LV2 TTL carries no vendor short-form in the subset (doap:maintainer
    // is a nested resource); report Unknown like format adapters do when
    // metadata is absent.
    let mut descriptor = PluginDescriptor::new(
        plugin_type_id.clone(),
        "Unknown",
        &display_name,
        PluginFormat::Lv2,
    )
    .with_audio_buses(default_io_layout.main_audio_buses())
    .with_parameters(parameter_descriptors_from_model(model))
    .with_state_contract(PluginStateContract {
        supports_snapshot: false,
        supports_reset: false,
        supports_bypass: false,
        exposes_latency: false,
        exposes_tail: false,
    })
    .with_processing_contract(PluginProcessingContract {
        max_block_frames: 4_096,
        sample_accurate_automation: false,
        accepts_midi: default_io_layout.midi_inputs > 0,
        accepts_note_events: default_io_layout.midi_inputs > 0,
        supports_note_expression: false,
        produces_midi: default_io_layout.midi_outputs > 0,
        silence_aware: true,
    })
    .with_lifecycle_contract(PluginLifecycleContract {
        requires_main_thread_for_state: false,
        supports_prepare: true,
        supports_activate: true,
        supports_reset_while_active: false,
    });
    let feature = if default_io_layout.midi_inputs > 0 && default_io_layout.audio_outputs > 0 {
        PluginFeature::Instrument
    } else {
        PluginFeature::AudioEffect
    };
    descriptor = descriptor.with_feature(feature);

    Lv2DiscoveredPluginType {
        plugin_type_id: PluginTypeId(plugin_type_id),
        plugin_uri: model.plugin_uri.clone(),
        bundle_root: bundle_root.to_string_lossy().into_owned(),
        manifest_path: bundle_root
            .join("manifest.ttl")
            .to_string_lossy()
            .into_owned(),
        binary_path: model.binary_path.to_string_lossy().into_owned(),
        required_features: model.required_features.clone(),
        optional_features: model.optional_features.clone(),
        ports: model.ports.clone(),
        prepare_fault: None,
        descriptor,
        default_io_layout,
    }
}
