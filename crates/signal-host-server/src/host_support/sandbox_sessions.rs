use signal_plugin_au::{AuDiscoveredPluginType, AuHostAdapter};
use signal_plugin_lv2::{
    Lv2DiscoveredPluginType, Lv2ExtensionNegotiationState, Lv2HostAdapter, Lv2PatchExchangePosture,
    Lv2PreparationFaultMode, Lv2UridNegotiationPosture, Lv2WorkerNegotiationPosture,
};
use signal_plugin_vst3::{Vst3DiscoveredPluginType, Vst3HostAdapter};
use signal_runtime::{
    ensure_broker_sandbox_session, ensure_prepared_sandbox_session,
    record_broker_attached_execution_summary, record_broker_sandbox_prepared,
    record_protocol_violation_prepare_failure, run_vst3_broker_execution_sequence,
    BrokerFailureStage, PluginFaultKind, PluginSandboxInstanceFaultRecord,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PreparedBrokerSandboxSpec, PreparedSandboxSessionRecord, RuntimeError,
    RuntimeLv2ExtensionNegotiationState, RuntimeLv2PatchExchangePosture,
    RuntimeLv2PreparedNegotiationRecord, RuntimeLv2UridNegotiationPosture, RuntimeLv2WorkerPosture,
    SandboxBrokerClientSession, SandboxBrokerFlavor, SandboxBrokerSpawnConfig, SignalRuntime,
};

use super::record_broker_failure_and_convert;

pub(crate) use signal_runtime::{teardown_broker_sandbox_session, SandboxBrokerSession};

pub(crate) fn ensure_au_sandbox_session(
    runtime: &mut SignalRuntime,
    au: &AuHostAdapter,
    discovered: &AuDiscoveredPluginType,
    request: &PluginSandboxSpec,
) -> Result<Option<SandboxBrokerSession>, RuntimeError> {
    let instance_id = format!("instance:server:au:{}", request.sandbox_id);
    ensure_prepared_sandbox_session(
        runtime,
        request,
        PreparedBrokerSandboxSpec {
            plugin_type_id: discovered.plugin_type_id.0.clone(),
            default_io_layout: discovered.default_io_layout,
            fallback_instance_id: instance_id.clone(),
            flavor: SandboxBrokerFlavor::Au,
            spawn_config: SandboxBrokerSpawnConfig {
                env: vec![
                    (
                        "SIGNAL_PLUGIN_SANDBOX_AU_PLUGIN_TYPE_ID".into(),
                        discovered.plugin_type_id.0.clone(),
                    ),
                    (
                        "SIGNAL_PLUGIN_SANDBOX_AU_BUNDLE_ROOT".into(),
                        discovered.bundle_root.clone(),
                    ),
                    (
                        "SIGNAL_PLUGIN_SANDBOX_AU_INSTANCE_ID".into(),
                        instance_id.clone(),
                    ),
                ],
            },
            lv2_prepared_negotiation: None,
        },
        |runtime| {
            let broker_instance =
                au.instantiate_plugin(discovered, &instance_id)
                    .map_err(|detail| {
                        record_au_prepare_failure(runtime, request, discovered, None, None, detail)
                    })?;
            let state_snapshot = au.store_state_snapshot(&broker_instance);
            let activation = au
                .activate_instance(
                    &broker_instance,
                    runtime.config().sample_rate.0,
                    runtime.config().graph.block_size as u32,
                    Some(&state_snapshot),
                )
                .map_err(|detail| {
                    record_au_prepare_failure(
                        runtime,
                        request,
                        discovered,
                        Some(&broker_instance.instance_id.0),
                        Some(PluginSandboxLifecycleStage::InstancePrepared),
                        detail,
                    )
                })?;
            let teardown = au.teardown_instance(&broker_instance, Some(&state_snapshot));
            Ok((
                Some(format!(
                    "{} | {}",
                    state_snapshot.summary, activation.summary
                )),
                Some(teardown.summary),
            ))
        },
        |runtime| {
            let instance = au
                .instantiate_plugin(discovered, &instance_id)
                .map_err(|detail| {
                    record_au_prepare_failure(runtime, request, discovered, None, None, detail)
                })?;
            runtime.record_plugin_sandbox_lifecycle(
                request.sandbox_id.as_str(),
                PluginSandboxLifecycleStage::PluginTypeLoaded,
                None,
            );
            runtime.record_plugin_sandbox_lifecycle(
                request.sandbox_id.as_str(),
                PluginSandboxLifecycleStage::InstanceCreated,
                None,
            );
            let session = au
                .prepare_session(
                    &instance,
                    runtime.config().sample_rate.0,
                    runtime.config().graph.block_size as u32,
                )
                .map_err(|detail| {
                    record_au_prepare_failure(
                        runtime,
                        request,
                        discovered,
                        Some(&instance.instance_id.0),
                        Some(PluginSandboxLifecycleStage::InstanceCreated),
                        detail,
                    )
                })?;
            let state_snapshot = au.store_state_snapshot(&instance);
            let activation = au
                .activate_instance(
                    &instance,
                    runtime.config().sample_rate.0,
                    runtime.config().graph.block_size as u32,
                    Some(&state_snapshot),
                )
                .map_err(|detail| {
                    record_au_prepare_failure(
                        runtime,
                        request,
                        discovered,
                        Some(&instance.instance_id.0),
                        Some(PluginSandboxLifecycleStage::InstancePrepared),
                        detail,
                    )
                })?;
            Ok(PreparedSandboxSessionRecord {
                plugin_type_id: instance.plugin_type_id.0.clone(),
                instance_id: instance.instance_id.0.clone(),
                sample_rate_hz: session.sample_rate_hz,
                max_block_frames: session.max_block_frames,
                audio_inputs: session.io_layout.audio_inputs,
                audio_outputs: session.io_layout.audio_outputs,
                midi_inputs: session.io_layout.midi_inputs,
                midi_outputs: session.io_layout.midi_outputs,
                processing_epoch: None,
                lease_id: format!("lease:{}", request.sandbox_id),
                region_id: format!("region:{}", request.sandbox_id),
                lv2_prepared_negotiation: None,
                summary: Some(format!(
                    "{} | {} | {}",
                    session.summary, state_snapshot.summary, activation.summary
                )),
            })
        },
        |_, _, _| Ok(()),
    )
}

