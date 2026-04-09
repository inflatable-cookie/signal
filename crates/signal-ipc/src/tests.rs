use crate::{
    PluginInstanceStatePayload, PluginIoLayoutPayload, PluginMessageEnvelope, PluginMessageName,
    PluginMessagePayload, PluginProcessConfigurationPayload, RuntimeDomain, SharedMemoryBroker,
    SharedMemoryLayoutPayload, SharedMemoryRegionLifecycleErrorKind, SharedMemoryRegionPayload,
    SharedMemoryTransportKind, SharedMemoryTransportPayload,
};
use std::{
    fs,
    path::PathBuf,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

fn test_broker_root(name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "signal-ipc-tests-{}-{name}-{timestamp}",
        process::id()
    ))
}

#[test]
fn plugin_command_envelope_uses_plugin_domain() {
    let envelope = PluginMessageEnvelope::command(
        PluginMessageName::SandboxHandshake,
        "cid-1",
        PluginMessagePayload::SandboxHandshakeRequest {
            sandbox_id: "sandbox-a".into(),
            format: "clap".into(),
        },
    );

    assert_eq!(envelope.message.domain, RuntimeDomain::Plugin);
    assert_eq!(envelope.message.name, "sandbox.handshake");
}

#[test]
fn plugin_response_envelope_preserves_correlation() {
    let request = PluginMessageEnvelope::command(
        PluginMessageName::SandboxActivateInstance,
        "cid-activate-1",
        PluginMessagePayload::ActivateInstanceRequest {
            sandbox_id: "sandbox-a".into(),
            instance_id: "instance-a".into(),
            processing_epoch: 1,
        },
    );
    let response = PluginMessageEnvelope::response(
        PluginMessageName::SandboxActivateInstance,
        request
            .message
            .correlation_id
            .clone()
            .expect("request correlation"),
        PluginMessagePayload::ActivateInstanceResponse {
            instance_id: "instance-a".into(),
            processing_epoch: 1,
            instance_state: PluginInstanceStatePayload {
                plugin_type_id: "plugin-a".into(),
                instance_id: "instance-a".into(),
                lifecycle_state: "Active".into(),
                readiness_state: "Ready".into(),
                degraded_reasons: Vec::new(),
                active: true,
                processing: Some(PluginProcessConfigurationPayload {
                    sample_rate_hz: 48_000,
                    max_block_frames: 512,
                    io_layout: PluginIoLayoutPayload {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 1,
                    },
                }),
                last_fault: None,
            },
        },
    );

    assert_eq!(
        response.message.correlation_id,
        request.message.correlation_id
    );
    assert_eq!(response.message.name, "sandbox.activateInstance");
}

#[test]
fn heartbeat_envelope_uses_heartbeat_message_name() {
    let envelope = PluginMessageEnvelope::command(
        PluginMessageName::SandboxHeartbeat,
        "cid-heartbeat-1",
        PluginMessagePayload::HeartbeatRequest {
            sandbox_id: "sandbox-a".into(),
            instance_id: Some("instance-a".into()),
            processing_epoch: Some(7),
        },
    );

    assert_eq!(envelope.message.name, "sandbox.heartbeat");
    assert_eq!(envelope.message.domain, RuntimeDomain::Plugin);
}

