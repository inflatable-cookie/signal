use super::*;

#[test]
fn runtime_owns_watchdog_restart_escalation() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().unwrap();

    let first = runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });
    assert_eq!(first.watchdog_restart_count, 1);
    assert!(!first.safe_mode_enabled);

    let second = runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
    });
    assert_eq!(second.watchdog_restart_count, 2);
    assert!(second.safe_mode_enabled);
    assert_eq!(
        second.last_watchdog_trigger,
        Some(RuntimeWatchdogTrigger::DeadlineMisses)
    );
    assert_eq!(second.last_processing_epoch, Some(2));
    assert!(matches!(
        runtime.get_readiness(),
        RuntimeReadiness::Degraded { .. }
    ));
}

#[test]
fn runtime_fault_status_snapshot_classifies_watchdog_plugin_fault_and_xrun_pressure() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");
    runtime.record_xrun_overload(Some(1));
    runtime.record_xrun_overload(Some(2));
    runtime.record_xrun_overload(Some(3));
    runtime.record_plugin_sandbox_fault(
        "sandbox-a",
        PluginFaultKind::Crash,
        "sandbox crashed during process block",
        Some(2),
    );
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 3,
    });
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 4,
    });

    let status =
        RuntimeFaultStatusSnapshot::capture(crate::interfaces::RuntimeFaultStatusCaptureInput {
            readiness: runtime.get_readiness(),
            control_snapshot: &runtime.get_control_snapshot(),
            diagnostics_snapshot: &runtime.get_diagnostics_snapshot(),
            supervision_snapshot: &runtime.get_supervision_snapshot(),
            engine_block_snapshot: &runtime.get_engine_block_snapshot(),
            transport_concurrency_snapshot: &runtime.get_transport_concurrency_snapshot(),
            plugin_lifecycle_snapshot: &runtime.get_plugin_lifecycle_snapshot(),
            device_loss_active: false,
            device_loss_count: 0,
        });

    assert_eq!(status.recovery_state, RuntimeRecoveryState::Recovering);
    assert_eq!(
        status.primary_fault_cause,
        Some(RuntimeFaultCause::WatchdogRestart)
    );
    assert_eq!(status.active_fault_count, 3);
    assert!(status.xrun_overload_active);
    assert!(status.plugin_fault_active);
    assert!(status.watchdog_active);
    assert!(status.safe_mode_enabled);
    assert_eq!(status.plugin_fault_count, 1);
    assert_eq!(status.watchdog_restart_count, 2);
}

#[test]
fn runtime_fault_status_snapshot_clears_watchdog_active_after_safe_mode_recovery() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
    });
    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("safe mode should clear after watchdog recovery");

    let status =
        RuntimeFaultStatusSnapshot::capture(crate::interfaces::RuntimeFaultStatusCaptureInput {
            readiness: runtime.get_readiness(),
            control_snapshot: &runtime.get_control_snapshot(),
            diagnostics_snapshot: &runtime.get_diagnostics_snapshot(),
            supervision_snapshot: &runtime.get_supervision_snapshot(),
            engine_block_snapshot: &runtime.get_engine_block_snapshot(),
            transport_concurrency_snapshot: &runtime.get_transport_concurrency_snapshot(),
            plugin_lifecycle_snapshot: &runtime.get_plugin_lifecycle_snapshot(),
            device_loss_active: false,
            device_loss_count: 0,
        });

    assert_eq!(status.recovery_state, RuntimeRecoveryState::Steady);
    assert_eq!(status.primary_fault_cause, None);
    assert_eq!(status.active_fault_count, 0);
    assert!(!status.watchdog_active);
    assert!(!status.safe_mode_enabled);
    assert_eq!(status.watchdog_restart_count, 2);
}

#[test]
fn runtime_observation_report_surfaces_restartable_interruption_summary() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
        processing_epoch: 1,
    });
    runtime.record_watchdog_restart(WatchdogRestartRecord {
        sandbox_id: "sandbox-a".into(),
        trigger: RuntimeWatchdogTrigger::DeadlineMisses,
        processing_epoch: 2,
    });

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());

    assert_eq!(
        observation.fault_status.primary_fault_cause,
        Some(RuntimeFaultCause::WatchdogRestart)
    );
    assert_eq!(
        observation.interruption_summary.class,
        RuntimeInterruptionClass::Restartable
    );
    assert!(observation.interruption_summary.active);
    assert!(!observation.interruption_summary.rebindable);

}

#[test]
fn runtime_fault_diagnostic_receipt_maps_xrun_pressure_into_runtime_owned_primary_family() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");
    runtime.record_xrun_overload(Some(1));
    runtime.record_xrun_overload(Some(2));
    runtime.record_xrun_overload(Some(3));

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let receipt = &observation.fault_diagnostic_receipt;
    let xrun = receipt
        .contributions
        .iter()
        .find(|entry| entry.family == crate::interfaces::RuntimeFaultDiagnosticFamily::XrunPressure)
        .expect("xrun contribution should be present");

    assert_eq!(
        receipt.primary_family,
        Some(crate::interfaces::RuntimeFaultDiagnosticFamily::XrunPressure)
    );
    assert_eq!(
        receipt.primary_fault_cause,
        Some(crate::interfaces::RuntimeFaultCause::XrunOverload)
    );
    assert_eq!(
        receipt.interruption_class,
        crate::interfaces::RuntimeInterruptionClass::Recoverable
    );
    assert!(xrun.active);
    assert_eq!(xrun.event_count, 3);
    assert_eq!(
        xrun.authority,
        crate::interfaces::RuntimeFaultDiagnosticAuthority::RuntimeCanonical
    );

}

