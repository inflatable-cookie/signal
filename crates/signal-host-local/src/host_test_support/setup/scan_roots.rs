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
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-vst3-module.txt"),
        vst3_metadata_contents(plugin_type_id),
    )
    .expect("local vst3 metadata should be written");
    fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-vst3-factory.txt"),
        vst3_factory_contents(plugin_type_id),
    )
    .expect("local vst3 factory metadata should be written");
}

fn write_au_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
        .expect("local au resources should be created");
    fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-au-component.txt"),
        au_metadata_contents(plugin_type_id),
    )
    .expect("local au metadata should be written");
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

fn vst3_factory_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:instrument" => {
            "component=7E1D8F8A4D874D56A2C44DE250100001|Instrument|Signal Instrument VST3 Plugin\ncontroller=7E1D8F8A4D874D56A2C44DE250100002|Controller|Signal Instrument VST3 Plugin\n"
        }
        "plugin:vst3:multiout-instrument" => {
            "component=7E1D8F8A4D874D56A2C44DE250100011|Instrument|Signal Multi Output Instrument VST3 Plugin\ncontroller=7E1D8F8A4D874D56A2C44DE250100012|Controller|Signal Multi Output Instrument VST3 Plugin\n"
        }
        "plugin:vst3:utility" => {
            "component=7E1D8F8A4D874D56A2C44DE250100201|Fx|Signal Utility VST3 Plugin\ncontroller=7E1D8F8A4D874D56A2C44DE250100202|Controller|Signal Utility VST3 Plugin\n"
        }
        "plugin:vst3:bus-fx" => {
            "component=7E1D8F8A4D874D56A2C44DE250100211|Fx|Signal Bus FX VST3 Plugin\ncontroller=7E1D8F8A4D874D56A2C44DE250100212|Controller|Signal Bus FX VST3 Plugin\n"
        }
        other => panic!("unknown local VST3 factory type: {other}"),
    }
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
