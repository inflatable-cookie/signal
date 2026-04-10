use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) struct TempPluginScanRoot {
    path: PathBuf,
}

impl TempPluginScanRoot {
    pub(crate) fn root(&self) -> String {
        self.path.display().to_string()
    }
}

impl Drop for TempPluginScanRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn temp_local_vst3_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("vst3");
    write_vst3_bundle(&root, "Signal Instrument.vst3", "plugin:vst3:instrument");
    write_vst3_bundle(
        &root,
        "Signal Multi Output Instrument.vst3",
        "plugin:vst3:multiout-instrument",
    );
    write_vst3_bundle(&root, "Signal Utility.vst3", "plugin:vst3:utility");
    write_vst3_bundle(&root, "Signal Bus FX.vst3", "plugin:vst3:bus-fx");
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_local_au_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("au");
    write_au_bundle(&root, "Signal Instrument.component", "plugin:au:instrument");
    write_au_bundle(
        &root,
        "Signal Multi Output Instrument.component",
        "plugin:au:multiout-instrument",
    );
    write_au_bundle(&root, "Signal Utility.component", "plugin:au:utility");
    write_au_bundle(&root, "Signal Bus FX.component", "plugin:au:bus-fx");
    TempPluginScanRoot { path: root }
}

fn temp_scan_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("signal-host-local-{label}-scan-{nanos}"));
    fs::create_dir_all(&path).expect("temp plugin scan root should be created");
    path
}

fn write_vst3_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
        .expect("local vst3 resources should be created");
    fs::write(
        bundle_root.join("Contents").join("Info.plist"),
        vst3_info_plist_contents(vst3_metadata_contents(plugin_type_id), &bundle_root),
    )
    .expect("local vst3 info plist should be written");
    fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("moduleinfo.json"),
        vst3_moduleinfo_contents(vst3_metadata_contents(plugin_type_id)),
    )
    .expect("local vst3 moduleinfo should be written");
}

fn write_au_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(bundle_root.join("Contents")).expect("local au contents should be created");
    fs::write(
        bundle_root.join("Contents").join("Info.plist"),
        au_info_plist_contents(au_metadata_contents(plugin_type_id)),
    )
    .expect("local au info plist should be written");
}

fn vst3_metadata_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:instrument" => {
            "plugin_type_id=plugin:vst3:instrument\nclass_id=7E1D8F8A4D874D56A2C44DE250100001\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100002\ncategory=Instrument\nvendor=Signal\nname=Signal Instrument VST3 Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=2\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        "plugin:vst3:multiout-instrument" => {
            "plugin_type_id=plugin:vst3:multiout-instrument\nclass_id=7E1D8F8A4D874D56A2C44DE250100011\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100012\ncategory=Instrument\nvendor=Signal\nname=Signal Multi Output Instrument VST3 Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=6\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        "plugin:vst3:utility" => {
            "plugin_type_id=plugin:vst3:utility\nclass_id=7E1D8F8A4D874D56A2C44DE250100201\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100202\ncategory=Fx\nvendor=Signal\nname=Signal Utility VST3 Plugin\nversion=0.1.0\naudio_inputs=2\naudio_outputs=2\nmidi_inputs=0\nmidi_outputs=0\nfeatures=AudioEffect,Utility\n"
        }
        "plugin:vst3:bus-fx" => {
            "plugin_type_id=plugin:vst3:bus-fx\nclass_id=7E1D8F8A4D874D56A2C44DE250100211\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100212\ncategory=Fx\nvendor=Signal\nname=Signal Bus FX VST3 Plugin\nversion=0.1.0\naudio_inputs=4\naudio_outputs=4\nmidi_inputs=0\nmidi_outputs=0\nfeatures=AudioEffect,Utility\n"
        }
        other => panic!("unknown local VST3 plugin type: {other}"),
    }
}

