use signal_runtime::{
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, SignalRuntime,
};

pub(crate) fn ensure_au_sandbox_session(
    runtime: &mut SignalRuntime,
    au: &signal_plugin_au::AuHostAdapter,
    request: &PluginSandboxSpec,
) {
    let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
        return;
    };
    let Some(discovered) = au.discover_plugin_type(plugin_type_id) else {
        return;
    };
    let instance = au.instantiate_plugin(
        &discovered,
        &format!("instance:local:au:{}", request.sandbox_id),
    );
    let session = au.prepare_session(
        &instance,
        runtime.config().sample_rate.0,
        runtime.config().graph.block_size as u32,
    );

    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::SandboxHandshaken,
        None,
    );
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
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstancePrepared,
        None,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id: instance.plugin_type_id.0.clone(),
        instance_id: instance.instance_id.0.clone(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: None,
        processing_sample_rate_hz: Some(session.sample_rate_hz),
        processing_max_block_frames: Some(session.max_block_frames),
        audio_inputs: Some(session.io_layout.audio_inputs),
        audio_outputs: Some(session.io_layout.audio_outputs),
        midi_inputs: Some(session.io_layout.midi_inputs),
        midi_outputs: Some(session.io_layout.midi_outputs),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::TransportAttached,
        None,
    );
    runtime.record_plugin_sandbox_transport(
        request.sandbox_id.as_str(),
        format!("lease:{}", request.sandbox_id),
        format!("region:{}", request.sandbox_id),
        PluginSandboxTransportStage::Attached,
        None,
        Some(session.summary),
    );
}

pub(crate) fn ensure_vst3_sandbox_session(
    runtime: &mut SignalRuntime,
    vst3: &signal_plugin_vst3::Vst3HostAdapter,
    request: &PluginSandboxSpec,
) {
    let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
        return;
    };
    let Some(discovered) = vst3.discover_plugin_type(plugin_type_id) else {
        return;
    };
    let instance = vst3.instantiate_plugin(
        &discovered,
        &format!("instance:local:vst3:{}", request.sandbox_id),
    );
    let session = vst3.prepare_session(
        &instance,
        runtime.config().sample_rate.0,
        runtime.config().graph.block_size as u32,
    );

    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::SandboxHandshaken,
        None,
    );
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
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstancePrepared,
        None,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id: instance.plugin_type_id.0.clone(),
        instance_id: instance.instance_id.0.clone(),
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: None,
        processing_sample_rate_hz: Some(session.sample_rate_hz),
        processing_max_block_frames: Some(session.max_block_frames),
        audio_inputs: Some(session.io_layout.audio_inputs),
        audio_outputs: Some(session.io_layout.audio_outputs),
        midi_inputs: Some(session.io_layout.midi_inputs),
        midi_outputs: Some(session.io_layout.midi_outputs),
        last_fault: None,
    });
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::TransportAttached,
        None,
    );
    runtime.record_plugin_sandbox_transport(
        request.sandbox_id.as_str(),
        format!("lease:{}", request.sandbox_id),
        format!("region:{}", request.sandbox_id),
        PluginSandboxTransportStage::Attached,
        None,
        Some(session.summary),
    );
}
