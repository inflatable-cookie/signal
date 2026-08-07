//! Bundle snapshot assembly from metadata + factory classes.

use crate::vst3_host_adapter::Vst3HostPlatform;
use signal_plugin::{
    PluginDescriptor, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginStateContract,
};
use std::{fs, io, path::Path};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

use super::derive::*;
use super::paths::{
    candidate_moduleinfo_paths, preflight_vendor_scan_access, read_vst3_bundle_info,
};
use super::scan_helper::load_vst3_factory_classes_with_helper;
use super::types::*;

pub(crate) fn read_vst3_bundle_snapshot(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<Vst3BundleSnapshot> {
    let bundle = read_vst3_bundle_info(bundle_root)?;
    preflight_vendor_scan_access(&bundle)?;
    let (factory_vendor, factory_classes) = read_vst3_factory_snapshot(bundle_root, platform)?;
    let component_classes = factory_classes
        .iter()
        .filter(|class| class.role == Vst3FactoryClassRole::Component)
        .cloned()
        .collect::<Vec<_>>();
    if component_classes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing VST3 component classes",
        ));
    }

    let component_count = component_classes.len();
    let plugins = component_classes
        .iter()
        .map(|component| {
            let controller = match_vst3_controller(component, &factory_classes, component_count);
            let is_instrument = class_is_instrument(component);
            let plugin_type_id = if component_count == 1 {
                bundle
                    .signal_plugin_type_id
                    .clone()
                    .unwrap_or_else(|| derive_plugin_type_id(&bundle, component))
            } else {
                derive_plugin_type_id(&bundle, component)
            };
            let io_layout = derive_io_layout(&bundle, is_instrument);
            let features = bundle
                .signal_features
                .clone()
                .unwrap_or_else(|| default_features(is_instrument));
            Ok(Vst3ModuleMetadata {
                plugin_type_id,
                class_id: component.class_id.clone(),
                controller_class_id: controller.as_ref().map(|class| class.class_id.clone()),
                category: component.category.clone(),
                vendor: component
                    .vendor()
                    .unwrap_or_else(|| fallback_vendor(&bundle, factory_vendor.as_deref())),
                name: component.name.clone(),
                version: component.version().unwrap_or_else(|| {
                    bundle
                        .version
                        .clone()
                        .unwrap_or_else(|| "0.1.0".to_string())
                }),
                audio_inputs: io_layout.audio_inputs,
                audio_outputs: io_layout.audio_outputs,
                midi_inputs: io_layout.midi_inputs,
                midi_outputs: io_layout.midi_outputs,
                features,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;

    Ok(Vst3BundleSnapshot {
        plugins,
        factory_classes,
    })
}

pub(crate) fn metadata_io_layout(metadata: &Vst3ModuleMetadata) -> PluginIoLayout {
    PluginIoLayout {
        audio_inputs: metadata.audio_inputs,
        audio_outputs: metadata.audio_outputs,
        midi_inputs: metadata.midi_inputs,
        midi_outputs: metadata.midi_outputs,
    }
}

pub(crate) fn metadata_descriptor(metadata: &Vst3ModuleMetadata) -> PluginDescriptor {
    let io_layout = metadata_io_layout(metadata);
    let mut descriptor = PluginDescriptor::new(
        metadata.plugin_type_id.clone(),
        metadata.vendor.clone(),
        metadata.name.clone(),
        PluginFormat::Vst3,
    )
    .with_version(metadata.version.as_str())
    .with_audio_buses(io_layout.main_audio_buses())
    // Scan-time parameter inventory is intentionally EMPTY: real inventories
    // arrive at load time via IEditController (g11.031, mirrors CLAP).
    .with_parameters(Vec::new())
    .with_state_contract(PluginStateContract {
        supports_snapshot: true,
        supports_reset: true,
        supports_bypass: true,
        exposes_latency: true,
        exposes_tail: true,
    })
    .with_lifecycle_contract(PluginLifecycleContract {
        requires_main_thread_for_state: false,
        supports_prepare: true,
        supports_activate: true,
        supports_reset_while_active: true,
    })
    .with_processing_contract(PluginProcessingContract {
        max_block_frames: 2048,
        sample_accurate_automation: true,
        accepts_midi: metadata.midi_inputs > 0,
        accepts_note_events: metadata.midi_inputs > 0,
        supports_note_expression: metadata.midi_inputs > 0,
        produces_midi: metadata.midi_outputs > 0,
        silence_aware: true,
    });
    descriptor.features = metadata.features.clone();
    descriptor
}

pub(crate) fn read_vst3_factory_snapshot(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<(Option<String>, Vec<Vst3FactoryClass>)> {
    let mut moduleinfo_error = None;
    if let Some(moduleinfo_path) = candidate_moduleinfo_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
    {
        match json5::from_str::<ModuleInfoDocument>(&fs::read_to_string(moduleinfo_path)?) {
            Ok(document) => {
                let vendor = document
                    .factory_info
                    .and_then(|factory| factory.vendor)
                    .or_else(|| {
                        document
                            .classes
                            .iter()
                            .find_map(|class| class.vendor.clone())
                    });
                let classes = document
                    .classes
                    .into_iter()
                    .map(|class| Vst3FactoryClass {
                        role: role_from_category(&class.category),
                        class_id: class.cid,
                        category: class.category,
                        name: class.name,
                        vendor: class.vendor,
                        version: class.version,
                        subcategories: class.subcategories,
                    })
                    .collect::<Vec<_>>();
                if !classes.is_empty() {
                    return Ok((vendor, classes));
                }
                moduleinfo_error = Some("missing VST3 classes in moduleinfo.json".to_string());
            }
            Err(error) => {
                moduleinfo_error = Some(format!("invalid VST3 moduleinfo.json: {error}"));
            }
        }
    }
    match load_vst3_factory_classes_with_helper(bundle_root, platform) {
        Ok(snapshot) => Ok(snapshot),
        Err(error) if moduleinfo_error.is_some() => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{}; factory fallback failed: {error}",
                moduleinfo_error.expect("checked moduleinfo error")
            ),
        )),
        Err(error) => Err(error),
    }
}

/// Whether `moduleinfo.json` explicitly advertises `class_id` as a component.
/// Hosting uses this to safely recover from vendors that ship stale generated
/// class IDs while their binary exposes one unambiguous component class.
pub(crate) fn moduleinfo_declares_component_class(bundle_root: &Path, class_id: &str) -> bool {
    candidate_moduleinfo_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|contents| json5::from_str::<ModuleInfoDocument>(&contents).ok())
        .is_some_and(|document| {
            document.classes.into_iter().any(|class| {
                role_from_category(&class.category) == Vst3FactoryClassRole::Component
                    && class.cid.eq_ignore_ascii_case(class_id)
            })
        })
}
