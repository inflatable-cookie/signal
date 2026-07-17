use super::introspection::{
    metadata_descriptor, metadata_io_layout, read_au_component_metadata_list, AuComponentMetadata,
};
use super::*;
#[cfg(target_os = "macos")]
use signal_plugin::PluginFeature;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

impl AuHostAdapter {
    /// Returns the default AU scan roots for the given platform.
    pub fn default_scan_roots(&self, platform: AuHostPlatform) -> Vec<AuScanRoot> {
        match platform {
            AuHostPlatform::MacOs => vec![
                AuScanRoot {
                    root: "~/Library/Audio/Plug-Ins/Components".into(),
                    platform,
                    kind: AuScanRootKind::UserComponentRoot,
                },
                AuScanRoot {
                    root: "/Library/Audio/Plug-Ins/Components".into(),
                    platform,
                    kind: AuScanRootKind::SystemComponentRoot,
                },
                AuScanRoot {
                    root: "/System/Library/Components".into(),
                    platform,
                    kind: AuScanRootKind::BuiltInComponentRoot,
                },
            ],
        }
    }

    /// Discover Audio Units through the system AudioComponent registry
    /// (rootless: `AudioComponentFindNext` enumeration per type of interest
    /// — `aufx`, `aumf`, `aumu`; converter/output types are skipped). The
    /// registry executes no plugin code, matching the `moduleinfo.json`
    /// descriptor-only posture. AUv3 components
    /// (`kAudioComponentFlag_IsV3AudioUnit`) are filtered out —
    /// `AudioComponentInstanceNew` cannot instantiate them. Discovered
    /// entries retain the matching filesystem bundle when its plist can be
    /// found, falling back to [`AU_REGISTRY_COMPONENT_PATH`] for registry-only
    /// components. The load key still re-resolves the component at runtime.
    /// Off macOS this returns an empty list.
    pub fn discover_plugins_from_registry(&self) -> Vec<AuDiscoveredPluginType> {
        #[cfg(target_os = "macos")]
        {
            registry_discovery()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Vec::new()
        }
    }

    /// Scans the given filesystem roots for AU component bundles and returns
    /// all discovered plugin types (plist metadata only, kept for the
    /// cross-platform unit tests; production discovery goes through
    /// [`AuHostAdapter::discover_plugins_from_registry`]). An empty root
    /// list scans nothing — system plugin directories are never scanned
    /// implicitly; pass [`AuHostAdapter::default_scan_roots`] explicitly to
    /// opt in.
    pub fn discover_plugins_for_roots(
        &self,
        _platform: AuHostPlatform,
        roots: &[String],
    ) -> Vec<AuDiscoveredPluginType> {
        let mut discovered = Vec::new();
        for root in roots {
            let expanded_root = expand_scan_root(root);
            if expanded_root
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("component"))
            {
                push_au_components_if_present(&mut discovered, &expanded_root);
                continue;
            }
            let Ok(entries) = fs::read_dir(&expanded_root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let Some(file_name) = file_name.to_str() else {
                    continue;
                };
                if !file_name.ends_with(".component") {
                    continue;
                }
                push_au_components_if_present(&mut discovered, &path);
            }
        }
        discovered
    }
}

/// Push every AudioComponent a bundle's plist declares (a bundle may
/// register several components — the historical scan only read the first).
fn push_au_components_if_present(discovered: &mut Vec<AuDiscoveredPluginType>, path: &Path) {
    let Ok(metadata_list) = read_au_component_metadata_list(path) else {
        return;
    };
    for metadata in metadata_list {
        push_discovered_plugin(discovered, &metadata, &path.to_string_lossy());
    }
}

fn push_discovered_plugin(
    discovered: &mut Vec<AuDiscoveredPluginType>,
    metadata: &AuComponentMetadata,
    bundle_root: &str,
) {
    let plugin = AuDiscoveredPluginType {
        plugin_type_id: PluginTypeId(metadata.plugin_type_id.clone()),
        component_type: metadata.component_type.clone(),
        component_subtype: metadata.component_subtype.clone(),
        manufacturer_code: metadata.manufacturer_code.clone(),
        bundle_root: bundle_root.to_string(),
        descriptor: metadata_descriptor(metadata),
        default_io_layout: metadata_io_layout(metadata),
    };
    if !discovered
        .iter()
        .any(|existing: &AuDiscoveredPluginType| existing.plugin_type_id == plugin.plugin_type_id)
    {
        discovered.push(plugin);
    }
}