#[test]
fn shared_memory_broker_round_trips_bytes_across_attachment() {
    let root = test_broker_root("roundtrip");
    let broker = SharedMemoryBroker::new(&root);
    let mut region = broker
        .create_region("lease-roundtrip", 256)
        .expect("create brokered region");

    region.as_mut_slice()[0..4].copy_from_slice(&[1, 2, 3, 4]);
    region.flush().expect("flush brokered region");

    let transport = region.metadata().clone();
    let attached = broker
        .attach_region(&transport)
        .expect("attach brokered region");

    assert_eq!(&attached.as_slice()[0..4], &[1, 2, 3, 4]);
    assert_eq!(attached.file_len().expect("file len"), 256);
    assert_eq!(
        transport.transport_kind,
        SharedMemoryTransportKind::MappedFile
    );

    broker.destroy_region(&transport).expect("destroy region");
    assert!(!std::path::Path::new(&transport.backing_path).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shared_memory_broker_rejects_missing_metadata_sidecar_on_attach() {
    let root = test_broker_root("missing-metadata");
    let broker = SharedMemoryBroker::new(&root);
    let region = broker
        .create_region("lease-metadata-missing", 256)
        .expect("create brokered region");
    let transport = region.metadata().clone();
    let metadata_path = PathBuf::from(format!("{}.meta", transport.backing_path));

    fs::remove_file(&metadata_path).expect("remove metadata sidecar");
    let error = broker
        .attach_region(&transport)
        .expect_err("missing metadata must fail attach");
    assert_eq!(
        error.kind(),
        SharedMemoryRegionLifecycleErrorKind::MissingMetadata
    );

    let _ = fs::remove_file(&transport.backing_path);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shared_memory_broker_rejects_size_mismatch_on_attach() {
    let root = test_broker_root("size-mismatch");
    let broker = SharedMemoryBroker::new(&root);
    let region = broker
        .create_region("lease-size-mismatch", 256)
        .expect("create brokered region");
    let transport = region.metadata().clone();

    fs::write(
        format!("{}.meta", transport.backing_path),
        format!(
            "region_id={}\nlease_id=lease-size-mismatch\ntotal_bytes={}\nowner_pid={}\n",
            transport.region_id,
            128,
            process::id()
        ),
    )
    .expect("rewrite metadata sidecar");

    let error = broker
        .attach_region(&transport)
        .expect_err("mismatched metadata size must fail attach");
    assert_eq!(
        error.kind(),
        SharedMemoryRegionLifecycleErrorKind::SizeMismatch
    );

    let _ = fs::remove_file(&transport.backing_path);
    let _ = fs::remove_file(format!("{}.meta", transport.backing_path));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shared_memory_broker_rejects_missing_backing_file_on_destroy() {
    let root = test_broker_root("missing-backing-destroy");
    let broker = SharedMemoryBroker::new(&root);
    let region = broker
        .create_region("lease-destroy-missing-backing", 256)
        .expect("create brokered region");
    let transport = region.metadata().clone();

    fs::remove_file(&transport.backing_path).expect("remove backing file");
    let error = broker
        .destroy_region(&transport)
        .expect_err("missing backing file must fail destroy");
    assert_eq!(
        error.kind(),
        SharedMemoryRegionLifecycleErrorKind::MissingBackingFile
    );

    let _ = fs::remove_file(format!("{}.meta", transport.backing_path));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn prepare_payload_can_carry_transport_metadata() {
    let payload = PluginMessagePayload::PrepareInstanceRequest {
        sandbox_id: "sandbox-a".into(),
        instance_id: "instance-a".into(),
        processing_epoch: 2,
        shared_memory_lease_id: "lease-a".into(),
        shared_memory_transport: SharedMemoryTransportPayload {
            region_id: "region-a".into(),
            transport_kind: SharedMemoryTransportKind::MappedFile,
            backing_path: "/tmp/region-a.signal-shm".into(),
            total_bytes: 1024,
        },
        sample_rate_hz: 48_000,
        max_block_frames: 512,
        io_layout: PluginIoLayoutPayload {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        shared_memory: SharedMemoryLayoutPayload {
            audio_input: SharedMemoryRegionPayload {
                offset_bytes: 0,
                size_bytes: 256,
            },
            audio_output: SharedMemoryRegionPayload {
                offset_bytes: 256,
                size_bytes: 256,
            },
            event_input: SharedMemoryRegionPayload {
                offset_bytes: 512,
                size_bytes: 64,
            },
            event_output: SharedMemoryRegionPayload {
                offset_bytes: 576,
                size_bytes: 64,
            },
            render_context: SharedMemoryRegionPayload {
                offset_bytes: 640,
                size_bytes: 256,
            },
            completion: SharedMemoryRegionPayload {
                offset_bytes: 896,
                size_bytes: 64,
            },
        },
    };

    match payload {
        PluginMessagePayload::PrepareInstanceRequest {
            shared_memory_transport,
            ..
        } => {
            assert_eq!(shared_memory_transport.region_id, "region-a");
            assert_eq!(shared_memory_transport.total_bytes, 1024);
        }
        other => panic!("expected prepare payload, got {other:?}"),
    }
}