fn record_au_prepare_failure(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    discovered: &AuDiscoveredPluginType,
    instance_id: Option<&str>,
    lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    detail: String,
) -> RuntimeError {
    record_protocol_violation_prepare_failure(
        runtime,
        request,
        discovered.plugin_type_id.0.clone(),
        instance_id
            .map(str::to_string)
            .unwrap_or_else(|| format!("instance:server:au:{}", request.sandbox_id)),
        discovered.default_io_layout,
        lifecycle_stage,
        detail,
    )
}

pub(crate) fn ensure_lv2_sandbox_session(
    runtime: &mut SignalRuntime,
    lv2: &Lv2HostAdapter,
    discovered: &Lv2DiscoveredPluginType,
    request: &PluginSandboxSpec,
) -> Result<Option<SandboxBrokerSession>, RuntimeError> {
    let instance = lv2.instantiate_plugin(
        discovered,
        &format!("instance:server:lv2:{}", request.sandbox_id),
    );
    let session = lv2.prepare_session(
        &instance,
        runtime.config().sample_rate.0,
        runtime.config().graph.block_size as u32,
    );
    let lv2_prepared_negotiation =
        runtime_lv2_prepared_negotiation_record(&session.extension_preparation);
    if let Some(prepare_fault) = discovered.prepare_fault {
        return Err(record_lv2_prepare_failure(
            runtime,
            request,
            discovered,
            &instance.instance_id.0,
            runtime_faulted_lv2_prepared_negotiation_record(prepare_fault),
        ));
    }
    if SandboxBrokerClientSession::broker_enabled() {
        let teardown = lv2.teardown_instance(&instance, &session);
        let mut broker_session = ensure_broker_sandbox_session(
            runtime,
            request,
            discovered.plugin_type_id.0.as_str(),
            discovered.default_io_layout,
            &format!("instance:server:lv2:{}", request.sandbox_id),
            SandboxBrokerFlavor::Lv2,
            SandboxBrokerSpawnConfig {
                env: vec![
                    (
                        "SIGNAL_PLUGIN_SANDBOX_LV2_PLUGIN_TYPE_ID".into(),
                        discovered.plugin_type_id.0.clone(),
                    ),
                    (
                        "SIGNAL_PLUGIN_SANDBOX_LV2_BUNDLE_ROOT".into(),
                        discovered.bundle_root.clone(),
                    ),
                    (
                        "SIGNAL_PLUGIN_SANDBOX_LV2_INSTANCE_ID".into(),
                        format!("instance:server:lv2:{}", request.sandbox_id),
                    ),
                ],
            },
            Some(session.summary.clone()),
            Some(teardown.summary),
            Some(lv2_prepared_negotiation),
        )?;
        let execution = broker_session
            .client
            .request_lv2_execution_stream()
            .map_err(|error| {
                record_broker_failure_and_convert(
                    runtime,
                    request.sandbox_id.as_str(),
                    Some(broker_session.attached.lease_id.clone()),
                    Some(broker_session.attached.processing_epoch),
                    None,
                    BrokerFailureStage::PreparePlanCreate,
                    error,
                )
            })?;
        record_broker_attached_execution_summary(
            runtime,
            request,
            &mut broker_session,
            format!("broker:{}", execution.detail),
        );
        return Ok(Some(broker_session));
    }
    record_broker_sandbox_prepared(
        runtime,
        request,
        PreparedSandboxSessionRecord {
            plugin_type_id: instance.plugin_type_id.0.clone(),
            instance_id: instance.instance_id.0.clone(),
            sample_rate_hz: session.sample_rate_hz,
            max_block_frames: session.max_block_frames,
            audio_inputs: session.io_layout.audio_inputs,
            audio_outputs: session.io_layout.audio_outputs,
            midi_inputs: session.io_layout.midi_inputs,
            midi_outputs: session.io_layout.midi_outputs,
            processing_epoch: None,
            lease_id: format!("lease:{}", request.sandbox_id),
            region_id: format!("region:{}", request.sandbox_id),
            lv2_prepared_negotiation: Some(lv2_prepared_negotiation),
            summary: Some(session.summary),
        },
    );
    Ok(None)
}