fn expand_scan_root(root: &str) -> PathBuf {
    if let Some(stripped) = root.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(root)
}

// ── System-registry enumeration (macOS only) ───────────────────────────────

/// AudioComponent types of interest: effects, music effects, instruments.
/// Output (`auou`) and format-converter (`aufc`) units are host plumbing,
/// not user plugins.
#[cfg(target_os = "macos")]
const REGISTRY_TYPES_OF_INTEREST: [&str; 3] = ["aufx", "aumf", "aumu"];

#[cfg(target_os = "macos")]
fn registry_discovery() -> Vec<AuDiscoveredPluginType> {
    use super::hosting::ffi;

    let bundle_metadata = registry_bundle_metadata();
    let mut discovered = Vec::new();
    for type_code in REGISTRY_TYPES_OF_INTEREST {
        let Some(component_type) = super::hosting::fourcc_from_str(type_code) else {
            continue;
        };
        let description = ffi::AudioComponentDescription {
            componentType: component_type,
            componentSubType: 0,
            componentManufacturer: 0,
            componentFlags: 0,
            componentFlagsMask: 0,
        };
        let mut component: ffi::AudioComponent = std::ptr::null_mut();
        loop {
            component = unsafe { ffi::AudioComponentFindNext(component, &description) };
            if component.is_null() {
                break;
            }
            let Some(mut metadata) = (unsafe { registry_component_metadata(component) }) else {
                continue;
            };
            let load_key = format!(
                "{}:{}:{}",
                metadata.component_type, metadata.component_subtype, metadata.manufacturer_code,
            );
            let bundle = bundle_metadata.get(&load_key);
            if let Some(version) = bundle.and_then(|(_, version)| version.as_ref()) {
                metadata.version.clone_from(version);
            }
            let bundle_root = bundle
                .map(|(root, _)| root.as_str())
                .unwrap_or(AU_REGISTRY_COMPONENT_PATH);
            push_discovered_plugin(&mut discovered, &metadata, bundle_root);
        }
    }
    discovered
}

#[cfg(target_os = "macos")]
fn registry_bundle_metadata() -> HashMap<String, (String, Option<String>)> {
    let adapter = AuHostAdapter::default();
    let platform = current_au_platform();
    let roots = adapter
        .default_scan_roots(platform)
        .into_iter()
        .map(|root| root.root)
        .collect::<Vec<_>>();
    adapter
        .discover_plugins_for_roots(platform, &roots)
        .into_iter()
        .map(|plugin| {
            (
                plugin.load_key(),
                (plugin.bundle_root, plugin.descriptor.version),
            )
        })
        .collect()
}

