use super::super::super::{local_demo_runtime_assembly, LifecycleRunSummary, LocalRuntimeHost};
use signal_plugin::PluginFormat;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi,
    RuntimeProjectionApi, RuntimeSupervisorApi, SignalRuntime,
};

pub(crate) fn prepare_local_host_with_lifecycle() -> (
    LocalRuntimeHost,
    ClapBlockProtocol,
    ClapSandboxLifecycleHarness,
    LifecycleRunSummary,
) {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let runtime_config = RuntimeConfigRequest::new(
        host.runtime.config().sample_rate.0,
        host.runtime.config().graph.block_size,
    );
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-local".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })
        .expect("handshake");
    host.runtime.configure(runtime_config).expect("configure");
    let assembly = local_demo_runtime_assembly();
    host.runtime
        .apply_graph_projection(assembly.graph.clone())
        .expect("graph projection");
    host.runtime
        .apply_graph_contract_projection(assembly.graph_contracts.clone())
        .expect("graph contract projection");

    host.prepare_default_output_hardware()
        .expect("hardware config");
    host.start_plugin_scan(signal_runtime::PluginScanRequest {
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
    host.runtime.set_cpu_load_percent(4.5);
    host.runtime.set_graph_latency_ms(2.7);
    host.runtime.start().expect("start runtime");

    let protocol = ClapBlockProtocol::new(
        "plugin:clap:default",
        "instance:local:default",
        signal_plugin::PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
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

pub(crate) fn prepare_local_host_without_lifecycle() -> (LocalRuntimeHost, ClapBlockProtocol) {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let runtime_config = RuntimeConfigRequest::new(
        host.runtime.config().sample_rate.0,
        host.runtime.config().graph.block_size,
    );
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-local".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })
        .expect("handshake");
    host.runtime.configure(runtime_config).expect("configure");
    let assembly = local_demo_runtime_assembly();
    host.runtime
        .apply_graph_projection(assembly.graph.clone())
        .expect("graph projection");
    host.runtime
        .apply_graph_contract_projection(assembly.graph_contracts.clone())
        .expect("graph contract projection");

    host.prepare_default_output_hardware()
        .expect("hardware config");
    host.start_plugin_scan(signal_runtime::PluginScanRequest {
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
    host.runtime.set_cpu_load_percent(4.5);
    host.runtime.set_graph_latency_ms(2.7);
    host.runtime.start().expect("start runtime");

    let protocol = ClapBlockProtocol::new(
        "plugin:clap:default",
        "instance:local:default",
        signal_plugin::PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        2048,
    );
    (host, protocol)
}