fn record_lv2_prepare_failure(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    discovered: &Lv2DiscoveredPluginType,
    instance_id: &str,
    negotiation: RuntimeLv2PreparedNegotiationRecord,
) -> RuntimeError {
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        None,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstanceCreated,
        None,
    );
    runtime.record_plugin_sandbox_lv2_prepared_negotiation(
        request.sandbox_id.as_str(),
        negotiation.clone(),
    );
    runtime.record_plugin_sandbox_fault(
        request.sandbox_id.as_str(),
        PluginFaultKind::ProtocolViolation,
        negotiation.summary.clone(),
        None,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id: discovered.plugin_type_id.0.clone(),
        instance_id: instance_id.into(),
        lifecycle_state: "Faulted".into(),
        readiness_state: "Faulted".into(),
        degraded_reasons: vec![negotiation.summary.clone()],
        active: false,
        processing_epoch: None,
        processing_sample_rate_hz: Some(runtime.config().sample_rate.0),
        processing_max_block_frames: Some(runtime.config().graph.block_size as u32),
        audio_inputs: Some(discovered.default_io_layout.audio_inputs),
        audio_outputs: Some(discovered.default_io_layout.audio_outputs),
        midi_inputs: Some(discovered.default_io_layout.midi_inputs),
        midi_outputs: Some(discovered.default_io_layout.midi_outputs),
        last_fault: Some(PluginSandboxInstanceFaultRecord {
            kind: "ProtocolViolation".into(),
            severity: "Error".into(),
            message: negotiation.summary.clone(),
        }),
    });
    RuntimeError::new(
        signal_runtime::RuntimeErrorKind::InvalidRequest,
        negotiation.summary,
    )
}

