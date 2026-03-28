use signal_plugin_au::AuHostAdapter;
use signal_plugin_lv2::Lv2HostAdapter;
use signal_plugin_vst3::Vst3HostAdapter;
use signal_runtime::{
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, SignalRuntime,
};

fn record_prepared_sandbox_session(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    plugin_type_id: String,
    instance_id: String,
    sample_rate_hz: u32,
    max_block_frames: u32,
    audio_inputs: u16,
    audio_outputs: u16,
    midi_inputs: u16,
    midi_outputs: u16,
    summary: String,
) {
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
        plugin_type_id,
        instance_id,
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: None,
        processing_sample_rate_hz: Some(sample_rate_hz),
        processing_max_block_frames: Some(max_block_frames),
        audio_inputs: Some(audio_inputs),
        audio_outputs: Some(audio_outputs),
        midi_inputs: Some(midi_inputs),
        midi_outputs: Some(midi_outputs),
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
        Some(summary),
    );
}

pub(crate) fn ensure_au_sandbox_session(
    runtime: &mut SignalRuntime,
    au: &AuHostAdapter,
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
        &format!("instance:server:au:{}", request.sandbox_id),
    );
    let session = au.prepare_session(
        &instance,
        runtime.config().sample_rate.0,
        runtime.config().graph.block_size as u32,
    );
    record_prepared_sandbox_session(
        runtime,
        request,
        instance.plugin_type_id.0.clone(),
        instance.instance_id.0.clone(),
        session.sample_rate_hz,
        session.max_block_frames,
        session.io_layout.audio_inputs,
        session.io_layout.audio_outputs,
        session.io_layout.midi_inputs,
        session.io_layout.midi_outputs,
        session.summary,
    );
}

pub(crate) fn ensure_lv2_sandbox_session(
    runtime: &mut SignalRuntime,
    lv2: &Lv2HostAdapter,
    request: &PluginSandboxSpec,
) {
    let Some(plugin_type_id) = request.plugin_type_id.as_deref() else {
        return;
    };
    let Some(discovered) = lv2.discover_plugin_type(plugin_type_id) else {
        return;
    };
    let instance = lv2.instantiate_plugin(
        &discovered,
        &format!("instance:server:lv2:{}", request.sandbox_id),
    );
    let session = lv2.prepare_session(
        &instance,
        runtime.config().sample_rate.0,
        runtime.config().graph.block_size as u32,
    );
    record_prepared_sandbox_session(
        runtime,
        request,
        instance.plugin_type_id.0.clone(),
        instance.instance_id.0.clone(),
        session.sample_rate_hz,
        session.max_block_frames,
        session.io_layout.audio_inputs,
        session.io_layout.audio_outputs,
        session.io_layout.midi_inputs,
        session.io_layout.midi_outputs,
        session.summary,
    );
}

pub(crate) fn ensure_vst3_sandbox_session(
    runtime: &mut SignalRuntime,
    vst3: &Vst3HostAdapter,
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
        &format!("instance:server:vst3:{}", request.sandbox_id),
    );
    let session = vst3.prepare_session(
        &instance,
        runtime.config().sample_rate.0,
        runtime.config().graph.block_size as u32,
    );
    record_prepared_sandbox_session(
        runtime,
        request,
        instance.plugin_type_id.0.clone(),
        instance.instance_id.0.clone(),
        session.sample_rate_hz,
        session.max_block_frames,
        session.io_layout.audio_inputs,
        session.io_layout.audio_outputs,
        session.io_layout.midi_inputs,
        session.io_layout.midi_outputs,
        session.summary,
    );
}
