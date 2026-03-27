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