pub(crate) fn ensure_vst3_sandbox_session(
    runtime: &mut SignalRuntime,
    vst3: &Vst3HostAdapter,
    discovered: &Vst3DiscoveredPluginType,
    request: &PluginSandboxSpec,
) -> Result<Option<SandboxBrokerSession>, RuntimeError> {
    let instance_id = format!("instance:server:vst3:{}", request.sandbox_id);
    ensure_prepared_sandbox_session(
        runtime,
        request,
        PreparedBrokerSandboxSpec {
            plugin_type_id: discovered.plugin_type_id.0.clone(),
            default_io_layout: discovered.default_io_layout,
            fallback_instance_id: instance_id.clone(),
            flavor: SandboxBrokerFlavor::Vst3,
            spawn_config: SandboxBrokerSpawnConfig {
                env: vec![
                    (
                        "SIGNAL_PLUGIN_SANDBOX_VST3_PLUGIN_TYPE_ID".into(),
                        discovered.plugin_type_id.0.clone(),
                    ),
                    (
                        "SIGNAL_PLUGIN_SANDBOX_VST3_MODULE_ROOT".into(),
                        discovered.module_root.clone(),
                    ),
                    (
                        "SIGNAL_PLUGIN_SANDBOX_VST3_INSTANCE_ID".into(),
                        instance_id.clone(),
                    ),
                ],
            },
            lv2_prepared_negotiation: None,
        },
        |runtime| {
            let broker_instance =
                vst3.instantiate_plugin(discovered, &instance_id)
                    .map_err(|error| {
                        RuntimeError::new(
                            signal_runtime::RuntimeErrorKind::InvalidRequest,
                            error.to_string(),
                        )
                    })?;
            let state_snapshot = vst3
                .store_state_snapshot(&broker_instance)
                .map_err(|error| {
                    RuntimeError::new(
                        signal_runtime::RuntimeErrorKind::InvalidRequest,
                        error.to_string(),
                    )
                })?;
            let _activation = vst3
                .activate_instance(
                    &broker_instance,
                    runtime.config().sample_rate.0,
                    runtime.config().graph.block_size as u32,
                    Some(&state_snapshot),
                )
                .map_err(|error| {
                    RuntimeError::new(
                        signal_runtime::RuntimeErrorKind::InvalidRequest,
                        error.to_string(),
                    )
                })?;
            let _teardown = vst3
                .teardown_instance(&broker_instance, Some(&state_snapshot))
                .map_err(|error| {
                    RuntimeError::new(
                        signal_runtime::RuntimeErrorKind::InvalidRequest,
                        error.to_string(),
                    )
                })?;
            Ok((None, None))
        },
        |runtime| {
            let instance = vst3
                .instantiate_plugin(discovered, &instance_id)
                .map_err(|error| {
                    RuntimeError::new(
                        signal_runtime::RuntimeErrorKind::InvalidRequest,
                        error.to_string(),
                    )
                })?;
            let state_snapshot = vst3.store_state_snapshot(&instance).map_err(|error| {
                RuntimeError::new(
                    signal_runtime::RuntimeErrorKind::InvalidRequest,
                    error.to_string(),
                )
            })?;
            let activation = vst3
                .activate_instance(
                    &instance,
                    runtime.config().sample_rate.0,
                    runtime.config().graph.block_size as u32,
                    Some(&state_snapshot),
                )
                .map_err(|error| {
                    RuntimeError::new(
                        signal_runtime::RuntimeErrorKind::InvalidRequest,
                        error.to_string(),
                    )
                })?;
            let session = vst3.prepare_session(
                &instance,
                runtime.config().sample_rate.0,
                runtime.config().graph.block_size as u32,
            );
            Ok(PreparedSandboxSessionRecord {
                plugin_type_id: instance.plugin_type_id.0.clone(),
                instance_id: instance.instance_id.0.clone(),
                sample_rate_hz: session.sample_rate_hz,
                max_block_frames: session.max_block_frames,
                audio_inputs: session.io_layout.audio_inputs,
                audio_outputs: session.io_layout.audio_outputs,
                midi_inputs: session.io_layout.midi_inputs,
                midi_outputs: session.io_layout.midi_outputs,
                processing_epoch: None,
                lease_id: format!("lease:{}", request.sandbox_id),
                region_id: format!("region:{}", request.sandbox_id),
                lv2_prepared_negotiation: None,
                summary: Some(format!(
                    "{} | {} | {}",
                    session.summary, state_snapshot.summary, activation.summary
                )),
            })
        },
        |runtime, request, broker_session| {
            run_vst3_broker_execution_sequence(runtime, request, broker_session)
        },
    )
}