/// Build scan-time metadata for one registry component: identity from its
/// `AudioComponentDescription`, vendor/name from the `"Vendor: Name"`
/// convention of `AudioComponentCopyName`, version decoded from the packed
/// `AudioComponentGetVersion` word. `None` for AUv3 components (filtered)
/// or components whose description cannot be read.
///
/// # Safety
/// `component` must be a live `AudioComponent` from the system registry.
#[cfg(target_os = "macos")]
unsafe fn registry_component_metadata(
    component: super::hosting::ffi::AudioComponent,
) -> Option<AuComponentMetadata> {
    use super::hosting::{ffi, fourcc_to_string};
    use super::introspection::split_component_name;

    let mut description = ffi::AudioComponentDescription::default();
    if unsafe { ffi::AudioComponentGetDescription(component, &mut description) } != 0 {
        return None;
    }
    if description.componentFlags & ffi::kAudioComponentFlag_IsV3AudioUnit != 0 {
        return None;
    }
    let component_type = fourcc_to_string(description.componentType);
    let component_subtype = fourcc_to_string(description.componentSubType);
    let manufacturer_code = fourcc_to_string(description.componentManufacturer);

    let mut cf_name: ffi::CFStringRef = std::ptr::null();
    let component_name = if unsafe { ffi::AudioComponentCopyName(component, &mut cf_name) } == 0 {
        unsafe { ffi::cfstring_into_string(cf_name) }.unwrap_or_default()
    } else {
        String::new()
    };
    let (vendor, name) = split_component_name(&component_name);

    let mut packed_version: u32 = 0;
    let version = if unsafe { ffi::AudioComponentGetVersion(component, &mut packed_version) } == 0 {
        decode_packed_version(packed_version)
    } else {
        "0.0.0".to_string()
    };

    Some(AuComponentMetadata {
        plugin_type_id: format!(
            "plugin:au:{component_type}:{component_subtype}:{manufacturer_code}"
        ),
        component_type: component_type.clone(),
        component_subtype,
        manufacturer_code,
        vendor: vendor.unwrap_or_else(|| "Unknown".into()),
        name: if name.is_empty() {
            format!("AU {component_type}")
        } else {
            name
        },
        version,
        audio_inputs: if component_type == "aumu" { 0 } else { 2 },
        audio_outputs: 2,
        midi_inputs: if component_type == "aufx" { 0 } else { 1 },
        midi_outputs: 0,
        features: if component_type == "aumu" {
            vec![PluginFeature::Instrument]
        } else {
            vec![PluginFeature::AudioEffect]
        },
    })
}

/// Decode the packed AudioComponent version word (`0x00MMmmpp`-style
/// major/minor/patch fields) into `"M.m.p"`.
#[cfg(target_os = "macos")]
fn decode_packed_version(packed: u32) -> String {
    let major = (packed >> 16) & 0xFFFF;
    let minor = (packed >> 8) & 0xFF;
    let patch = packed & 0xFF;
    format!("{major}.{minor}.{patch}")
}

#[cfg(all(test, target_os = "macos"))]
mod registry_tests {
    use super::super::hosting::{ffi, fourcc_from_str};
    use super::*;

    #[test]
    fn packed_versions_decode_to_dotted_triples() {
        assert_eq!(decode_packed_version(0x0001_0000), "1.0.0");
        assert_eq!(decode_packed_version(0x0002_0301), "2.3.1");
    }

    /// Every registry component carrying the AUv3 flag must be absent from
    /// the discovered set (vacuous on machines without AUv3 components).
    /// Triples that ALSO exist as a v2 registration are skipped — the v2
    /// twin is legitimately discovered under the same plugin_type_id.
    #[test]
    fn registry_discovery_filters_auv3_components() {
        let discovered = AuHostAdapter::default().discover_plugins_from_registry();
        let mut v2_triples = Vec::new();
        let mut v3_triples = Vec::new();
        for type_code in REGISTRY_TYPES_OF_INTEREST {
            let description = ffi::AudioComponentDescription {
                componentType: fourcc_from_str(type_code).expect("type code"),
                componentSubType: 0,
                componentManufacturer: 0,
                componentFlags: 0,
                componentFlagsMask: 0,
            };
            let mut component: ffi::AudioComponent = std::ptr::null_mut();
            loop {
                component = unsafe { ffi::AudioComponentFindNext(component, &description) };
                if component.is_null() {
                    break;
                }
                let mut resolved = ffi::AudioComponentDescription::default();
                if unsafe { ffi::AudioComponentGetDescription(component, &mut resolved) } != 0 {
                    continue;
                }
                let triple = format!(
                    "plugin:au:{}:{}:{}",
                    super::super::hosting::fourcc_to_string(resolved.componentType),
                    super::super::hosting::fourcc_to_string(resolved.componentSubType),
                    super::super::hosting::fourcc_to_string(resolved.componentManufacturer),
                );
                if resolved.componentFlags & ffi::kAudioComponentFlag_IsV3AudioUnit != 0 {
                    v3_triples.push(triple);
                } else {
                    v2_triples.push(triple);
                }
            }
        }
        for v3_id in v3_triples {
            if v2_triples.contains(&v3_id) {
                continue;
            }
            assert!(
                !discovered
                    .iter()
                    .any(|plugin| plugin.plugin_type_id.0 == v3_id),
                "AUv3 component leaked into discovery: {v3_id}",
            );
        }
    }
}
