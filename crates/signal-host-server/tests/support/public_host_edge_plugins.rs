#![allow(dead_code)]

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

pub(crate) fn temp_public_server_vst3_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("vst3");
    write_vst3_bundle(&root, "Signal Linux Synth.vst3", "plugin:vst3:linux-synth");
    write_vst3_bundle(
        &root,
        "Signal Multi Output Instrument.vst3",
        "plugin:vst3:multiout-instrument",
    );
    write_vst3_bundle(&root, "Signal Utility.vst3", "plugin:vst3:utility");
    write_vst3_bundle(&root, "Signal Bus FX.vst3", "plugin:vst3:bus-fx");
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_public_server_au_scan_root() -> TempPluginScanRoot {
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

pub(crate) fn temp_public_server_lv2_scan_root() -> TempPluginScanRoot {
    let root = temp_scan_root("lv2");
    write_lv2_bundle(&root, "Signal Linux Synth.lv2", "plugin:lv2:linux-synth");
    write_lv2_bundle(
        &root,
        "Signal Multi Output Instrument.lv2",
        "plugin:lv2:multiout-instrument",
    );
    write_lv2_bundle(&root, "Signal Utility.lv2", "plugin:lv2:utility");
    write_lv2_bundle(&root, "Signal Bus FX.lv2", "plugin:lv2:bus-fx");
    write_manifest_bundle(
        &root,
        "Broken Manifest.lv2",
        "@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:broken-public\"\n",
    );
    write_manifest_bundle(
        &root,
        "Unsupported Feature.lv2",
        "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:unsupported-public\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/unsupported-public\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Unsupported Public LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"2\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/atom#sequence\" .\nsignal:feature \"AudioEffect\" .\n",
    );
    write_manifest_bundle(
        &root,
        "Worker Unavailable.lv2",
        "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:worker-unavailable-public\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/worker-unavailable-public\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Worker Unavailable Public LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"2\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"1\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:prepare_fault \"worker_unavailable\" .\nsignal:feature \"AudioEffect\" .\n",
    );
    TempPluginScanRoot { path: root }
}

fn temp_scan_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("signal-public-host-server-{label}-scan-{unique}"));
    fs::create_dir_all(&root).expect("public server scan root should be created");
    root
}

fn write_vst3_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
        .expect("public server vst3 resources should be created");
    fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-vst3-module.txt"),
        vst3_metadata_contents(plugin_type_id),
    )
    .expect("public server vst3 metadata should be written");
    fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-vst3-factory.txt"),
        vst3_factory_contents(plugin_type_id),
    )
    .expect("public server vst3 factory metadata should be written");
}

fn write_au_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
        .expect("public server au resources should be created");
    fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-au-component.txt"),
        au_metadata_contents(plugin_type_id),
    )
    .expect("public server au metadata should be written");
}

fn write_lv2_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(&bundle_root).expect("public server lv2 bundle should be created");
    fs::write(
        bundle_root.join("manifest.ttl"),
        lv2_manifest_contents(plugin_type_id),
    )
    .expect("public server lv2 manifest should be written");
}

fn write_manifest_bundle(root: &PathBuf, bundle: &str, manifest: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(&bundle_root).expect("public server lv2 bundle should be created");
    fs::write(bundle_root.join("manifest.ttl"), manifest)
        .expect("public server lv2 manifest should be written");
}

fn vst3_metadata_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:linux-synth" => {
            "plugin_type_id=plugin:vst3:linux-synth\nclass_id=7E1D8F8A4D874D56A2C44DE250100101\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100102\ncategory=Instrument\nvendor=Signal\nname=Signal Linux Synth VST3 Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=2\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
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
        other => panic!("unknown server public VST3 plugin type: {other}"),
    }
}

fn vst3_factory_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:linux-synth" => {
            "component=7E1D8F8A4D874D56A2C44DE250100101|Instrument|Signal Linux Synth VST3 Plugin\ncontroller=7E1D8F8A4D874D56A2C44DE250100102|Controller|Signal Linux Synth VST3 Plugin\n"
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
        other => panic!("unknown server public VST3 factory type: {other}"),
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
        other => panic!("unknown server public AU plugin type: {other}"),
    }
}

fn lv2_manifest_contents(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:lv2:linux-synth" => {
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:linux-synth\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/linux-synth\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Linux Synth LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"0\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"1\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/state#state\" .\nsignal:feature \"Instrument\" .\nsignal:feature \"Analyzer\" .\n"
        }
        "plugin:lv2:multiout-instrument" => {
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:multiout-instrument\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/multiout-instrument\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Multi Output Instrument LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"0\" .\nsignal:audio_outputs \"6\" .\nsignal:midi_inputs \"1\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/worker#schedule\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/state#state\" .\nsignal:feature \"Instrument\" .\nsignal:feature \"Analyzer\" .\n"
        }
        "plugin:lv2:utility" => {
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:utility\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/utility\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Utility LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"2\" .\nsignal:audio_outputs \"2\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/options#options\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/options#options\" .\nsignal:feature \"AudioEffect\" .\nsignal:feature \"Utility\" .\n"
        }
        "plugin:lv2:bus-fx" => {
            "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n@prefix signal: <https://signal.dev/ns/lv2#> .\nsignal:plugin_type_id \"plugin:lv2:bus-fx\" .\nsignal:plugin_uri \"https://signal.dev/plugins/lv2/bus-fx\" .\nsignal:vendor \"Signal\" .\nsignal:name \"Signal Bus FX LV2 Plugin\" .\nsignal:version \"0.1.0\" .\nsignal:audio_inputs \"4\" .\nsignal:audio_outputs \"4\" .\nsignal:midi_inputs \"0\" .\nsignal:midi_outputs \"0\" .\nsignal:required_feature \"http://lv2plug.in/ns/ext/urid#map\" .\nsignal:supported_extension \"http://lv2plug.in/ns/ext/patch#Message\" .\nsignal:feature \"AudioEffect\" .\nsignal:feature \"Utility\" .\n"
        }
        other => panic!("unknown server public LV2 plugin type: {other}"),
    }
}
