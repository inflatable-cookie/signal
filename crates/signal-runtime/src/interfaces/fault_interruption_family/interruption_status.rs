use super::*;

/// Overall recovery trajectory of the runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecoveryState {
    /// No faults; normal operation.
    Steady,
    /// At least one recoverable fault is active.
    Recovering,
    /// Runtime hit a fatal error and cannot self-recover.
    Faulted,
}

/// Classification of the current interruption severity.
///
/// Drives the host's recovery UX: `Steady` → no action; `Resumable` → brief
/// pause; `Restartable` → sandbox restart; `Recoverable` → user-visible
/// degradation; `Terminal` → manual intervention required.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeInterruptionClass {
    /// No interruption; normal operation.
    #[default]
    Steady,
    /// Brief transient pause; no user-visible action required.
    Resumable,
    /// Sandbox must be restarted.
    Restartable,
    /// User-visible degradation; recovery is possible.
    Recoverable,
    /// Fatal; manual intervention required.
    Terminal,
}

/// Aggregated fault status snapshot used to derive the interruption class and
/// recovery state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultStatusSnapshot {
    /// Current recovery trajectory.
    pub recovery_state: RuntimeRecoveryState,
    /// Primary fault cause driving the current interruption, if any.
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    /// Number of distinct active fault conditions.
    pub active_fault_count: usize,
    /// Whether xrun overload is currently active.
    pub xrun_overload_active: bool,
    /// Whether at least one plugin sandbox has faulted.
    pub plugin_fault_active: bool,
    /// Whether the watchdog has triggered a restart.
    pub watchdog_active: bool,
    /// Whether a device-loss event is currently active.
    pub device_loss_active: bool,
    /// Whether a transport session has faulted.
    pub transport_fault_active: bool,
    /// Whether a plugin binding is missing.
    pub missing_plugin_binding_active: bool,
    /// Whether safe mode is engaged.
    pub safe_mode_enabled: bool,
    /// Total runtime restart count.
    pub restart_count: u64,
    /// Total watchdog-triggered restart count.
    pub watchdog_restart_count: u32,
    /// Number of faulted or quarantined plugin sandboxes.
    pub plugin_fault_count: usize,
    /// Number of transport sessions that have faulted during detach.
    pub transport_faulted_session_count: usize,
    /// Total device-loss event count since startup.
    pub device_loss_count: u64,
    /// Human-readable summary of this snapshot.
    pub summary: String,
}

/// Input data bundle for [`RuntimeFaultStatusSnapshot::capture`].
#[derive(Clone, Debug)]
pub struct RuntimeFaultStatusCaptureInput<'a> {
    /// Current runtime readiness state.
    pub readiness: RuntimeReadiness,
    /// Control plane snapshot.
    pub control_snapshot: &'a RuntimeControlSnapshot,
    /// Diagnostics snapshot.
    pub diagnostics_snapshot: &'a RuntimeDiagnosticsSnapshot,
    /// Supervision snapshot.
    pub supervision_snapshot: &'a RuntimeSupervisionSnapshot,
    /// Most recent engine block snapshot.
    pub engine_block_snapshot: &'a RuntimeEngineBlockSnapshot,
    /// Transport concurrency snapshot.
    pub transport_concurrency_snapshot: &'a RuntimeTransportConcurrencySnapshot,
    /// Plugin lifecycle snapshot.
    pub plugin_lifecycle_snapshot: &'a RuntimePluginLifecycleSnapshot,
    /// Whether a device-loss event is currently active.
    pub device_loss_active: bool,
    /// Current device-loss event count.
    pub device_loss_count: u64,
}

impl RuntimeFaultStatusSnapshot {
    /// Captures a fault status snapshot from the current runtime state.
    pub fn capture(input: RuntimeFaultStatusCaptureInput<'_>) -> Self {
        let xrun_overload_active = input.supervision_snapshot.xrun_overload_active;
        let plugin_fault_count = input
            .plugin_lifecycle_snapshot
            .faulted_sandbox_count
            .saturating_add(input.plugin_lifecycle_snapshot.quarantined_sandbox_count);
        let plugin_fault_active = plugin_fault_count > 0;
        let watchdog_active = input.supervision_snapshot.safe_mode_enabled
            && input.supervision_snapshot.watchdog_restart_count > 0;
        let transport_faulted_session_count = input
            .transport_concurrency_snapshot
            .current_detach_faulted_sessions;
        let transport_fault_active = transport_faulted_session_count > 0;
        let missing_plugin_binding_active = input
            .engine_block_snapshot
            .prework_service_missing_bound_plugin_sandboxes
            > 0;
        let runtime_error_active = matches!(input.readiness, RuntimeReadiness::Failed { .. });
        let primary_fault_cause = if input.device_loss_active {
            Some(RuntimeFaultCause::DeviceLoss)
        } else if watchdog_active {
            Some(RuntimeFaultCause::WatchdogRestart)
        } else if plugin_fault_active {
            Some(RuntimeFaultCause::PluginFault)
        } else if transport_fault_active {
            Some(RuntimeFaultCause::TransportFault)
        } else if xrun_overload_active {
            Some(RuntimeFaultCause::XrunOverload)
        } else if missing_plugin_binding_active {
            Some(RuntimeFaultCause::MissingPluginBinding)
        } else if runtime_error_active {
            Some(RuntimeFaultCause::RuntimeError)
        } else {
            None
        };
        let mut active_fault_count = usize::from(xrun_overload_active)
            + usize::from(plugin_fault_active)
            + usize::from(watchdog_active)
            + usize::from(input.device_loss_active)
            + usize::from(transport_fault_active)
            + usize::from(missing_plugin_binding_active);
        if runtime_error_active && primary_fault_cause == Some(RuntimeFaultCause::RuntimeError) {
            active_fault_count = active_fault_count.saturating_add(1);
        }
        let recovery_state = if runtime_error_active {
            RuntimeRecoveryState::Faulted
        } else if input.supervision_snapshot.safe_mode_enabled
            || xrun_overload_active
            || input.device_loss_active
            || watchdog_active
            || transport_fault_active
            || input.plugin_lifecycle_snapshot.restarting_sandbox_count > 0
            || input.control_snapshot.restart_count > 0
        {
            RuntimeRecoveryState::Recovering
        } else {
            RuntimeRecoveryState::Steady
        };
        let mut snapshot = Self {
            recovery_state,
            primary_fault_cause,
            active_fault_count,
            xrun_overload_active,
            plugin_fault_active,
            watchdog_active,
            device_loss_active: input.device_loss_active,
            transport_fault_active,
            missing_plugin_binding_active,
            safe_mode_enabled: input.supervision_snapshot.safe_mode_enabled,
            restart_count: input.control_snapshot.restart_count,
            watchdog_restart_count: input.supervision_snapshot.watchdog_restart_count,
            plugin_fault_count,
            transport_faulted_session_count,
            device_loss_count: input.device_loss_count,
            summary: String::new(),
        };
        snapshot.summary = format!(
            "recovery={:?} primary={:?} faults={} xruns={} plugin_faults={} watchdog_restarts={} device_losses={} transport_faulted_sessions={} safe_mode={} restarts={}",
            snapshot.recovery_state,
            snapshot.primary_fault_cause,
            snapshot.active_fault_count,
            input.diagnostics_snapshot.xruns,
            snapshot.plugin_fault_count,
            snapshot.watchdog_restart_count,
            snapshot.device_loss_count,
            snapshot.transport_faulted_session_count,
            snapshot.safe_mode_enabled,
            snapshot.restart_count,
        );
        snapshot
    }
}