fn runtime_lv2_prepared_negotiation_record(
    record: &signal_plugin_lv2::Lv2ExtensionPreparationRecord,
) -> RuntimeLv2PreparedNegotiationRecord {
    RuntimeLv2PreparedNegotiationRecord {
        worker_posture: match record.worker_posture {
            Lv2WorkerNegotiationPosture::WorkerAbsent => RuntimeLv2WorkerPosture::WorkerAbsent,
            Lv2WorkerNegotiationPosture::WorkerAvailable => {
                RuntimeLv2WorkerPosture::WorkerAvailable
            }
            Lv2WorkerNegotiationPosture::WorkerRequiredAvailable => {
                RuntimeLv2WorkerPosture::WorkerRequiredAvailable
            }
        },
        urid_negotiation_posture: match record.urid_negotiation_posture {
            Lv2UridNegotiationPosture::NotRequired => RuntimeLv2UridNegotiationPosture::NotRequired,
            Lv2UridNegotiationPosture::Negotiated => RuntimeLv2UridNegotiationPosture::Negotiated,
        },
        patch_exchange_posture: match record.patch_exchange_posture {
            Lv2PatchExchangePosture::Absent => RuntimeLv2PatchExchangePosture::Absent,
            Lv2PatchExchangePosture::Supported => RuntimeLv2PatchExchangePosture::Supported,
        },
        extension_negotiation_state: match record.extension_negotiation_state {
            Lv2ExtensionNegotiationState::NotRequired => {
                RuntimeLv2ExtensionNegotiationState::NotRequired
            }
            Lv2ExtensionNegotiationState::Negotiated => {
                RuntimeLv2ExtensionNegotiationState::Negotiated
            }
        },
        summary: record.summary.clone(),
    }
}

fn runtime_faulted_lv2_prepared_negotiation_record(
    mode: Lv2PreparationFaultMode,
) -> RuntimeLv2PreparedNegotiationRecord {
    match mode {
        Lv2PreparationFaultMode::WorkerUnavailable => RuntimeLv2PreparedNegotiationRecord {
            worker_posture: RuntimeLv2WorkerPosture::WorkerUnavailable,
            urid_negotiation_posture: RuntimeLv2UridNegotiationPosture::Negotiated,
            patch_exchange_posture: RuntimeLv2PatchExchangePosture::Supported,
            extension_negotiation_state: RuntimeLv2ExtensionNegotiationState::Unavailable,
            summary:
                "worker=WorkerUnavailable urid=Negotiated patch=Supported negotiation=Unavailable"
                    .into(),
        },
        Lv2PreparationFaultMode::UridUnavailable => RuntimeLv2PreparedNegotiationRecord {
            worker_posture: RuntimeLv2WorkerPosture::WorkerAvailable,
            urid_negotiation_posture: RuntimeLv2UridNegotiationPosture::Unavailable,
            patch_exchange_posture: RuntimeLv2PatchExchangePosture::Supported,
            extension_negotiation_state: RuntimeLv2ExtensionNegotiationState::Unavailable,
            summary:
                "worker=WorkerAvailable urid=Unavailable patch=Supported negotiation=Unavailable"
                    .into(),
        },
        Lv2PreparationFaultMode::PatchUnavailable => RuntimeLv2PreparedNegotiationRecord {
            worker_posture: RuntimeLv2WorkerPosture::WorkerAvailable,
            urid_negotiation_posture: RuntimeLv2UridNegotiationPosture::Negotiated,
            patch_exchange_posture: RuntimeLv2PatchExchangePosture::Unavailable,
            extension_negotiation_state: RuntimeLv2ExtensionNegotiationState::Unavailable,
            summary:
                "worker=WorkerAvailable urid=Negotiated patch=Unavailable negotiation=Unavailable"
                    .into(),
        },
    }
}
