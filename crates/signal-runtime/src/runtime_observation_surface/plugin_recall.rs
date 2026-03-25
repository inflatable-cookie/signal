use super::*;

pub(super) fn runtime_plugin_interchange_snapshot_from_snapshot(
    sandbox: Option<&RuntimePluginSandboxSnapshot>,
    discovered_types: &[RuntimePluginDiscoveredTypeRecord],
) -> RuntimePluginInterchangeSnapshot {
    let preset_descriptor = sandbox.and_then(|sandbox| sandbox.preset_descriptor.clone());
    let ara_context = sandbox.and_then(|sandbox| sandbox.ara_context.as_ref());
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
        } else if ara_context.is_some() {
            RuntimePluginRecallPortabilityClass::ContextOnly
        } else {
            RuntimePluginRecallPortabilityClass::NativeOnly
        }
    } else if ara_context.is_some() {
        RuntimePluginRecallPortabilityClass::ContextOnly
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
        preset_descriptor,
        summary: format!(
            "class={:?} shared_payload={} native_supplement={} preset={:?}",
            portability_class,
            shared_payload_available,
            native_supplement_required,
            sandbox
                .and_then(|sandbox| sandbox.preset_descriptor.as_ref())
                .and_then(|descriptor| descriptor.label.as_deref()),
        ),
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

    let mut snapshot = RuntimePluginRecallSnapshot {
        state,
        payload: runtime_plugin_recall_payload(sandbox_id, sandbox, discovered_types),
        summary: String::new(),
    };
    snapshot.summary = format!(
        "state={:?} sandbox={:?} plugin={:?}/{:?} lifecycle={:?}/{:?}/{:?} readiness={:?} recoveries={} restarts={} faults={} restart_intent={:?} stop_reason={:?} fault_kind={:?} portability={:?} ara={}",
        snapshot.state,
        snapshot.payload.sandbox_id.as_deref(),
        snapshot.payload.plugin_type_id.as_deref(),
        snapshot.payload.plugin_format,
        snapshot.payload.lifecycle_state,
        snapshot.payload.lifecycle_stage,
        snapshot.payload.transport_stage,
        snapshot.payload.readiness_state.as_deref(),
        snapshot.payload.recovery_count,
        snapshot.payload.restart_count,
        snapshot.payload.fault_count,
        snapshot.payload.last_restart_intent,
        snapshot.payload.last_stop_reason,
        snapshot.payload.last_fault_kind,
        snapshot.payload.interchange.portability_class,
        snapshot.payload.ara_context.is_some(),
    );
    snapshot
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
        ara_context: sandbox.and_then(|sandbox| sandbox.ara_context.clone()),
    }
}

pub(super) fn runtime_plugin_tail_remaining_samples(
    realized: &RuntimePluginRenderedNodeState,
    current_block_sequence: Option<u64>,
    frame_count: usize,
) -> Option<u32> {
    let current_block_sequence = current_block_sequence?;
    let consumed_samples = current_block_sequence
        .saturating_sub(realized.block_sequence)
        .saturating_mul(frame_count.max(1) as u64);
    Some(
        realized
            .tail_samples
            .saturating_sub(consumed_samples.min(u64::from(u32::MAX)) as u32),
    )
}

pub(super) fn runtime_plugin_compensation_observation(
    sandbox_id: Option<&str>,
    sandbox: Option<&RuntimePluginSandboxSnapshot>,
    realized: Option<&RuntimePluginRenderedNodeState>,
    current_block_sequence: Option<u64>,
    current_frame_count: usize,
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
    match realized {
        Some(realized) if realized.bypassed => RuntimePluginCompensationObservation {
            state: RuntimePluginCompensationState::Bypassed,
            realized_latency_samples: Some(realized.latency_samples),
            tail_samples: Some(realized.tail_samples),
        },
        Some(realized) => {
            let tail_remaining = runtime_plugin_tail_remaining_samples(
                realized,
                current_block_sequence,
                current_frame_count,
            );
            let state = match current_block_sequence
                .unwrap_or(realized.block_sequence)
                .saturating_sub(realized.block_sequence)
            {
                0 => RuntimePluginCompensationState::Compensated,
                _ if tail_remaining.unwrap_or(0) > 0 => RuntimePluginCompensationState::Settling,
                _ => RuntimePluginCompensationState::PendingRender,
            };
            RuntimePluginCompensationObservation {
                state,
                realized_latency_samples: matches!(
                    state,
                    RuntimePluginCompensationState::Compensated
                        | RuntimePluginCompensationState::Settling
                )
                .then_some(realized.latency_samples),
                tail_samples: matches!(
                    state,
                    RuntimePluginCompensationState::Compensated
                        | RuntimePluginCompensationState::Settling
                )
                .then_some(tail_remaining.unwrap_or(realized.tail_samples)),
            }
        }
        None => RuntimePluginCompensationObservation {
            state: RuntimePluginCompensationState::PendingRender,
            realized_latency_samples: None,
            tail_samples: None,
        },
    }
}
