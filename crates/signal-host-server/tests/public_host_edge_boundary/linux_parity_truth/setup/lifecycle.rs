use super::super::super::*;

pub(super) fn record_server_linux_parity_lifecycle(runtime: &mut SignalRuntime) {
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "server-linux-clap-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:server-linux-parity".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "server-linux-clap-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "server-linux-clap-sandbox",
        "lease-server-linux-clap",
        "region-server-linux-clap",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "server-linux-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:server-linux-parity".into()),
    });
    runtime.record_recovery_cycle(
        "server-linux-vst3-sandbox",
        signal_runtime::RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "server-linux-vst3-sandbox",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "server-linux-lv2-sandbox".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:server-linux-parity".into()),
    });
    runtime.record_plugin_sandbox_fault(
        "server-linux-lv2-sandbox",
        PluginFaultKind::Crash,
        "server linux lv2 parity fault",
        Some(3),
    );
}
