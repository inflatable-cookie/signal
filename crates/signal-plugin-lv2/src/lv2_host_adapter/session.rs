use super::*;

impl Lv2HostAdapter {
    pub fn instantiate_plugin(
        &self,
        discovered: &Lv2DiscoveredPluginType,
        instance_id: &str,
    ) -> Lv2InstanceControlSurface {
        Lv2InstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            plugin_uri: discovered.plugin_uri.clone(),
            bundle_root: discovered.bundle_root.clone(),
            manifest_path: discovered.manifest_path.clone(),
            required_features: discovered.required_features.clone(),
            prepare_fault: discovered.prepare_fault,
            default_io_layout: discovered.default_io_layout,
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        }
    }

    pub fn prepare_session(
        &self,
        instance: &Lv2InstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> Lv2ProcessSessionPlan {
        let extension_preparation = lv2_extension_preparation(instance);
        Lv2ProcessSessionPlan {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            plugin_uri: instance.plugin_uri.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: instance.default_io_layout,
            transport: SandboxTransport::SharedMemory,
            bundle_root: instance.bundle_root.clone(),
            manifest_path: instance.manifest_path.clone(),
            extension_preparation: extension_preparation.clone(),
            summary: format!(
                "plugin_type={} uri={} sample_rate={} max_block_frames={} bundle_root={} manifest={} required_features={} extensions={}",
                instance.plugin_type_id.0,
                instance.plugin_uri,
                sample_rate_hz,
                max_block_frames,
                instance.bundle_root,
                instance.manifest_path,
                instance.required_features.join(","),
                extension_preparation.summary,
            ),
        }
    }

    pub fn teardown_instance(
        &self,
        instance: &Lv2InstanceControlSurface,
        session: &Lv2ProcessSessionPlan,
    ) -> Lv2TeardownRecord {
        Lv2TeardownRecord {
            summary: format!(
                "instance={} plugin_type={} bundle_root={} manifest={} teardown=prepared_negotiation_flushed {}",
                instance.instance_id.0,
                instance.plugin_type_id.0,
                instance.bundle_root,
                instance.manifest_path,
                session.extension_preparation.summary,
            ),
        }
    }

    pub fn execute_block(
        &self,
        instance: &Lv2InstanceControlSurface,
        session: &Lv2ProcessSessionPlan,
        block_sequence: u64,
        block_frames: u32,
        patch_message_count: u16,
        midi_event_count: u16,
    ) -> Lv2BlockProcessingRecord {
        let completion_status = if matches!(
            session.extension_preparation.extension_negotiation_state,
            Lv2ExtensionNegotiationState::Negotiated | Lv2ExtensionNegotiationState::NotRequired
        ) {
            "Applied"
        } else {
            "Unavailable"
        };
        Lv2BlockProcessingRecord {
            block_sequence,
            block_frames,
            audio_output_channels: session.io_layout.audio_outputs,
            midi_event_count,
            patch_message_count,
            completion_status: completion_status.into(),
            summary: format!(
                "plugin_type={} instance={} block_sequence={} block_frames={} audio_outputs={} midi_events={} patch_messages={} completion={} {}",
                instance.plugin_type_id.0,
                instance.instance_id.0,
                block_sequence,
                block_frames,
                session.io_layout.audio_outputs,
                midi_event_count,
                patch_message_count,
                completion_status,
                session.extension_preparation.summary,
            ),
        }
    }
}

fn lv2_extension_preparation(
    instance: &Lv2InstanceControlSurface,
) -> Lv2ExtensionPreparationRecord {
    let requires_worker = instance
        .required_features
        .iter()
        .any(|feature| feature == "http://lv2plug.in/ns/ext/worker#schedule");
    let supports_worker = instance
        .required_features
        .iter()
        .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/worker#"));
    let requires_urid = instance
        .required_features
        .iter()
        .any(|feature| feature == "http://lv2plug.in/ns/ext/urid#map");
    let supports_urid = instance
        .required_features
        .iter()
        .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/urid#"));
    let supports_patch = instance
        .required_features
        .iter()
        .any(|feature| feature.starts_with("http://lv2plug.in/ns/ext/patch#"))
        || instance.descriptor.processing_contract.accepts_note_events;

    let worker_posture = if requires_worker {
        Lv2WorkerNegotiationPosture::WorkerRequiredAvailable
    } else if supports_worker {
        Lv2WorkerNegotiationPosture::WorkerAvailable
    } else {
        Lv2WorkerNegotiationPosture::WorkerAbsent
    };
    let urid_negotiation_posture = if requires_urid || supports_urid {
        Lv2UridNegotiationPosture::Negotiated
    } else {
        Lv2UridNegotiationPosture::NotRequired
    };
    let patch_exchange_posture = if supports_patch {
        Lv2PatchExchangePosture::Supported
    } else {
        Lv2PatchExchangePosture::Absent
    };
    let extension_negotiation_state =
        if matches!(worker_posture, Lv2WorkerNegotiationPosture::WorkerAbsent)
            && urid_negotiation_posture == Lv2UridNegotiationPosture::NotRequired
            && patch_exchange_posture == Lv2PatchExchangePosture::Absent
        {
            Lv2ExtensionNegotiationState::NotRequired
        } else {
            Lv2ExtensionNegotiationState::Negotiated
        };

    let mut record = Lv2ExtensionPreparationRecord {
        worker_posture,
        urid_negotiation_posture,
        patch_exchange_posture,
        extension_negotiation_state,
        summary: String::new(),
    };
    record.summary = format!(
        "worker={:?} urid={:?} patch={:?} negotiation={:?}",
        record.worker_posture,
        record.urid_negotiation_posture,
        record.patch_exchange_posture,
        record.extension_negotiation_state,
    );
    record
}
