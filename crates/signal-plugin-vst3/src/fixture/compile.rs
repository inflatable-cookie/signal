use super::source::vst3_fixture_source_with_default_bus_channels;
use super::VST3_FIXTURE_CLASS_ID_HEX;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Returns `true` when a `rustc` binary is invocable (fixture tests skip
/// gracefully when it is not).
pub fn rustc_available() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Compile the fixture bundle into `directory`, returning the bundle root
/// (`<plugin-name>.vst3`). The bundle carries `moduleinfo.json` (so scans
/// never execute the module) and an `Info.plist` with the Signal metadata
/// keys. Errors carry the rustc failure detail.
pub fn compile_vst3_fixture(
    directory: &Path,
    plugin_type_id: &str,
    plugin_name: &str,
) -> Result<PathBuf, String> {
    compile_vst3_fixture_with_default_bus_channels(directory, plugin_type_id, plugin_name, 2)
}

/// Compile the fixture with a configurable default main-bus channel count.
/// The processor still accepts stereo negotiation, allowing hosts to prove
/// dynamic layout handling for plugins that initialize as mono.
pub fn compile_vst3_fixture_with_default_bus_channels(
    directory: &Path,
    plugin_type_id: &str,
    plugin_name: &str,
    default_bus_channels: u16,
) -> Result<PathBuf, String> {
    let module_name = plugin_name.to_lowercase().replace(' ', "-");
    let bundle_root = directory.join(format!("{module_name}.vst3"));
    let contents = bundle_root.join("Contents");
    let module_dir = if cfg!(target_os = "macos") {
        contents.join("MacOS")
    } else if cfg!(target_os = "windows") {
        contents.join(if cfg!(target_arch = "aarch64") {
            "arm64-win"
        } else {
            "x86_64-win"
        })
    } else {
        contents.join(if cfg!(target_arch = "aarch64") {
            "aarch64-linux"
        } else {
            "x86_64-linux"
        })
    };
    let module_path = if cfg!(target_os = "macos") {
        module_dir.join(&module_name)
    } else if cfg!(target_os = "windows") {
        module_dir.join(format!("{module_name}.vst3"))
    } else {
        module_dir.join(format!("{module_name}.so"))
    };
    std::fs::create_dir_all(&module_dir)
        .map_err(|error| format!("fixture module dir create failed: {error}"))?;
    std::fs::create_dir_all(contents.join("Resources"))
        .map_err(|error| format!("fixture resources dir create failed: {error}"))?;

    let source_path = directory.join(format!("{module_name}-fixture.rs"));
    std::fs::write(
        &source_path,
        vst3_fixture_source_with_default_bus_channels(plugin_name, default_bus_channels),
    )
    .map_err(|error| format!("fixture source write failed: {error}"))?;
    let output = Command::new("rustc")
        .arg("--crate-type=cdylib")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&module_path)
        .output()
        .map_err(|error| format!("rustc invocation failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "vst3 fixture compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    std::fs::write(
        contents.join("Info.plist"),
        fixture_info_plist(plugin_type_id, plugin_name, &module_name),
    )
    .map_err(|error| format!("fixture Info.plist write failed: {error}"))?;
    std::fs::write(
        contents.join("Resources").join("moduleinfo.json"),
        fixture_moduleinfo(plugin_name),
    )
    .map_err(|error| format!("fixture moduleinfo write failed: {error}"))?;
    Ok(bundle_root)
}

fn fixture_info_plist(plugin_type_id: &str, plugin_name: &str, module_name: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{module_name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>dev.signal.vst3-fixture</string>\n\
  <key>CFBundleName</key>\n\
  <string>{plugin_name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>0.1.0</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>2</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>2</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>0</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>0</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
    <string>AudioEffect</string>\n\
    <string>Utility</string>\n\
  </array>\n\
</dict>\n\
</plist>\n"
    )
}

fn fixture_moduleinfo(plugin_name: &str) -> String {
    format!(
        "{{\n  \"Name\": \"{plugin_name}\",\n  \"Version\": \"0.1.0\",\n  \"Factory Info\": {{\n    \"Vendor\": \"Signal\",\n    \"URL\": \"https://signal.dev\",\n    \"E-Mail\": \"\"\n  }},\n  \"Classes\": [\n    {{\n      \"CID\": \"{VST3_FIXTURE_CLASS_ID_HEX}\",\n      \"Category\": \"Audio Module Class\",\n      \"Name\": \"{plugin_name}\",\n      \"Vendor\": \"Signal\",\n      \"Version\": \"0.1.0\",\n      \"Sub Categories\": [\"Fx\"]\n    }}\n  ]\n}}\n"
    )
}
