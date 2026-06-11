use super::*;

pub(super) fn runtime_plugin_interchange_snapshot_from_snapshot(
    sandbox: Option<&RuntimePluginSandboxSnapshot>,
    discovered_types: &[RuntimePluginDiscoveredTypeRecord],
) -> RuntimePluginInterchangeSnapshot {
    let discovered_type = runtime_plugin_discovered_type_for_recall(
        sandbox.and_then(|sandbox| sandbox.plugin_type_id.as_deref()),
        discovered_types,
    );
    let portability_class = if let Some(record) = discovered_type {
        if record.state_contract.supports_snapshot {
            match sandbox.and_then(|sandbox| sandbox.plugin_format) {
                Some(PluginFormat::Clap)
                    if !record.lifecycle_contract.requires_main_thread_for_state =>
                {
                    RuntimePluginRecallPortabilityClass::Portable
                }
                Some(_) => RuntimePluginRecallPortabilityClass::Guarded,
                None => RuntimePluginRecallPortabilityClass::Guarded,
            }
        } else {
            RuntimePluginRecallPortabilityClass::NativeOnly
        }
    } else {
        RuntimePluginRecallPortabilityClass::Unsupported
    };
    let shared_payload_available = matches!(
        portability_class,
        RuntimePluginRecallPortabilityClass::Portable
            | RuntimePluginRecallPortabilityClass::Guarded
    );
    let native_supplement_required = matches!(
        portability_class,
        RuntimePluginRecallPortabilityClass::Guarded
            | RuntimePluginRecallPortabilityClass::NativeOnly
    );
    RuntimePluginInterchangeSnapshot {
        portability_class,
        shared_payload_available,
        native_supplement_required,
    }
}

pub(super) fn runtime_plugin_recall_snapshot(
    sandbox_id: Option<&str>,
    sandbox: Option<&RuntimePluginSandboxSnapshot>,
    discovered_types: &[RuntimePluginDiscoveredTypeRecord],
) -> RuntimePluginRecallSnapshot {
    let state = match (sandbox_id, sandbox) {
        (None, _) => RuntimePluginRecallState::Unbound,
        (Some(_), None) => RuntimePluginRecallState::Cold,
        (Some(_), Some(sandbox))
            if matches!(
                sandbox.state,
                RuntimePluginLifecycleState::Faulted
                    | RuntimePluginLifecycleState::Restarting
                    | RuntimePluginLifecycleState::Quarantined
            ) =>
        {
            RuntimePluginRecallState::Unavailable
        }
        (Some(_), Some(sandbox)) if sandbox.recovery_count > 0 || sandbox.restart_count > 0 => {
            RuntimePluginRecallState::Recovered
        }
        (Some(_), Some(sandbox))
            if sandbox.instance_id.is_some() || sandbox.lifecycle_stage.is_some() =>
        {
            RuntimePluginRecallState::Warm
        }
        (Some(_), Some(_)) => RuntimePluginRecallState::Cold,
    };

    RuntimePluginRecallSnapshot {
        state,
        payload: runtime_plugin_recall_payload(sandbox_id, sandbox, discovered_types),
    }
}

pub(super) fn runtime_plugin_recall_payload(
    sandbox_id: Option<&str>,
    sandbox: Option<&RuntimePluginSandboxSnapshot>,
    discovered_types: &[RuntimePluginDiscoveredTypeRecord],
) -> RuntimePluginRecallPayload {
    RuntimePluginRecallPayload {
        sandbox_id: sandbox_id.map(str::to_string),
        plugin_type_id: sandbox.and_then(|sandbox| sandbox.plugin_type_id.clone()),
        plugin_format: sandbox.and_then(|sandbox| sandbox.plugin_format),
        lifecycle_state: sandbox.map(|sandbox| sandbox.state),
        lifecycle_stage: sandbox.and_then(|sandbox| sandbox.lifecycle_stage),
        transport_stage: sandbox.and_then(|sandbox| sandbox.transport_stage),
        readiness_state: sandbox.and_then(|sandbox| sandbox.readiness_state.clone()),
        recovery_count: sandbox.map(|sandbox| sandbox.recovery_count).unwrap_or(0),
        restart_count: sandbox.map(|sandbox| sandbox.restart_count).unwrap_or(0),
        fault_count: sandbox.map(|sandbox| sandbox.fault_count).unwrap_or(0),
        last_restart_intent: sandbox.and_then(|sandbox| sandbox.last_restart_intent),
        last_stop_reason: sandbox.and_then(|sandbox| sandbox.last_stop_reason),
        last_fault_kind: sandbox.and_then(|sandbox| sandbox.last_fault_kind),
        last_fault_detail: sandbox.and_then(|sandbox| sandbox.last_fault_detail.clone()),
        degraded_reasons: sandbox
            .map(|sandbox| sandbox.degraded_reasons.clone())
            .unwrap_or_default(),
        interchange: runtime_plugin_interchange_snapshot_from_snapshot(sandbox, discovered_types),
    }
}

pub(super) fn runtime_plugin_compensation_observation(
    sandbox_id: Option<&str>,
    sandbox: Option<&RuntimePluginSandboxSnapshot>,
) -> RuntimePluginCompensationObservation {
    if sandbox_id.is_none() {
        return RuntimePluginCompensationObservation {
            state: RuntimePluginCompensationState::MissingBinding,
            realized_latency_samples: None,
            tail_samples: None,
        };
    }
    if sandbox.is_some_and(|sandbox| {
        matches!(
            sandbox.state,
            RuntimePluginLifecycleState::Faulted
                | RuntimePluginLifecycleState::Restarting
                | RuntimePluginLifecycleState::Quarantined
                | RuntimePluginLifecycleState::Degraded
        ) || !sandbox.degraded_reasons.is_empty()
    }) {
        return RuntimePluginCompensationObservation {
            state: RuntimePluginCompensationState::Degraded,
            realized_latency_samples: None,
            tail_samples: None,
        };
    }
    RuntimePluginCompensationObservation {
        state: RuntimePluginCompensationState::PendingRender,
        realized_latency_samples: None,
        tail_samples: None,
    }
}
