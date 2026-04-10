use signal_ipc::{PluginMessagePayload, SharedMemoryBroker};
use signal_plugin_au::{AuDiscoveredPluginType, AuHostAdapter};
use signal_plugin_clap::{
    ClapDiscoveredPluginType, ClapPluginHostAdapter, ClapSandboxLifecycleHarness,
};
use signal_plugin_vst3::{Vst3DiscoveredPluginType, Vst3HostAdapter};
use signal_runtime::{
    ensure_prepared_sandbox_session, record_protocol_violation_prepare_failure,
    run_vst3_broker_execution_sequence, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PreparedBrokerSandboxSpec, PreparedSandboxSessionRecord, RuntimeError, SandboxBrokerFlavor,
    SandboxBrokerSpawnConfig, SignalRuntime,
};

pub(crate) use signal_runtime::{teardown_broker_sandbox_session, SandboxBrokerSession};

pub(crate) fn ensure_clap_sandbox_session(
    runtime: &mut SignalRuntime,
    broker: &SharedMemoryBroker,
    clap: &ClapPluginHostAdapter,
    discovered: &ClapDiscoveredPluginType,
    request: &PluginSandboxSpec,
) -> Result<Option<SandboxBrokerSession>, RuntimeError> {
    let instance_id = format!("instance:local:clap:{}", request.sandbox_id);
    let protocol = signal_plugin_clap::ClapBlockProtocol::new(
        discovered.plugin_type_id.0.clone(),
        instance_id.clone(),
        discovered.default_io_layout,
        2048,
    );
    let mut harness = ClapSandboxLifecycleHarness::with_adapter(clap.clone());
    let messages = protocol
        .lifecycle_sequence(
            broker,
            request.sandbox_id.as_str(),
            runtime.config().sample_rate.0,
            runtime.config().graph.block_size as u32,
            1,
        )
        .map_err(|error| {
            record_clap_prepare_failure(
                runtime,
                request,
                discovered,
                &instance_id,
                None,
                format!("clap_lifecycle_sequence:{error}"),
            )
        })?;

    let mut shared_memory_lease_id = None;
    let mut region_id = None;
    let mut processing_epoch = None;

    for message in messages {
        let response = harness.handle(message).map_err(|error| {
            record_clap_prepare_failure(
                runtime,
                request,
                discovered,
                &instance_id,
                None,
                format!("clap_prepare:{error:?}"),
            )
        })?;

        if let Some(stage) = clap_lifecycle_stage_for_response(&response.payload) {
            runtime.record_plugin_sandbox_lifecycle(request.sandbox_id.as_str(), stage, None);
        }

        if let Some(state) = super::plugin_instance_state_record_from_response(
            request.sandbox_id.as_str(),
            None,
            &response,
        ) {
            runtime.record_plugin_sandbox_instance_state(state);
        }

        if let PluginMessagePayload::PrepareInstanceResponse {
            processing_epoch: response_epoch,
            shared_memory_lease_id: lease_id,
            shared_memory_transport,
            ..
        } = &response.payload
        {
            processing_epoch = Some(*response_epoch);
            shared_memory_lease_id = Some(lease_id.clone());
            region_id = Some(shared_memory_transport.region_id.clone());
        }
    }

    let heartbeat = harness
        .handle(protocol.heartbeat_request(request.sandbox_id.as_str(), processing_epoch))
        .map_err(|error| {
            record_clap_prepare_failure(
                runtime,
                request,
                discovered,
                &instance_id,
                Some(PluginSandboxLifecycleStage::InstancePrepared),
                format!("clap_heartbeat:{error:?}"),
            )
        })?;
    if let Some(state) = super::plugin_instance_state_record_from_response(
        request.sandbox_id.as_str(),
        processing_epoch,
        &heartbeat,
    ) {
        runtime.record_plugin_sandbox_instance_state(state);
    }

    harness.teardown_active_transport().map_err(|error| {
        record_clap_prepare_failure(
            runtime,
            request,
            discovered,
            &instance_id,
            Some(PluginSandboxLifecycleStage::TransportAttached),
            format!("clap_transport_cleanup:{error}"),
        )
    })?;

    signal_runtime::record_broker_sandbox_prepared(
        runtime,
        request,
        PreparedSandboxSessionRecord {
            plugin_type_id: discovered.plugin_type_id.0.clone(),
            instance_id,
            sample_rate_hz: runtime.config().sample_rate.0,
            max_block_frames: runtime.config().graph.block_size as u32,
            audio_inputs: discovered.default_io_layout.audio_inputs,
            audio_outputs: discovered.default_io_layout.audio_outputs,
            midi_inputs: discovered.default_io_layout.midi_inputs,
            midi_outputs: discovered.default_io_layout.midi_outputs,
            processing_epoch,
            lease_id: shared_memory_lease_id
                .unwrap_or_else(|| format!("lease:{}", request.sandbox_id)),
            region_id: region_id.unwrap_or_else(|| format!("region:{}", request.sandbox_id)),
            lv2_prepared_negotiation: None,
            summary: Some(format!(
                "clap:prepared plugin_type={} io={:?}",
                discovered.plugin_type_id.0, discovered.default_io_layout
            )),
        },
    );
    let _ = clap;
    Ok(None)
}

fn record_clap_prepare_failure(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    discovered: &ClapDiscoveredPluginType,
    instance_id: &str,
    lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    detail: String,
) -> RuntimeError {
    record_protocol_violation_prepare_failure(
        runtime,
        request,
        discovered.plugin_type_id.0.clone(),
        instance_id.to_string(),
        discovered.default_io_layout,
        lifecycle_stage,
        detail,
    )
}

fn clap_lifecycle_stage_for_response(
    payload: &PluginMessagePayload,
) -> Option<PluginSandboxLifecycleStage> {
    match payload {
        PluginMessagePayload::SandboxHandshakeResponse { .. } => {
            Some(PluginSandboxLifecycleStage::SandboxHandshaken)
        }
        PluginMessagePayload::LoadPluginTypeResponse { .. } => {
            Some(PluginSandboxLifecycleStage::PluginTypeLoaded)
        }
        PluginMessagePayload::CreateInstanceResponse { .. } => {
            Some(PluginSandboxLifecycleStage::InstanceCreated)
        }
        PluginMessagePayload::PrepareInstanceResponse { .. } => {
            Some(PluginSandboxLifecycleStage::InstancePrepared)
        }
        PluginMessagePayload::ActivateInstanceResponse { .. } => {
            Some(PluginSandboxLifecycleStage::TransportAttached)
        }
        _ => None,
    }
}

pub(crate) fn ensure_au_sandbox_session(
    runtime: &mut SignalRuntime,
    au: &AuHostAdapter,
    discovered: &AuDiscoveredPluginType,
    request: &PluginSandboxSpec,
) -> Result<Option<SandboxBrokerSession>, RuntimeError> {
    let instance_id = format!("instance:local:au:{}", request.sandbox_id);
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
            .unwrap_or_else(|| format!("instance:local:au:{}", request.sandbox_id)),
        discovered.default_io_layout,
        lifecycle_stage,
        detail,
    )
}

pub(crate) fn ensure_vst3_sandbox_session(
    runtime: &mut SignalRuntime,
    vst3: &Vst3HostAdapter,
    discovered: &Vst3DiscoveredPluginType,
    request: &PluginSandboxSpec,
) -> Result<Option<SandboxBrokerSession>, RuntimeError> {
    let instance_id = format!("instance:local:vst3:{}", request.sandbox_id);
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