fn vst3_info_plist_contents(metadata: &str, bundle_root: &PathBuf) -> String {
    let mut plugin_type_id = "";
    let mut name = "Signal VST3 Plugin";
    let mut version = "0.1.0";
    let mut audio_inputs = "2";
    let mut audio_outputs = "2";
    let mut midi_inputs = "0";
    let mut midi_outputs = "0";
    let mut features = "";

    for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "plugin_type_id" => plugin_type_id = value.trim(),
            "name" => name = value.trim(),
            "version" => version = value.trim(),
            "audio_inputs" => audio_inputs = value.trim(),
            "audio_outputs" => audio_outputs = value.trim(),
            "midi_inputs" => midi_inputs = value.trim(),
            "midi_outputs" => midi_outputs = value.trim(),
            "features" => features = value.trim(),
            _ => {}
        }
    }

    let executable_name = bundle_root
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name);
    let feature_array = features
        .split(',')
        .map(str::trim)
        .filter(|feature| !feature.is_empty())
        .map(|feature| format!("    <string>{feature}</string>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{executable_name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>CFBundleName</key>\n\
  <string>{name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>{version}</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>{audio_inputs}</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>{audio_outputs}</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>{midi_inputs}</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>{midi_outputs}</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
{feature_array}\n\
  </array>\n\
</dict>\n\
</plist>\n"
    )
}

fn vst3_moduleinfo_contents(metadata: &str) -> String {
    let mut class_id = "";
    let mut controller_class_id = "";
    let mut category = "Fx";
    let mut vendor = "Signal";
    let mut name = "Signal VST3 Plugin";
    let mut version = "0.1.0";

    for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "class_id" => class_id = value.trim(),
            "controller_class_id" => controller_class_id = value.trim(),
            "category" => category = value.trim(),
            "vendor" => vendor = value.trim(),
            "name" => name = value.trim(),
            "version" => version = value.trim(),
            _ => {}
        }
    }

    let subcategory = if category.eq_ignore_ascii_case("Instrument") {
        "Instrument"
    } else {
        "Fx"
    };
    let controller_class = if controller_class_id.is_empty()
        || controller_class_id.eq_ignore_ascii_case("none")
    {
        String::new()
    } else {
        format!(
            ",\n    {{\n      \"CID\": \"{controller_class_id}\",\n      \"Category\": \"Component Controller Class\",\n      \"Name\": \"{name}\",\n      \"Vendor\": \"{vendor}\",\n      \"Version\": \"{version}\",\n      \"Sub Categories\": [\"{subcategory}\"]\n    }}"
        )
    };

    format!(
        "{{\n  \"Name\": \"{name}\",\n  \"Version\": \"{version}\",\n  \"Factory Info\": {{\n    \"Vendor\": \"{vendor}\",\n    \"URL\": \"https://signal.dev\",\n    \"E-Mail\": \"\"\n  }},\n  \"Classes\": [\n    {{\n      \"CID\": \"{class_id}\",\n      \"Category\": \"Audio Module Class\",\n      \"Name\": \"{name}\",\n      \"Vendor\": \"{vendor}\",\n      \"Version\": \"{version}\",\n      \"Sub Categories\": [\"{subcategory}\"]\n    }}{controller_class}\n  ]\n}}\n"
    )
}

