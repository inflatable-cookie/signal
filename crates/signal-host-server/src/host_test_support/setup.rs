use crate::host::host_support::{server_demo_runtime_assembly, LifecycleRunSummary};
use crate::host::ServerRuntimeHost;
use signal_plugin::PluginFormat;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    BackendPolicyOverride, HandshakeRequest, PluginScanRequest, RuntimeConfig,
    RuntimeConfigRequest, RuntimeLifecycleApi, RuntimeProjectionApi, RuntimeSupervisorApi,
    SignalRuntime,
};
use std::{
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
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn temp_server_vst3_scan_root() -> TempPluginScanRoot {
    let root = temp_plugin_scan_dir("vst3");
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

pub(crate) fn temp_server_au_scan_root() -> TempPluginScanRoot {
    let root = temp_plugin_scan_dir("au");
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

pub(crate) fn temp_server_lv2_scan_root() -> TempPluginScanRoot {
    let root = temp_plugin_scan_dir("lv2");
    write_lv2_bundle(&root, "Signal Linux Synth.lv2", "plugin:lv2:linux-synth");
    write_lv2_bundle(
        &root,
        "Signal Multi Output Instrument.lv2",
        "plugin:lv2:multiout-instrument",
    );
    write_lv2_bundle(&root, "Signal Utility.lv2", "plugin:lv2:utility");
    write_lv2_bundle(&root, "Signal Bus FX.lv2", "plugin:lv2:bus-fx");
    TempPluginScanRoot { path: root }
}

pub(crate) fn temp_media_fixture_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for tests")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "signal-host-server-{label}-{}-{unique}.bin",
        std::process::id()
    ))
}

fn temp_plugin_scan_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for tests")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "signal-host-server-{label}-scan-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("temp plugin scan root should be created");
    root
}

fn write_vst3_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    std::fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
        .expect("server vst3 resources should be created");
    std::fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-vst3-module.txt"),
        vst3_metadata_contents(plugin_type_id),
    )
    .expect("server vst3 metadata should be written");
    std::fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-vst3-factory.txt"),
        vst3_factory_contents(plugin_type_id),
    )
    .expect("server vst3 factory metadata should be written");
}

fn write_au_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    std::fs::create_dir_all(bundle_root.join("Contents").join("Resources"))
        .expect("server au resources should be created");
    std::fs::write(
        bundle_root
            .join("Contents")
            .join("Resources")
            .join("signal-au-component.txt"),
        au_metadata_contents(plugin_type_id),
    )
    .expect("server au metadata should be written");
}

fn write_lv2_bundle(root: &PathBuf, bundle: &str, plugin_type_id: &str) {
    let bundle_root = root.join(bundle);
    std::fs::create_dir_all(&bundle_root).expect("server lv2 bundle should be created");
    std::fs::write(
        bundle_root.join("manifest.ttl"),
        lv2_manifest_contents(plugin_type_id),
    )
    .expect("server lv2 manifest should be written");
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
        other => panic!("unknown server VST3 plugin type: {other}"),
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
        other => panic!("unknown server VST3 factory type: {other}"),
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
        other => panic!("unknown server AU plugin type: {other}"),
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
        other => panic!("unknown server LV2 plugin type: {other}"),
    }
}

pub(crate) fn prepare_server_host_with_lifecycle() -> (
    ServerRuntimeHost,
    ClapBlockProtocol,
    ClapSandboxLifecycleHarness,
    LifecycleRunSummary,
) {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let mut runtime_config = RuntimeConfigRequest::new(
        host.runtime.config().sample_rate.0,
        host.runtime.config().graph.block_size,
    );
    runtime_config.anticipative_enabled = false;
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-server".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })
        .expect("handshake");
    host.runtime.configure(runtime_config).expect("configure");
    let assembly = server_demo_runtime_assembly();
    host.runtime
        .apply_graph_projection(assembly.graph.clone())
        .expect("graph projection");

    let hardware_request = signal_hardware::HardwareConfigRequest::new(
        host.runtime.config().sample_rate.0,
        host.runtime.config().graph.block_size,
        signal_hardware::BackendPolicyTier::Tier0InHost,
    );
    host.runtime
        .apply_hardware_config(hardware_request)
        .expect("hardware config");
    host.runtime
        .set_active_output_device("server:virtual-output");
    host.set_backend_policy(BackendPolicyOverride {
        tier: hardware_request.backend_policy,
    })
    .expect("backend policy");
    host.runtime
        .set_backend_policy_tier(hardware_request.backend_policy);
    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["/srv/plugins/clap".into()],
        formats: vec![PluginFormat::Clap],
    })
    .expect("plugin scan");
    for sandbox in &assembly.plugin_sandboxes {
        host.ensure_plugin_sandbox(sandbox.spec())
            .expect("ensure sandbox");
    }
    host.runtime
        .apply_plugin_backed_node_bindings(assembly.plugin_bindings())
        .expect("plugin bindings");
    host.runtime
        .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());
    host.runtime.set_cpu_load_percent(1.2);
    host.runtime.set_graph_latency_ms(1.1);
    host.runtime.start().expect("start runtime");

    let protocol = ClapBlockProtocol::new(
        "plugin:clap:server",
        "instance:server:default",
        signal_plugin::PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        2048,
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let sandbox = assembly.primary_sandbox();
    let run = host
        .run_lifecycle(
            &protocol,
            sandbox.request.sandbox_id.as_str(),
            1,
            &mut lifecycle,
        )
        .expect("lifecycle");
    (host, protocol, lifecycle, run)
}

pub(crate) fn prepare_server_host_without_lifecycle() -> (ServerRuntimeHost, ClapBlockProtocol) {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let mut runtime_config = RuntimeConfigRequest::new(
        host.runtime.config().sample_rate.0,
        host.runtime.config().graph.block_size,
    );
    runtime_config.anticipative_enabled = false;
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-server".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })
        .expect("handshake");
    host.runtime.configure(runtime_config).expect("configure");
    let assembly = server_demo_runtime_assembly();
    host.runtime
        .apply_graph_projection(assembly.graph.clone())
        .expect("graph projection");

    let hardware_request = signal_hardware::HardwareConfigRequest::new(
        host.runtime.config().sample_rate.0,
        host.runtime.config().graph.block_size,
        signal_hardware::BackendPolicyTier::Tier0InHost,
    );
    host.runtime
        .apply_hardware_config(hardware_request)
        .expect("hardware config");
    host.runtime
        .set_active_output_device("server:virtual-output");
    host.set_backend_policy(BackendPolicyOverride {
        tier: hardware_request.backend_policy,
    })
    .expect("backend policy");
    host.runtime
        .set_backend_policy_tier(hardware_request.backend_policy);
    host.start_plugin_scan(PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        formats: vec![PluginFormat::Clap],
    })
    .expect("plugin scan");
    for sandbox in &assembly.plugin_sandboxes {
        host.ensure_plugin_sandbox(sandbox.spec())
            .expect("ensure sandbox");
    }
    host.runtime
        .apply_plugin_backed_node_bindings(assembly.plugin_bindings())
        .expect("plugin bindings");
    host.runtime
        .set_active_plugin_sandboxes(assembly.active_plugin_sandbox_count());
    host.runtime.set_cpu_load_percent(3.2);
    host.runtime.set_graph_latency_ms(1.1);
    host.runtime.start().expect("start runtime");

    let protocol = ClapBlockProtocol::new(
        "plugin:clap:server",
        "instance:server:default",
        signal_plugin::PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        2048,
    );
    (host, protocol)
}