/// High-level interruption summary for host UX: active state, class,
/// rebindability, and deferred-service context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeInterruptionSummary {
    /// Whether an interruption is currently active.
    pub active: bool,
    /// Severity class of the current interruption.
    pub class: RuntimeInterruptionClass,
    /// Whether rebinding is possible without a full restart.
    pub rebindable: bool,
    /// Current recovery trajectory.
    pub recovery_state: RuntimeRecoveryState,
    /// Primary fault cause, if any.
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    /// Whether safe mode is engaged.
    pub safe_mode_enabled: bool,
    /// Class of the most recent deferred service work item, if any.
    pub deferred_service_class: Option<RuntimeDeferredServiceClass>,
    /// Decision made for the most recent deferred service work item, if any.
    pub deferred_service_decision: Option<RuntimeDeferredServiceDecision>,
    /// Human-readable summary of this interruption state.
    pub summary: String,
}

impl RuntimeInterruptionSummary {
    /// Captures an interruption summary from the current fault status and deferred service receipt.
    pub fn capture(
        fault_status: &RuntimeFaultStatusSnapshot,
        last_deferred_service_receipt: Option<&RuntimeDeferredServiceReceipt>,
    ) -> Self {
        let class = match fault_status.recovery_state {
            RuntimeRecoveryState::Faulted => RuntimeInterruptionClass::Terminal,
            RuntimeRecoveryState::Recovering
                if matches!(
                    fault_status.primary_fault_cause,
                    Some(
                        RuntimeFaultCause::DeviceLoss
                            | RuntimeFaultCause::WatchdogRestart
                            | RuntimeFaultCause::PluginFault
                            | RuntimeFaultCause::TransportFault
                            | RuntimeFaultCause::MissingPluginBinding
                    )
                ) =>
            {
                RuntimeInterruptionClass::Restartable
            }
            RuntimeRecoveryState::Recovering => RuntimeInterruptionClass::Recoverable,
            RuntimeRecoveryState::Steady
                if matches!(
                    last_deferred_service_receipt.map(|receipt| receipt.decision),
                    Some(
                        RuntimeDeferredServiceDecision::Defer
                            | RuntimeDeferredServiceDecision::Throttle
                    )
                ) =>
            {
                RuntimeInterruptionClass::Resumable
            }
            RuntimeRecoveryState::Steady => RuntimeInterruptionClass::Steady,
        };
        let rebindable = matches!(
            fault_status.primary_fault_cause,
            Some(
                RuntimeFaultCause::DeviceLoss
                    | RuntimeFaultCause::PluginFault
                    | RuntimeFaultCause::TransportFault
                    | RuntimeFaultCause::MissingPluginBinding
            )
        );
        let mut summary = Self {
            active: class != RuntimeInterruptionClass::Steady,
            class,
            rebindable,
            recovery_state: fault_status.recovery_state,
            primary_fault_cause: fault_status.primary_fault_cause,
            safe_mode_enabled: fault_status.safe_mode_enabled,
            deferred_service_class: last_deferred_service_receipt.map(|receipt| receipt.work_class),
            deferred_service_decision: last_deferred_service_receipt
                .map(|receipt| receipt.decision),
            summary: String::new(),
        };
        summary.summary = format!(
            "class={:?} active={} rebindable={} recovery={:?} primary={:?} deferred={:?}/{:?} safe_mode={}",
            summary.class,
            summary.active,
            summary.rebindable,
            summary.recovery_state,
            summary.primary_fault_cause,
            summary.deferred_service_class,
            summary.deferred_service_decision,
            summary.safe_mode_enabled,
        );
        summary
    }
}