fn au_metadata_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:au:instrument" => {
            "plugin_type_id=plugin:au:instrument\ncomponent_type=aumu\ncomponent_subtype=sigi\nmanufacturer_code=sigl\nvendor=Signal\nname=Signal Instrument AU Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=2\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        "plugin:au:multiout-instrument" => {
            "plugin_type_id=plugin:au:multiout-instrument\ncomponent_type=aumu\ncomponent_subtype=sigm\nmanufacturer_code=sigl\nvendor=Signal\nname=Signal Multi Output Instrument AU Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=6\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        "plugin:au:utility" => {
            "plugin_type_id=plugin:au:utility\ncomponent_type=aufx\ncomponent_subtype=sigu\nmanufacturer_code=sigl\nvendor=Signal\nname=Signal Utility AU Plugin\nversion=0.1.0\naudio_inputs=2\naudio_outputs=2\nmidi_inputs=0\nmidi_outputs=0\nfeatures=AudioEffect,Utility\n"
        }
        "plugin:au:bus-fx" => {
            "plugin_type_id=plugin:au:bus-fx\ncomponent_type=aufx\ncomponent_subtype=sigb\nmanufacturer_code=sigl\nvendor=Signal\nname=Signal Bus FX AU Plugin\nversion=0.1.0\naudio_inputs=4\naudio_outputs=4\nmidi_inputs=0\nmidi_outputs=0\nfeatures=AudioEffect,Utility\n"
        }
        other => panic!("unknown local AU plugin type: {other}"),
    }
}

fn au_info_plist_contents(metadata: &str) -> String {
    let mut plugin_type_id = "";
    let mut component_type = "";
    let mut component_subtype = "";
    let mut manufacturer_code = "";
    let mut vendor = "Signal";
    let mut name = "Signal AU Plugin";
    let mut version = "0.1.0";
    let mut audio_inputs = "2";
    let mut audio_outputs = "2";
    let mut midi_inputs = "0";
    let mut midi_outputs = "0";
    let mut features = "";

    for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "plugin_type_id" => plugin_type_id = value.trim(),
            "component_type" => component_type = value.trim(),
            "component_subtype" => component_subtype = value.trim(),
            "manufacturer_code" => manufacturer_code = value.trim(),
            "vendor" => vendor = value.trim(),
            "name" => name = value.trim(),
            "version" => version = value.trim(),
            "audio_inputs" => audio_inputs = value.trim(),
            "audio_outputs" => audio_outputs = value.trim(),
            "midi_inputs" => midi_inputs = value.trim(),
            "midi_outputs" => midi_outputs = value.trim(),
            "features" => features = value.trim(),
            _ => {}
        }
    }

    let feature_array = features
        .split(',')
        .map(str::trim)
        .filter(|feature| !feature.is_empty())
        .map(|feature| format!("    <string>{feature}</string>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>AudioComponents</key>\n\
  <array>\n\
    <dict>\n\
      <key>manufacturer</key>\n\
      <string>{manufacturer_code}</string>\n\
      <key>name</key>\n\
      <string>{vendor}: {name}</string>\n\
      <key>sandboxSafe</key>\n\
      <false/>\n\
      <key>subtype</key>\n\
      <string>{component_subtype}</string>\n\
      <key>type</key>\n\
      <string>{component_type}</string>\n\
      <key>version</key>\n\
      <integer>1</integer>\n\
    </dict>\n\
  </array>\n\
  <key>CFBundleExecutable</key>\n\
  <string>{name}</string>\n\
  <key>CFBundleIdentifier</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>CFBundleName</key>\n\
  <string>{name}</string>\n\
  <key>CFBundlePackageType</key>\n\
  <string>BNDL</string>\n\
  <key>CFBundleShortVersionString</key>\n\
  <string>{version}</string>\n\
  <key>SignalPluginTypeId</key>\n\
  <string>{plugin_type_id}</string>\n\
  <key>SignalVendor</key>\n\
  <string>{vendor}</string>\n\
  <key>SignalDisplayName</key>\n\
  <string>{name}</string>\n\
  <key>SignalAudioInputs</key>\n\
  <integer>{audio_inputs}</integer>\n\
  <key>SignalAudioOutputs</key>\n\
  <integer>{audio_outputs}</integer>\n\
  <key>SignalMidiInputs</key>\n\
  <integer>{midi_inputs}</integer>\n\
  <key>SignalMidiOutputs</key>\n\
  <integer>{midi_outputs}</integer>\n\
  <key>SignalFeatures</key>\n\
  <array>\n\
{feature_array}\n\
  </array>\n\
</dict>\n\
</plist>\n"
    )
}