#[test]
fn runtime_fault_diagnostic_receipt_maps_deferred_work_pressure_without_faulting_runtime() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode");

    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:queue:fault-diagnostic:deferred".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer offline render queue");
    assert_eq!(
        deferred.orchestration.decision,
        RuntimeDeferredServiceDecision::Defer
    );

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let receipt = &observation.fault_diagnostic_receipt;
    let deferred_entry = receipt
        .contributions
        .iter()
        .find(|entry| {
            entry.family == crate::interfaces::RuntimeFaultDiagnosticFamily::DeferredWorkPressure
        })
        .expect("deferred-work contribution should be present");

    assert_eq!(
        receipt.primary_family,
        Some(crate::interfaces::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert_eq!(receipt.primary_fault_cause, None);
    assert_eq!(
        receipt.interruption_class,
        crate::interfaces::RuntimeInterruptionClass::Recoverable
    );
    assert!(deferred_entry.active);
    assert!(deferred_entry.event_count >= 1);
    assert!(deferred_entry
        .detail
        .as_deref()
        .unwrap_or_default()
        .contains("decision=Some(Defer)"));
}

#[test]
fn runtime_xrun_overload_escalates_into_safe_mode_and_clears_after_recovery() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");

    let first = runtime.record_xrun_overload(Some(1));
    assert!(!first.safe_mode_enabled);
    assert!(!first.xrun_overload_active);

    let second = runtime.record_xrun_overload(Some(2));
    assert!(!second.safe_mode_enabled);
    assert!(!second.xrun_overload_active);

    let third = runtime.record_xrun_overload(Some(3));
    assert!(third.safe_mode_enabled);
    assert!(third.xrun_overload_active);
    assert!(matches!(
        runtime.get_readiness(),
        RuntimeReadiness::Degraded { .. }
    ));

    let active_status =
        RuntimeFaultStatusSnapshot::capture(crate::interfaces::RuntimeFaultStatusCaptureInput {
            readiness: runtime.get_readiness(),
            control_snapshot: &runtime.get_control_snapshot(),
            diagnostics_snapshot: &runtime.get_diagnostics_snapshot(),
            supervision_snapshot: &runtime.get_supervision_snapshot(),
            engine_block_snapshot: &runtime.get_engine_block_snapshot(),
            transport_concurrency_snapshot: &runtime.get_transport_concurrency_snapshot(),
            plugin_lifecycle_snapshot: &runtime.get_plugin_lifecycle_snapshot(),
            device_loss_active: false,
            device_loss_count: 0,
        });
    assert_eq!(
        active_status.recovery_state,
        RuntimeRecoveryState::Recovering
    );
    assert_eq!(
        active_status.primary_fault_cause,
        Some(RuntimeFaultCause::XrunOverload)
    );
    assert_eq!(active_status.active_fault_count, 1);
    assert!(active_status.xrun_overload_active);
    assert!(active_status.safe_mode_enabled);

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("safe mode should clear");

    let recovered_status =
        RuntimeFaultStatusSnapshot::capture(crate::interfaces::RuntimeFaultStatusCaptureInput {
            readiness: runtime.get_readiness(),
            control_snapshot: &runtime.get_control_snapshot(),
            diagnostics_snapshot: &runtime.get_diagnostics_snapshot(),
            supervision_snapshot: &runtime.get_supervision_snapshot(),
            engine_block_snapshot: &runtime.get_engine_block_snapshot(),
            transport_concurrency_snapshot: &runtime.get_transport_concurrency_snapshot(),
            plugin_lifecycle_snapshot: &runtime.get_plugin_lifecycle_snapshot(),
            device_loss_active: false,
            device_loss_count: 0,
        });
    assert_eq!(
        recovered_status.recovery_state,
        RuntimeRecoveryState::Steady
    );
    assert_eq!(recovered_status.primary_fault_cause, None);
    assert_eq!(recovered_status.active_fault_count, 0);
    assert!(!recovered_status.xrun_overload_active);
    assert_eq!(runtime.get_diagnostics_snapshot().xruns, 3);
}

#[test]
fn runtime_fail_runtime_marks_faulted_recovery_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().expect("start runtime");

    let readiness = runtime.fail_runtime(RuntimeError::new(
        RuntimeErrorKind::HardwareFailure,
        "simulated output recovery exhaustion",
    ));
    assert!(matches!(readiness, RuntimeReadiness::Failed { .. }));

    let status =
        RuntimeFaultStatusSnapshot::capture(crate::interfaces::RuntimeFaultStatusCaptureInput {
            readiness: runtime.get_readiness(),
            control_snapshot: &runtime.get_control_snapshot(),
            diagnostics_snapshot: &runtime.get_diagnostics_snapshot(),
            supervision_snapshot: &runtime.get_supervision_snapshot(),
            engine_block_snapshot: &runtime.get_engine_block_snapshot(),
            transport_concurrency_snapshot: &runtime.get_transport_concurrency_snapshot(),
            plugin_lifecycle_snapshot: &runtime.get_plugin_lifecycle_snapshot(),
            device_loss_active: false,
            device_loss_count: 0,
        });
    assert_eq!(status.recovery_state, RuntimeRecoveryState::Faulted);
    assert_eq!(
        status.primary_fault_cause,
        Some(RuntimeFaultCause::RuntimeError)
    );
    assert_eq!(status.active_fault_count, 1);
    assert!(runtime.get_effective_config().safe_mode_enabled);
}
