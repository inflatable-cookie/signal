use super::super::*;

#[test]
fn clap_lifecycle_sequence_builds_prepare_and_activate_requests() {
    let root = test_broker_root("sequence");
    let broker = SharedMemoryBroker::new(&root);
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-a",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        2048,
    );

    let messages = protocol
        .lifecycle_sequence(&broker, "sandbox-a", 48_000, 512, 1)
        .expect("build lifecycle sequence");
    assert_eq!(messages.len(), 5);
    assert_eq!(
        messages[0].message.name,
        PluginMessageName::SandboxHandshake.as_str()
    );

    match &messages[3].payload {
        PluginMessagePayload::PrepareInstanceRequest {
            processing_epoch,
            shared_memory_lease_id,
            shared_memory_transport,
            sample_rate_hz,
            max_block_frames,
            shared_memory,
            ..
        } => {
            assert_eq!(*processing_epoch, 1);
            assert_eq!(*sample_rate_hz, 48_000);
            assert_eq!(*max_block_frames, 512);
            assert!(shared_memory_lease_id.contains("sandbox-a"));
            assert_eq!(
                shared_memory_transport.transport_kind,
                SharedMemoryTransportKind::MappedFile
            );
            assert!(std::path::Path::new(&shared_memory_transport.backing_path).exists());
            assert!(shared_memory.total_bytes() > 0);
        }
        other => panic!("expected prepare request, got {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn clap_lifecycle_harness_accepts_full_control_sequence() {
    let root = test_broker_root("accept");
    let broker = SharedMemoryBroker::new(&root);
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-b",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        1024,
    );
    let mut harness = ClapSandboxLifecycleHarness::default();
    let messages = protocol
        .lifecycle_sequence(&broker, "sandbox-b", 48_000, 512, 1)
        .expect("build lifecycle sequence");

    let responses = messages
        .into_iter()
        .map(|message| harness.handle(message).expect("accepted request"))
        .collect::<Vec<_>>();

    assert_eq!(responses.len(), 5);
    assert_eq!(
        responses.last().expect("last response").message.name,
        PluginMessageName::SandboxActivateInstance.as_str()
    );
    match &responses.last().expect("last response").payload {
        PluginMessagePayload::ActivateInstanceResponse { instance_state, .. } => {
            assert_eq!(instance_state.lifecycle_state, "Active");
            assert_eq!(instance_state.readiness_state, "Ready");
            assert!(instance_state.active);
            assert!(instance_state.processing.is_some());
        }
        other => panic!("expected activate response, got {other:?}"),
    }
    assert_eq!(
        harness
            .lease()
            .expect("prepared lease")
            .invalidated_epochs()
            .len(),
        0
    );
    assert!(harness
        .lease()
        .and_then(|lease| lease.transport())
        .is_some());
    harness
        .teardown_active_transport()
        .expect("teardown transport");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn clap_lifecycle_harness_can_invalidate_active_epoch() {
    let root = test_broker_root("invalidate");
    let broker = SharedMemoryBroker::new(&root);
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-invalidate",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        1024,
    );
    let mut harness = ClapSandboxLifecycleHarness::default();
    let messages = protocol
        .lifecycle_sequence(&broker, "sandbox-invalidate", 48_000, 512, 3)
        .expect("build lifecycle sequence");

    for message in messages {
        harness.handle(message).expect("accepted request");
    }

    let (completion_invalidated, lease_invalidated) = harness.invalidate_active_epoch(3);
    assert!(completion_invalidated);
    assert!(lease_invalidated);
    assert_eq!(
        harness
            .lease()
            .expect("prepared lease")
            .invalidated_epochs(),
        &[3]
    );

    harness
        .teardown_active_transport()
        .expect("teardown transport");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn clap_harness_accepts_deactivate_reset_and_destroy_sequence() {
    let root = test_broker_root("teardown");
    let broker = SharedMemoryBroker::new(&root);
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-teardown",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        1024,
    );
    let mut harness = ClapSandboxLifecycleHarness::default();
    let messages = protocol
        .lifecycle_sequence(&broker, "sandbox-teardown", 48_000, 512, 1)
        .expect("build lifecycle sequence");
    for message in messages {
        harness.handle(message).expect("accepted request");
    }

    let teardown_responses = protocol
        .teardown_sequence("sandbox-teardown", 2)
        .into_iter()
        .map(|message| harness.handle(message).expect("accepted teardown request"))
        .collect::<Vec<_>>();

    assert_eq!(
        teardown_responses[0].message.name,
        PluginMessageName::SandboxDeactivateInstance.as_str()
    );
    assert_eq!(
        teardown_responses[1].message.name,
        PluginMessageName::SandboxResetInstance.as_str()
    );
    assert_eq!(
        teardown_responses[2].message.name,
        PluginMessageName::SandboxDestroyInstance.as_str()
    );
    assert!(harness.lease().is_none());
    let _ = fs::remove_dir_all(root);
}
