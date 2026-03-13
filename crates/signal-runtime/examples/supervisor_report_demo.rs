use signal_runtime::{
    HandshakeRequest, PluginFaultKind, RuntimeConfig, RuntimeConfigRequest, RuntimeEvent,
    RuntimeEventRecorder, RuntimeEventSink, RuntimeLifecycleApi, RuntimeSupervisorReport,
    RuntimeWatchdogTrigger, SignalRuntime,
};

fn main() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "signal-runtime-example".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("handshake runtime");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("configure runtime");
    runtime.start().expect("start runtime");

    let mut recorder = RuntimeEventRecorder::default();
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::SupervisionChanged(signal_runtime::RuntimeSupervisionSnapshot {
            watchdog_restart_count: 3,
            safe_mode_enabled: true,
            xrun_overload_active: false,
            last_watchdog_trigger: Some(RuntimeWatchdogTrigger::HeartbeatMisses),
            last_sandbox_id: Some("sandbox-demo".into()),
            last_processing_epoch: Some(4),
        }),
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxFault {
            sandbox_id: "sandbox-demo".into(),
            kind: PluginFaultKind::Timeout,
            detail: "heartbeat watchdog exceeded threshold".into(),
            processing_epoch: Some(4),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxFault {
            sandbox_id: "sandbox-demo".into(),
            kind: PluginFaultKind::Timeout,
            detail: "block deadline missed twice".into(),
            processing_epoch: Some(3),
        },
    );

    let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    println!("{}", report.render_multiline());
}
