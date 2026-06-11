use super::*;

/// Structured fault diagnosis: primary cause, interruption class, and a list
/// of [`RuntimeFaultContributionReceipt`]s covering each fault family.
///
/// Built by `RuntimeFaultDiagnosticReceipt::capture()`.  Used by the host to
/// present structured fault information in the UI and to decide recovery
/// actions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultDiagnosticReceipt {
    /// Primary fault family, if a dominant fault family can be identified.
    pub primary_family: Option<RuntimeFaultDiagnosticFamily>,
    /// Primary fault cause driving the current interruption.
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    /// Interruption class derived from the fault status.
    pub interruption_class: RuntimeInterruptionClass,
    /// Recovery trajectory at the time this receipt was captured.
    pub recovery_state: RuntimeRecoveryState,
    /// Whether safe mode is currently engaged.
    pub safe_mode_enabled: bool,
    /// Whether the current fault state allows rebinding without a full restart.
    pub rebindable: bool,
    /// Per-family contribution receipts.
    pub contributions: Vec<RuntimeFaultContributionReceipt>,
}

impl RuntimeFaultDiagnosticReceipt {
    /// Captures a diagnostic receipt from the current fault, interruption, and engine state.
    pub fn capture(
        fault_status: &RuntimeFaultStatusSnapshot,
        interruption_summary: &RuntimeInterruptionSummary,
        degradation_summary: &RuntimeDegradationSummary,
        engine_block_snapshot: &RuntimeEngineBlockSnapshot,
        last_deferred_service_receipt: Option<&RuntimeDeferredServiceReceipt>,
        host_io: Option<&RuntimeHostIoSummary>,
    ) -> Self {
        let plugin_boundary_event_count = degradation_summary
            .plugin_fault_count
            .saturating_add(degradation_summary.transport_fault_event_count)
            .saturating_add(degradation_summary.broker_failure_event_count)
            .saturating_add(degradation_summary.sandbox_operation_failure_event_count)
            .saturating_add(usize::from(
                fault_status.missing_plugin_binding_active
                    || degradation_summary.missing_bound_plugin_sandboxes > 0,
            )) as u64;
        let plugin_boundary_active = fault_status.plugin_fault_active
            || fault_status.transport_fault_active
            || fault_status.missing_plugin_binding_active;
        let xrun_event_count = degradation_summary.xrun_count;
        let deferred_event_count = engine_block_snapshot
            .prework_service_starvation_count
            .saturating_add(engine_block_snapshot.prework_service_throttle_count)
            .saturating_add(engine_block_snapshot.prework_service_yield_count)
            .saturating_add(
                last_deferred_service_receipt
                    .map(|receipt| receipt.deferred_work_item_count as u64)
                    .unwrap_or(0),
            );
        let deferred_active = matches!(
            last_deferred_service_receipt.map(|receipt| receipt.decision),
            Some(RuntimeDeferredServiceDecision::Defer | RuntimeDeferredServiceDecision::Throttle)
        ) || matches!(
            engine_block_snapshot.prework_service_state,
            RuntimePreworkServiceState::Yielding
                | RuntimePreworkServiceState::Paused
                | RuntimePreworkServiceState::Starved
        ) || matches!(
            engine_block_snapshot.prework_service_pressure,
            RuntimePreworkServicePressure::Elevated | RuntimePreworkServicePressure::Critical
        );
        let callback_event_count = host_io
            .map(|host_io| {
                host_io
                    .hardware
                    .callback_overrun_count
                    .saturating_add(host_io.hardware.xrun_count)
            })
            .unwrap_or(0);
        let callback_active = host_io
            .map(|host_io| {
                host_io.hardware.callback_overrun_count > 0
                    || host_io.hardware.xrun_count > 0
                    || host_io.hardware.restart_failure_count > 0
            })
            .unwrap_or(false);

        let mut contributions = vec![
            RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::XrunPressure,
                authority: RuntimeFaultDiagnosticAuthority::RuntimeCanonical,
                active: fault_status.xrun_overload_active,
                event_count: xrun_event_count,
                detail: Some(format!(
                    "xrun_overload_active={} safe_mode={} runtime_xruns={}",
                    fault_status.xrun_overload_active,
                    fault_status.safe_mode_enabled,
                    xrun_event_count
                )),
            },
            RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::PluginBoundaryFault,
                authority: RuntimeFaultDiagnosticAuthority::RuntimeCanonical,
                active: plugin_boundary_active,
                event_count: plugin_boundary_event_count,
                detail: Some(format!(
                    "plugin_faults={} transport_fault_events={} broker_failures={} sandbox_operation_failures={} missing_bindings={}",
                    degradation_summary.plugin_fault_count,
                    degradation_summary.transport_fault_event_count,
                    degradation_summary.broker_failure_event_count,
                    degradation_summary.sandbox_operation_failure_event_count,
                    usize::from(
                        fault_status.missing_plugin_binding_active
                            || degradation_summary.missing_bound_plugin_sandboxes > 0
                    )
                )),
            },
            RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::DevicePathFault,
                authority: RuntimeFaultDiagnosticAuthority::RuntimeCanonical,
                active: fault_status.device_loss_active,
                event_count: fault_status.device_loss_count,
                detail: Some(format!(
                    "device_loss_active={} device_losses={} watchdog_restarts={}",
                    fault_status.device_loss_active,
                    fault_status.device_loss_count,
                    fault_status.watchdog_restart_count
                )),
            },
            RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::DeferredWorkPressure,
                authority: RuntimeFaultDiagnosticAuthority::RuntimeCanonical,
                active: deferred_active,
                event_count: deferred_event_count,
                detail: Some(format!(
                    "decision={:?} reason={:?} prework_state={:?} prework_pressure={:?} starvations={} throttles={} yields={} deferred_items={}",
                    last_deferred_service_receipt.map(|receipt| receipt.decision),
                    last_deferred_service_receipt.map(|receipt| receipt.reason),
                    engine_block_snapshot.prework_service_state,
                    engine_block_snapshot.prework_service_pressure,
                    engine_block_snapshot.prework_service_starvation_count,
                    engine_block_snapshot.prework_service_throttle_count,
                    engine_block_snapshot.prework_service_yield_count,
                    last_deferred_service_receipt
                        .map(|receipt| receipt.deferred_work_item_count)
                        .unwrap_or(0)
                )),
            },
        ];
        if let Some(host_io) = host_io {
            contributions.push(RuntimeFaultContributionReceipt {
                family: RuntimeFaultDiagnosticFamily::CallbackPressure,
                authority: RuntimeFaultDiagnosticAuthority::HostAdvisory,
                active: callback_active,
                event_count: callback_event_count,
                detail: Some(format!(
                    "callback_count={} callback_interval_ms={:.3} callback_overruns={} backend_xruns={} restart_failures={}",
                    host_io.audio_pump.callback_count,
                    host_io.clocking.callback_interval_ms,
                    host_io.hardware.callback_overrun_count,
                    host_io.hardware.xrun_count,
                    host_io.hardware.restart_failure_count
                )),
            });
        }

        for _contribution in &mut contributions {}

        let primary_family = match fault_status.primary_fault_cause {
            Some(RuntimeFaultCause::XrunOverload) => {
                Some(RuntimeFaultDiagnosticFamily::XrunPressure)
            }
            Some(
                RuntimeFaultCause::PluginFault
                | RuntimeFaultCause::TransportFault
                | RuntimeFaultCause::MissingPluginBinding,
            ) => Some(RuntimeFaultDiagnosticFamily::PluginBoundaryFault),
            Some(RuntimeFaultCause::DeviceLoss) => {
                Some(RuntimeFaultDiagnosticFamily::DevicePathFault)
            }
            Some(RuntimeFaultCause::WatchdogRestart) => {
                if plugin_boundary_active {
                    Some(RuntimeFaultDiagnosticFamily::PluginBoundaryFault)
                } else if fault_status.device_loss_active {
                    Some(RuntimeFaultDiagnosticFamily::DevicePathFault)
                } else if fault_status.xrun_overload_active {
                    Some(RuntimeFaultDiagnosticFamily::XrunPressure)
                } else if deferred_active {
                    Some(RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
                } else {
                    None
                }
            }
            Some(RuntimeFaultCause::RuntimeError) => None,
            None if deferred_active => Some(RuntimeFaultDiagnosticFamily::DeferredWorkPressure),
            None => None,
        };

        Self {
            primary_family,
            primary_fault_cause: fault_status.primary_fault_cause,
            interruption_class: interruption_summary.class,
            recovery_state: fault_status.recovery_state,
            safe_mode_enabled: fault_status.safe_mode_enabled,
            rebindable: interruption_summary.rebindable,
            contributions,
        }
    }
}
