use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecoveryState {
    Steady,
    Recovering,
    Faulted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeInterruptionClass {
    #[default]
    Steady,
    Resumable,
    Restartable,
    Recoverable,
    Terminal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultStatusSnapshot {
    pub recovery_state: RuntimeRecoveryState,
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    pub active_fault_count: usize,
    pub xrun_overload_active: bool,
    pub plugin_fault_active: bool,
    pub watchdog_active: bool,
    pub device_loss_active: bool,
    pub transport_fault_active: bool,
    pub missing_plugin_binding_active: bool,
    pub safe_mode_enabled: bool,
    pub restart_count: u64,
    pub watchdog_restart_count: u32,
    pub plugin_fault_count: usize,
    pub transport_faulted_session_count: usize,
    pub device_loss_count: u64,
    pub summary: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeFaultStatusCaptureInput<'a> {
    pub readiness: RuntimeReadiness,
    pub control_snapshot: &'a RuntimeControlSnapshot,
    pub diagnostics_snapshot: &'a RuntimeDiagnosticsSnapshot,
    pub supervision_snapshot: &'a RuntimeSupervisionSnapshot,
    pub engine_block_snapshot: &'a RuntimeEngineBlockSnapshot,
    pub transport_concurrency_snapshot: &'a RuntimeTransportConcurrencySnapshot,
    pub plugin_lifecycle_snapshot: &'a RuntimePluginLifecycleSnapshot,
    pub device_loss_active: bool,
    pub device_loss_count: u64,
}

impl RuntimeFaultStatusSnapshot {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeInterruptionSummary {
    pub active: bool,
    pub class: RuntimeInterruptionClass,
    pub rebindable: bool,
    pub recovery_state: RuntimeRecoveryState,
    pub primary_fault_cause: Option<RuntimeFaultCause>,
    pub safe_mode_enabled: bool,
    pub deferred_service_class: Option<RuntimeDeferredServiceClass>,
    pub deferred_service_decision: Option<RuntimeDeferredServiceDecision>,
    pub summary: String,
}

impl RuntimeInterruptionSummary {
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
