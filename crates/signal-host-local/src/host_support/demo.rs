use signal_plugin::{
    BlockPayload, ParameterValueEvent, PluginEvent, PluginFormat, PluginSandboxRequest,
    SandboxPolicy, WatchdogTriggerReason,
};
use signal_runtime::{
    PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxSpec,
    RuntimeWatchdogTrigger, TransportAttachIntent,
};
use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use super::super::LOCAL_DEMO_PLUGIN_NODE_ID;
use super::demo_graph::{local_demo_graph_contract_projection, local_demo_graph_projection};

#[derive(Clone, Debug)]
pub(crate) struct LocalDemoPluginSandboxAssembly {
    pub(crate) request: PluginSandboxRequest,
    pub(crate) plugin_format: PluginFormat,
    pub(crate) plugin_type_id: Option<String>,
    pub(crate) bound_node_ids: Vec<&'static str>,
}

impl LocalDemoPluginSandboxAssembly {
    pub(crate) fn spec(&self) -> PluginSandboxSpec {
        PluginSandboxSpec {
            sandbox_id: self.request.sandbox_id.clone(),
            plugin_format: self.plugin_format,
            plugin_type_id: self.plugin_type_id.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LocalDemoRuntimeAssembly {
    pub(crate) graph: signal_runtime::GraphProjection,
    pub(crate) graph_contracts: signal_runtime::GraphContractProjection,
    pub(crate) scan_roots: Vec<String>,
    pub(crate) scan_formats: Vec<PluginFormat>,
    pub(crate) plugin_sandboxes: Vec<LocalDemoPluginSandboxAssembly>,
}

impl LocalDemoRuntimeAssembly {
    pub(crate) fn primary_sandbox(&self) -> &LocalDemoPluginSandboxAssembly {
        self.plugin_sandboxes
            .first()
            .expect("local demo assembly should define a primary sandbox")
    }

    pub(crate) fn active_plugin_sandbox_count(&self) -> u32 {
        self.plugin_sandboxes.len() as u32
    }

    pub(crate) fn plugin_bindings(&self) -> PluginBackedNodeBindingProjection {
        PluginBackedNodeBindingProjection {
            graph_id: self.graph.graph_id.clone(),
            bindings: self
                .plugin_sandboxes
                .iter()
                .flat_map(|sandbox| {
                    sandbox
                        .bound_node_ids
                        .iter()
                        .map(|node_id| PluginBackedNodeBinding {
                            node_id: (*node_id).into(),
                            sandbox_id: sandbox.request.sandbox_id.clone(),
                        })
                })
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DemoInteractionMode {
    ParameterStep,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DemoInteractionStep {
    pub(crate) mode: DemoInteractionMode,
    pub(crate) value: f32,
}

pub(crate) fn local_demo_runtime_assembly() -> LocalDemoRuntimeAssembly {
    let broker_override = broker_demo_plugin_override();
    let graph = local_demo_graph_projection();
    LocalDemoRuntimeAssembly {
        graph_contracts: local_demo_graph_contract_projection(&graph.graph_id),
        graph,
        scan_roots: broker_override
            .as_ref()
            .map(|override_spec| vec![override_spec.scan_root.clone()])
            .unwrap_or_else(|| vec!["~/Library/Audio/Plug-Ins/CLAP".into()]),
        scan_formats: vec![broker_override
            .as_ref()
            .map(|override_spec| override_spec.plugin_format)
            .unwrap_or(PluginFormat::Clap)],
        plugin_sandboxes: vec![LocalDemoPluginSandboxAssembly {
            request: PluginSandboxRequest::new(
                "local-default-sandbox",
                broker_override
                    .as_ref()
                    .map(|override_spec| override_spec.plugin_format)
                    .unwrap_or(PluginFormat::Clap),
                SandboxPolicy::Strict,
            ),
            plugin_format: broker_override
                .as_ref()
                .map(|override_spec| override_spec.plugin_format)
                .unwrap_or(PluginFormat::Clap),
            plugin_type_id: broker_override
                .as_ref()
                .map(|override_spec| override_spec.plugin_type_id.clone())
                .or_else(|| Some("plugin:clap:default".into())),
            bound_node_ids: vec![LOCAL_DEMO_PLUGIN_NODE_ID],
        }],
    }
}

pub struct DemoBootstrapGuard {
    root: Option<PathBuf>,
    old_demo_format: Option<OsString>,
    old_demo_root: Option<OsString>,
    old_demo_plugin_type_id: Option<OsString>,
}

impl Drop for DemoBootstrapGuard {
    fn drop(&mut self) {
        restore_demo_env(
            "SIGNAL_HOST_DEMO_PLUGIN_FORMAT",
            self.old_demo_format.as_ref(),
        );
        restore_demo_env("SIGNAL_HOST_DEMO_PLUGIN_ROOT", self.old_demo_root.as_ref());
        restore_demo_env(
            "SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID",
            self.old_demo_plugin_type_id.as_ref(),
        );
        if let Some(root) = self.root.as_ref() {
            let _ = fs::remove_dir_all(root);
        }
    }
}

pub fn ensure_default_demo_plugin_override() -> DemoBootstrapGuard {
    if broker_demo_plugin_override().is_some() {
        return DemoBootstrapGuard {
            root: None,
            old_demo_format: None,
            old_demo_root: None,
            old_demo_plugin_type_id: None,
        };
    }

    let root = temp_demo_scan_root("local-host-demo-vst3");
    write_demo_vst3_bundle(&root, "Signal Instrument.vst3", "plugin:vst3:instrument");
    let old_demo_format = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_FORMAT");
    let old_demo_root = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_ROOT");
    let old_demo_plugin_type_id = std::env::var_os("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID");

    unsafe {
        std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_FORMAT", "vst3");
        std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_ROOT", root.display().to_string());
        std::env::set_var("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID", "plugin:vst3:instrument");
    }

    DemoBootstrapGuard {
        root: Some(root),
        old_demo_format,
        old_demo_root,
        old_demo_plugin_type_id,
    }
}

#[derive(Clone, Debug)]
struct BrokerDemoPluginOverride {
    plugin_format: PluginFormat,
    plugin_type_id: String,
    scan_root: String,
}

fn broker_demo_plugin_override() -> Option<BrokerDemoPluginOverride> {
    let plugin_format = std::env::var("SIGNAL_HOST_DEMO_PLUGIN_FORMAT")
        .ok()
        .and_then(|value| parse_demo_plugin_format(&value))?;
    let plugin_type_id = std::env::var("SIGNAL_HOST_DEMO_PLUGIN_TYPE_ID").ok()?;
    let scan_root = std::env::var("SIGNAL_HOST_DEMO_PLUGIN_ROOT").ok()?;
    Some(BrokerDemoPluginOverride {
        plugin_format,
        plugin_type_id,
        scan_root,
    })
}

fn parse_demo_plugin_format(value: &str) -> Option<PluginFormat> {
    match value {
        "clap" => Some(PluginFormat::Clap),
        "vst3" => Some(PluginFormat::Vst3),
        "au" => Some(PluginFormat::Au),
        "lv2" => Some(PluginFormat::Lv2),
        _ => None,
    }
}

fn temp_demo_scan_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("signal-{label}-{unique}"));
    fs::create_dir_all(&root).expect("demo scan root should be created");
    root
}

fn write_demo_vst3_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
        .expect("demo VST3 resources should be created");
    fs::write(
        bundle_root.join("Contents").join("Info.plist"),
        demo_vst3_info_plist_contents(demo_vst3_module_metadata(plugin_type_id), &bundle_root),
    )
    .expect("demo VST3 info plist should be written");
    fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("moduleinfo.json"),
        demo_vst3_moduleinfo_contents(demo_vst3_module_metadata(plugin_type_id)),
    )
    .expect("demo VST3 moduleinfo should be written");
}

fn demo_vst3_module_metadata(plugin_type_id: &str) -> &'static str {
    match plugin_type_id {
        "plugin:vst3:instrument" => {
            "plugin_type_id=plugin:vst3:instrument\nclass_id=7E1D8F8A4D874D56A2C44DE250100001\ncontroller_class_id=7E1D8F8A4D874D56A2C44DE250100002\ncategory=Instrument\nvendor=Signal\nname=Signal Instrument VST3 Plugin\nversion=0.1.0\naudio_inputs=0\naudio_outputs=2\nmidi_inputs=1\nmidi_outputs=0\nfeatures=Instrument,Analyzer\n"
        }
        other => panic!("unknown local demo VST3 plugin type: {other}"),
    }
}

fn demo_vst3_info_plist_contents(metadata: &str, bundle_root: &PathBuf) -> String {
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

fn demo_vst3_moduleinfo_contents(metadata: &str) -> String {
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

fn restore_demo_env(key: &str, value: Option<&OsString>) {
    unsafe {
        if let Some(value) = value {
            std::env::set_var(key, value);
        } else {
            std::env::remove_var(key);
        }
    }
}

pub(crate) fn plugin_automation_value_from_runtime_batch(
    automation_parameter_id: u32,
    parameter_batch: Option<&signal_runtime::ParameterBatch>,
) -> Option<ParameterValueEvent> {
    let parameter_batch = parameter_batch?;
    let value = parameter_batch.events.last()?.normalized_value;
    Some(ParameterValueEvent {
        offset_frames: 0,
        parameter_id: automation_parameter_id,
        normalized_value: value,
    })
}

pub(crate) fn demo_interaction_parameter_step(
    automation_parameter_id: u32,
) -> Option<ParameterValueEvent> {
    let step = demo_interaction_step()?;
    Some(ParameterValueEvent {
        offset_frames: 0,
        parameter_id: automation_parameter_id,
        normalized_value: step.value,
    })
}

pub(crate) fn demo_interaction_step() -> Option<DemoInteractionStep> {
    let mode = std::env::var("SIGNAL_HOST_DEMO_INTERACTION_MODE").ok()?;
    let value = std::env::var("SIGNAL_HOST_DEMO_INTERACTION_VALUE")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .map(|value| value.clamp(0.0, 1.0))
        .unwrap_or(0.75);
    match mode.as_str() {
        "parameter-step" => Some(DemoInteractionStep {
            mode: DemoInteractionMode::ParameterStep,
            value,
        }),
        _ => None,
    }
}

pub(crate) fn payload_automation_value(
    payload: &BlockPayload,
    automation_parameter_id: u32,
) -> Option<f32> {
    payload.events.events.iter().find_map(|event| match event {
        PluginEvent::ParameterValue(event) if event.parameter_id == automation_parameter_id => {
            Some(event.normalized_value)
        }
        _ => None,
    })
}

pub(crate) fn runtime_watchdog_trigger(reason: WatchdogTriggerReason) -> RuntimeWatchdogTrigger {
    match reason {
        WatchdogTriggerReason::DeadlineMisses => RuntimeWatchdogTrigger::DeadlineMisses,
        WatchdogTriggerReason::HeartbeatMisses => RuntimeWatchdogTrigger::HeartbeatMisses,
    }
}

pub(crate) fn transport_attach_intent(processing_epoch: u64) -> TransportAttachIntent {
    if processing_epoch > 1 {
        TransportAttachIntent::RecoveryOverlap
    } else {
        TransportAttachIntent::SteadyState
    }
}
