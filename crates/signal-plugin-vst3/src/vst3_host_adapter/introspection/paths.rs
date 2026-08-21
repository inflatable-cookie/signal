//! VST3 bundle path resolution helpers.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[cfg(not(target_os = "macos"))]
use libloading::Library;

use crate::vst3_host_adapter::Vst3HostPlatform;
use super::derive::{
    parse_feature_list, plist_string, plist_string_array, plist_to_io_error, plist_u16,
};
use super::types::*;

pub(crate) fn candidate_info_plist_paths(bundle_root: &Path) -> Vec<PathBuf> {
    vec![bundle_root.join("Contents").join("Info.plist")]
}

pub(crate) fn candidate_moduleinfo_paths(bundle_root: &Path) -> Vec<PathBuf> {
    vec![
        bundle_root
            .join("Contents")
            .join("Resources")
            .join(VST3_MODULEINFO_FILE),
        bundle_root.join("Contents").join(VST3_MODULEINFO_FILE),
    ]
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn resolve_module_binary_path(
    bundle_root: &Path,
    platform: Vst3HostPlatform,
) -> io::Result<PathBuf> {
    let bundle = read_vst3_bundle_info(bundle_root)?;
    let bundle_stem = bundle_root
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid VST3 bundle name"))?;
    let executable_name = bundle.executable_name.as_deref().unwrap_or(bundle_stem);
    let direct_candidates = match platform {
        Vst3HostPlatform::MacOs => vec![bundle_root
            .join("Contents")
            .join("MacOS")
            .join(executable_name)],
        Vst3HostPlatform::Linux => vec![
            bundle_root
                .join("Contents")
                .join("x86_64-linux")
                .join(format!("{executable_name}.so")),
            bundle_root
                .join("Contents")
                .join("aarch64-linux")
                .join(format!("{executable_name}.so")),
        ],
        Vst3HostPlatform::Windows => vec![
            bundle_root
                .join("Contents")
                .join("x86_64-win")
                .join(format!("{executable_name}.vst3")),
            bundle_root
                .join("Contents")
                .join("arm64-win")
                .join(format!("{executable_name}.vst3")),
        ],
    };
    if let Some(path) = direct_candidates.into_iter().find(|path| path.is_file()) {
        return Ok(path);
    }
    let search_root = bundle_root.join("Contents");
    let entries = fs::read_dir(&search_root).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "failed to read VST3 bundle contents for module resolution: {}",
                search_root.display()
            ),
        )
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let Ok(children) = fs::read_dir(&path) else {
                continue;
            };
            for child in children.flatten() {
                let child_path = child.path();
                if child_path.is_file() {
                    return Ok(child_path);
                }
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "unable to resolve VST3 module binary path",
    ))
}

pub(crate) fn read_vst3_bundle_info(bundle_root: &Path) -> io::Result<Vst3BundleInfo> {
    let mut bundle = Vst3BundleInfo {
        bundle_name: bundle_root
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::to_string),
        ..Vst3BundleInfo::default()
    };
    let Some(info_plist_path) = candidate_info_plist_paths(bundle_root)
        .into_iter()
        .find(|path| path.is_file())
    else {
        return Ok(bundle);
    };
    let value = plist::Value::from_file(&info_plist_path).map_err(plist_to_io_error)?;
    let Some(dict) = value.into_dictionary() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VST3 Info.plist should be a dictionary",
        ));
    };

    bundle.bundle_identifier = plist_string(&dict, "CFBundleIdentifier");
    bundle.bundle_name = plist_string(&dict, "CFBundleName")
        .or_else(|| plist_string(&dict, "CFBundleDisplayName"))
        .or(bundle.bundle_name);
    bundle.executable_name = plist_string(&dict, "CFBundleExecutable");
    bundle.version = plist_string(&dict, "CFBundleShortVersionString")
        .or_else(|| plist_string(&dict, "CFBundleVersion"));
    bundle.signal_plugin_type_id = plist_string(&dict, "SignalPluginTypeId");
    bundle.signal_audio_inputs = plist_u16(&dict, "SignalAudioInputs");
    bundle.signal_audio_outputs = plist_u16(&dict, "SignalAudioOutputs");
    bundle.signal_midi_inputs = plist_u16(&dict, "SignalMidiInputs");
    bundle.signal_midi_outputs = plist_u16(&dict, "SignalMidiOutputs");
    bundle.signal_features = plist_string_array(&dict, "SignalFeatures")
        .map(|features| parse_feature_list(&features.join(",")))
        .transpose()?;

    Ok(bundle)
}
pub(crate) fn preflight_vendor_scan_access(bundle: &Vst3BundleInfo) -> io::Result<()> {
    if !bundle
        .bundle_identifier
        .as_deref()
        .is_some_and(is_native_instruments_bundle)
    {
        return Ok(());
    }
    let Some(home) = std::env::var_os("HOME") else {
        return Ok(());
    };
    let documents = PathBuf::from(home)
        .join("Documents")
        .join("Native Instruments");
    let denied = match fs::read_dir(&documents) {
        Err(error) => error.kind() == io::ErrorKind::PermissionDenied,
        Ok(mut entries) => entries.next().is_some_and(|entry| {
            entry.is_err_and(|error| error.kind() == io::ErrorKind::PermissionDenied)
        }),
    };
    if denied {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Native Instruments VST3 inspection requires macOS Documents folder access ({})",
                documents.display()
            ),
        ))
    } else {
        Ok(())
    }
}
pub(crate) fn is_native_instruments_bundle(identifier: &str) -> bool {
    identifier
        .to_ascii_lowercase()
        .starts_with("com.native-instruments.")
}
