//! Runtime-facing sandbox broker ensure/record/teardown helpers.

use signal_plugin::PluginIoLayout;

use crate::{
    BrokerFailureStage, PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, RuntimeError, RuntimeErrorKind, SignalRuntime,
};

use super::types::*;

/// Records lifecycle events and instance state for a successfully prepared broker sandbox.
pub fn record_broker_sandbox_prepared(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    record: PreparedSandboxSessionRecord,
) {
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::SandboxHandshaken,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstanceCreated,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstancePrepared,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id: record.plugin_type_id,
        instance_id: record.instance_id,
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: record.processing_epoch,
        processing_sample_rate_hz: Some(record.sample_rate_hz),
        processing_max_block_frames: Some(record.max_block_frames),
        audio_inputs: Some(record.audio_inputs),
        audio_outputs: Some(record.audio_outputs),
        midi_inputs: Some(record.midi_inputs),
        midi_outputs: Some(record.midi_outputs),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::TransportAttached,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_transport(
        request.sandbox_id.as_str(),
        record.lease_id,
        record.region_id,
        PluginSandboxTransportStage::Attached,
        record.processing_epoch,
        record.summary,
    );
}

/// Spawns a broker process, attaches a sandbox session, and records the prepared lifecycle events.
#[allow(clippy::too_many_arguments)] // mirrors the broker attach wire contract one-to-one
pub fn ensure_broker_sandbox_session(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    plugin_type_id: &str,
    default_io_layout: PluginIoLayout,
    fallback_instance_id: &str,
    spawn_config: SandboxBrokerSpawnConfig,
    prepared_summary: Option<String>,
    teardown_summary: Option<String>,
) -> Result<SandboxBrokerSession, RuntimeError> {
    let mut client = SandboxBrokerClientSession::spawn_from_env(&spawn_config)?;
    client.read_startup_receipts()?;
    let attached = client
        .attach(request.sandbox_id.as_str(), fallback_instance_id)
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                None,
                None,
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;

    record_broker_sandbox_prepared(
        runtime,
        request,
        PreparedSandboxSessionRecord {
            plugin_type_id: plugin_type_id.to_string(),
            instance_id: attached.instance_id.clone(),
            sample_rate_hz: runtime.config().sample_rate.0,
            max_block_frames: runtime.config().graph.block_size as u32,
            audio_inputs: default_io_layout.audio_inputs,
            audio_outputs: default_io_layout.audio_outputs,
            midi_inputs: default_io_layout.midi_inputs,
            midi_outputs: default_io_layout.midi_outputs,
            processing_epoch: Some(attached.processing_epoch),
            lease_id: attached.lease_id.clone(),
            region_id: attached.region_id.clone(),
            summary: Some(match &prepared_summary {
                Some(summary) => format!("broker:{} | {}", attached.detail, summary),
                None => format!("broker:{}", attached.detail),
            }),
        },
    );

    Ok(SandboxBrokerSession {
        client,
        attached,
        prepared_summary,
        teardown_summary,
    })
}

/// Prepares a sandbox session via the broker if enabled, otherwise via the direct path.
pub fn ensure_prepared_sandbox_session<BrokerPrepareFn, DirectPrepareFn, AfterBrokerAttachFn>(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    broker_spec: PreparedBrokerSandboxSpec,
    broker_prepare: BrokerPrepareFn,
    direct_prepare: DirectPrepareFn,
    after_broker_attach: AfterBrokerAttachFn,
) -> Result<Option<SandboxBrokerSession>, RuntimeError>
where
    BrokerPrepareFn:
        FnOnce(&mut SignalRuntime) -> Result<(Option<String>, Option<String>), RuntimeError>,
    DirectPrepareFn:
        FnOnce(&mut SignalRuntime) -> Result<PreparedSandboxSessionRecord, RuntimeError>,
    AfterBrokerAttachFn: FnOnce(
        &mut SignalRuntime,
        &PluginSandboxSpec,
        &mut SandboxBrokerSession,
    ) -> Result<(), RuntimeError>,
{
    if SandboxBrokerClientSession::broker_enabled() {
        let (prepared_summary, teardown_summary) = broker_prepare(runtime)?;
        let mut broker_session = ensure_broker_sandbox_session(
            runtime,
            request,
            broker_spec.plugin_type_id.as_str(),
            broker_spec.default_io_layout,
            broker_spec.fallback_instance_id.as_str(),
            broker_spec.spawn_config,
            prepared_summary,
            teardown_summary,
        )?;
        after_broker_attach(runtime, request, &mut broker_session)?;
        Ok(Some(broker_session))
    } else {
        let record = direct_prepare(runtime)?;
        record_broker_sandbox_prepared(runtime, request, record);
        Ok(None)
    }
}

