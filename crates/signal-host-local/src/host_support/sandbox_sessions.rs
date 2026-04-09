use signal_plugin_au::{AuDiscoveredPluginType, AuHostAdapter};
use signal_plugin_vst3::{Vst3DiscoveredPluginType, Vst3HostAdapter};
use signal_runtime::{
    ensure_prepared_sandbox_session, record_protocol_violation_prepare_failure,
    run_vst3_broker_execution_sequence, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PreparedBrokerSandboxSpec, PreparedSandboxSessionRecord, RuntimeError, SandboxBrokerFlavor,
    SandboxBrokerSpawnConfig, SignalRuntime,
};

pub(crate) use signal_runtime::{teardown_broker_sandbox_session, SandboxBrokerSession};

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
