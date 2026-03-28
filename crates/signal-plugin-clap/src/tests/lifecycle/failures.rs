use super::super::*;

#[test]
fn clap_lifecycle_harness_rejects_unknown_plugin_type_requests() {
    let root = test_broker_root("unknown-plugin");
    let broker = SharedMemoryBroker::new(&root);
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-unknown",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        1024,
    );
    let mut harness = ClapSandboxLifecycleHarness::default();
    let mut messages = protocol
        .lifecycle_sequence(&broker, "sandbox-unknown", 48_000, 512, 1)
        .expect("build lifecycle sequence");
    if let Some(load) = messages.get_mut(1) {
        load.payload = PluginMessagePayload::LoadPluginTypeRequest {
            sandbox_id: "sandbox-unknown".into(),
            plugin_type_id: "plugin:vst:missing".into(),
            descriptor: PluginDescriptorPayload {
                plugin_id: "plugin:vst:missing".into(),
                vendor: "Signal".into(),
                name: "Missing Plugin".into(),
                format: "clap".into(),
            },
        };
    }

    harness
        .handle(messages.remove(0))
        .expect("accepted handshake");

    match harness
        .handle(messages.remove(0))
        .expect_err("missing plugin failure")
        .payload
    {
        PluginMessagePayload::SandboxFailure {
            error_kind,
            detail,
            fault,
            ..
        } => {
            assert_eq!(error_kind, "unsupported");
            assert!(detail.contains("not available in the local catalog"));
            assert_eq!(fault.kind, "unsupportedCapability");
            assert_eq!(fault.severity, "warning");
        }
        other => panic!("expected sandbox failure, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn clap_lifecycle_harness_rejects_prepare_requests_above_contract_limit() {
    let root = test_broker_root("prepare-limit");
    let broker = SharedMemoryBroker::new(&root);
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-prepare-limit",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        1024,
    );
    let mut harness = ClapSandboxLifecycleHarness::default();
    let mut messages = protocol
        .lifecycle_sequence(&broker, "sandbox-prepare-limit", 48_000, 512, 1)
        .expect("build lifecycle sequence");
    if let Some(prepare) = messages.get_mut(3) {
        let original_payload = prepare.payload.clone();
        match original_payload {
            PluginMessagePayload::PrepareInstanceRequest {
                sandbox_id,
                instance_id,
                processing_epoch,
                shared_memory_lease_id,
                shared_memory_transport,
                sample_rate_hz,
                io_layout,
                shared_memory,
                ..
            } => {
                prepare.payload = PluginMessagePayload::PrepareInstanceRequest {
                    sandbox_id,
                    instance_id,
                    processing_epoch,
                    shared_memory_lease_id,
                    shared_memory_transport,
                    sample_rate_hz,
                    max_block_frames: 8_192,
                    io_layout,
                    shared_memory,
                };
            }
            other => panic!("expected prepare request, got {other:?}"),
        }
    }
    let mut messages = messages.into_iter();
    harness
        .handle(messages.next().expect("handshake request"))
        .expect("accepted handshake");
    harness
        .handle(messages.next().expect("load request"))
        .expect("accepted load");
    harness
        .handle(messages.next().expect("create request"))
        .expect("accepted create");

    match harness.handle(messages.next().expect("prepare request")) {
        Ok(_) => panic!("expected prepare failure"),
        Err(failure) => match failure.payload {
            PluginMessagePayload::SandboxFailure {
                error_kind,
                detail,
                processing_epoch,
                shared_memory_lease_id,
                fault,
                ..
            } => {
                assert_eq!(error_kind, "resourceUnavailable");
                assert!(detail.contains("exceeds discovered CLAP processing contract"));
                assert_eq!(processing_epoch, Some(1));
                assert!(shared_memory_lease_id.is_some());
                assert_eq!(fault.kind, "resourceUnavailable");
                assert_eq!(fault.severity, "recoverable");
            }
            other => panic!("expected sandbox failure, got {other:?}"),
        },
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn clap_lifecycle_harness_emits_failure_and_invalidates_epoch() {
    let root = test_broker_root("failure");
    let broker = SharedMemoryBroker::new(&root);
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-fault",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        1024,
    );
    let mut harness = ClapSandboxLifecycleHarness::default();
    let mut messages = protocol
        .lifecycle_sequence(&broker, "sandbox-fault", 48_000, 512, 1)
        .expect("build lifecycle sequence");
    if let Some(activate) = messages.last_mut() {
        activate.payload = PluginMessagePayload::ActivateInstanceRequest {
            sandbox_id: "sandbox-fault".into(),
            instance_id: "instance-fault".into(),
            processing_epoch: 9,
        };
    }

    let mut last_failure = None;
    for message in messages {
        match harness.handle(message) {
            Ok(_) => {}
            Err(failure) => {
                last_failure = Some(failure);
                break;
            }
        }
    }

    match last_failure.expect("failure envelope").payload {
        PluginMessagePayload::SandboxFailure {
            error_kind,
            fault,
            processing_epoch,
            ..
        } => {
            assert_eq!(error_kind, "protocolViolation");
            assert_eq!(fault.kind, "protocolViolation");
            assert_eq!(fault.severity, "critical");
            assert_eq!(processing_epoch, Some(9));
        }
        other => panic!("expected sandbox failure envelope, got {other:?}"),
    }
    assert!(!harness.lease().expect("lease").is_epoch_valid(9));
    harness
        .teardown_active_transport()
        .expect("teardown transport");
    let _ = fs::remove_dir_all(root);
}