/// Records lifecycle and fault events for a prepare failure caused by a protocol violation.
pub fn record_protocol_violation_prepare_failure(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    plugin_type_id: String,
    instance_id: String,
    default_io_layout: PluginIoLayout,
    lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    detail: String,
) -> RuntimeError {
    if let Some(stage) = lifecycle_stage {
        runtime.record_plugin_sandbox_lifecycle(request.sandbox_id.as_str(), stage, None);
    } else {
        runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
    }
    runtime.record_plugin_sandbox_fault(
        request.sandbox_id.as_str(),
        crate::PluginFaultKind::ProtocolViolation,
        detail.clone(),
        None,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id,
        instance_id,
        lifecycle_state: "Faulted".into(),
        readiness_state: "Faulted".into(),
        degraded_reasons: vec![detail.clone()],
        active: false,
        processing_epoch: None,
        processing_sample_rate_hz: Some(runtime.config().sample_rate.0),
        processing_max_block_frames: Some(runtime.config().graph.block_size as u32),
        audio_inputs: Some(default_io_layout.audio_inputs),
        audio_outputs: Some(default_io_layout.audio_outputs),
        midi_inputs: Some(default_io_layout.midi_inputs),
        midi_outputs: Some(default_io_layout.midi_outputs),
        last_fault: Some(crate::interfaces::PluginSandboxInstanceFaultRecord {
            kind: "ProtocolViolation".into(),
            severity: "Error".into(),
            message: detail.clone(),
        }),
    });
    RuntimeError::new(RuntimeErrorKind::InvalidRequest, detail)
}

/// Records a transport attached event with an execution summary and appends it to the session's prepared summary.
pub fn record_broker_attached_execution_summary(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    session: &mut SandboxBrokerSession,
    execution_summary: String,
) {
    runtime.record_plugin_sandbox_transport(
        request.sandbox_id.as_str(),
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        PluginSandboxTransportStage::Attached,
        Some(session.attached.processing_epoch),
        Some(execution_summary.clone()),
    );
    session.prepared_summary = Some(match session.prepared_summary.take() {
        Some(summary) => format!("{summary} | {execution_summary}"),
        None => execution_summary,
    });
}

/// Records a transport `DetachRequested` stage event for the given sandbox session.
pub fn record_broker_transport_detach_requested(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
) {
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        lease_id,
        region_id,
        PluginSandboxTransportStage::DetachRequested,
        Some(processing_epoch),
        Some(detail.into()),
    );
}

/// Records transport `Detached` and lifecycle teardown events for a broker sandbox.
pub fn record_broker_sandbox_detached(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
    record_instance_destroyed: bool,
) {
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        lease_id,
        region_id,
        PluginSandboxTransportStage::Detached,
        Some(processing_epoch),
        Some(detail.into()),
    );
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::TransportTornDown,
        Some(processing_epoch),
    );
    if record_instance_destroyed {
        runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::InstanceDestroyed,
            Some(processing_epoch),
        );
    }
}

/// Requests teardown from the broker process, records the detach events, and shuts down the child.
pub fn teardown_broker_sandbox_session(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    mut session: SandboxBrokerSession,
) -> Result<(), RuntimeError> {
    record_broker_transport_detach_requested(
        runtime,
        sandbox_id,
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        session.attached.processing_epoch,
        "broker_teardown_requested",
    );

    let teardown_receipt = session.client.request_teardown().map_err(|error| {
        record_broker_failure_and_convert(
            runtime,
            sandbox_id,
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::TransportTeardown,
            error,
        )
    })?;
    if teardown_receipt.state != SandboxBrokerReceiptState::TeardownComplete {
        return Err(record_broker_failure_and_convert(
            runtime,
            sandbox_id,
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::TransportTeardown,
            std::io::Error::other(format!(
                "unexpected broker teardown state: {} ({})",
                teardown_receipt.state, teardown_receipt.detail
            )),
        ));
    }

    let detail = match &session.teardown_summary {
        Some(teardown_summary) => format!("{} | {teardown_summary}", teardown_receipt.detail),
        None => teardown_receipt.detail,
    };
    record_broker_sandbox_detached(
        runtime,
        sandbox_id,
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        session.attached.processing_epoch,
        detail,
        true,
    );

    session.client.shutdown().map_err(|error| {
        record_broker_failure_and_convert(
            runtime,
            sandbox_id,
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::TransportTeardown,
            error,
        )
    })?;
    Ok(())
}
