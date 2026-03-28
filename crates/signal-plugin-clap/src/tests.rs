// Tests for signal-plugin-clap
#[allow(clippy::module_inception)]
mod tests {
    use crate::{
        classify_sandbox_failure, sandbox_failure_event, ClapBlockProtocol, ClapEvent,
        ClapHostExtension, ClapNoteExpressionEvent, ClapNoteExpressionKind, ClapParamGestureEvent,
        ClapParamGesturePhase, ClapPluginHostAdapter, ClapSandboxFailureInput,
        ClapSandboxFailureStage, ClapSandboxLifecycleHarness,
    };
    use signal_ipc::{
        PluginDescriptorPayload, PluginMessageName, PluginMessagePayload, SharedMemoryBroker,
        SharedMemoryTransportKind,
    };
    use signal_plugin::{CompletionState, EventPacket, PluginFormat, PluginIoLayout};
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
            "signal-plugin-clap-tests-{}-{name}-{timestamp}",
            process::id()
        ))
    }

    #[test]
    fn clap_adapter_reports_supported_format_and_extensions() {
        let adapter = ClapPluginHostAdapter::default();
        assert!(adapter.supports_format(PluginFormat::Clap));
        assert!(adapter
            .minimum_extension_set()
            .contains(&ClapHostExtension::Params));
        assert_eq!(adapter.minimum_extension_set()[0].as_str(), "audio-ports");
    }

    #[test]
    fn clap_adapter_discovers_concrete_plugin_type_metadata() {
        let adapter = ClapPluginHostAdapter::default();
        let discovered = adapter
            .discover_plugin_type("plugin:clap:sandbox")
            .expect("discovered sandbox plugin");

        assert_eq!(discovered.plugin_type_id.0, "plugin:clap:sandbox");
        assert_eq!(discovered.descriptor.plugin_id, "plugin:clap:sandbox");
        assert_eq!(discovered.descriptor.name, "Signal Sandbox CLAP Plugin");
        assert_eq!(discovered.descriptor.format, PluginFormat::Clap);
        assert_eq!(discovered.default_io_layout.audio_inputs, 2);
        assert_eq!(discovered.default_io_layout.audio_outputs, 2);
        assert_eq!(discovered.default_io_layout.midi_inputs, 1);
        assert_eq!(discovered.default_io_layout.midi_outputs, 1);
    }

    #[test]
    fn clap_protocol_descriptor_projects_plugin_neutral_contract_surface() {
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

        let descriptor = protocol.descriptor();
        assert_eq!(descriptor.plugin_id, "plugin:clap:test");
        assert_eq!(descriptor.version.as_deref(), Some("0.1.0"));
        assert_eq!(descriptor.audio_buses.len(), 2);
        assert_eq!(descriptor.parameters.len(), 2);
        assert!(descriptor.processing_contract.sample_accurate_automation);
        assert!(descriptor.processing_contract.accepts_midi);
        assert!(descriptor.state_contract.supports_snapshot);
        assert!(descriptor.lifecycle_contract.supports_reset_while_active);
    }

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

    #[test]
    fn clap_shared_memory_header_scales_with_channel_count() {
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-c",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            1024,
        );
        let header = protocol.block_header(1, 2, 512);
        assert_eq!(header.block.channel_count, 2);
        assert!(header.layout.audio_input.size_bytes > 0);
    }

    #[test]
    fn clap_event_translation_upgrades_note_and_modulation_semantics() {
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-translate",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            1024,
        );
        let payload = protocol.test_input_payload(3, 512);

        let clap_events = protocol.translate_input_events(&payload.events);
        assert_eq!(clap_events.events.len(), 11);
        assert!(matches!(clap_events.events[0], ClapEvent::ParamGesture(_)));
        assert!(matches!(clap_events.events[1], ClapEvent::ParamGesture(_)));
        assert!(matches!(clap_events.events[2], ClapEvent::ParamValue(_)));
        assert!(matches!(clap_events.events[3], ClapEvent::ParamValue(_)));
        assert!(matches!(
            clap_events.events[4],
            ClapEvent::ParamModulation(_)
        ));
        assert!(matches!(
            clap_events.events[5],
            ClapEvent::ParamModulation(_)
        ));
        assert!(matches!(clap_events.events[6], ClapEvent::Note(_)));
        assert!(matches!(
            clap_events.events[7],
            ClapEvent::NoteExpression(ClapNoteExpressionEvent {
                expression: ClapNoteExpressionKind::Timbre,
                ..
            })
        ));
        assert!(matches!(
            clap_events.events[8],
            ClapEvent::NoteExpression(ClapNoteExpressionEvent {
                expression: ClapNoteExpressionKind::Tuning,
                ..
            })
        ));
        assert!(matches!(
            clap_events.events[9],
            ClapEvent::NoteExpression(ClapNoteExpressionEvent {
                expression: ClapNoteExpressionKind::Pressure,
                ..
            })
        ));
        assert!(matches!(clap_events.events[10], ClapEvent::Midi(_)));
        assert!(matches!(
            clap_events.events[1],
            ClapEvent::ParamGesture(ClapParamGestureEvent {
                phase: ClapParamGesturePhase::End,
                ..
            })
        ));

        let round_tripped = protocol.translate_output_events(&clap_events);
        let summary = round_tripped.summary();
        assert_eq!(summary.parameter_value_events, 2);
        assert_eq!(summary.parameter_gesture_events, 2);
        assert_eq!(summary.parameter_modulation_events, 2);
        assert_eq!(summary.note_events, 1);
        assert_eq!(summary.note_expression_events, 3);
        assert_eq!(summary.midi_events, 1);
        let automation =
            round_tripped.parameter_automation_summary(protocol.automation_parameter_id());
        assert_eq!(automation.value_events, 1);
        assert_eq!(automation.modulation_events, 1);
        assert_eq!(automation.gesture_begin_events, 0);
        assert_eq!(automation.gesture_end_events, 1);
        assert_eq!(automation.first_value, Some(0.25));
        assert_eq!(automation.last_value, Some(0.25));
        assert_eq!(automation.last_modulation, Some(-0.02));
    }

    #[test]
    fn clap_harness_processes_brokered_block_and_heartbeat() {
        let root = test_broker_root("block");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-block",
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
            .lifecycle_sequence(&broker, "sandbox-block", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        let responses = messages
            .into_iter()
            .map(|message| harness.handle(message).expect("accepted request"))
            .collect::<Vec<_>>();
        let transport = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::PrepareInstanceResponse {
                    shared_memory_transport,
                    ..
                } => Some(shared_memory_transport.clone()),
                _ => None,
            })
            .expect("prepare transport");

        let heartbeat = harness
            .handle(protocol.heartbeat_request("sandbox-block", Some(1)))
            .expect("heartbeat response");
        match heartbeat.payload {
            PluginMessagePayload::HeartbeatResponse { active, .. } => assert!(active),
            other => panic!("expected heartbeat response, got {other:?}"),
        }
        assert_eq!(harness.heartbeat_count(), 1);

        let dispatch = protocol.block_dispatch(1, 4, 512, protocol.default_render_context(512));
        let payload = protocol.test_input_payload(4, 512);
        protocol
            .write_block_payload(&broker, &transport, &dispatch, &payload)
            .expect("write block payload");
        let result = harness.process_pending_block().expect("process block");
        assert_eq!(result.slot.state, CompletionState::Completed);
        let expected_output_events =
            protocol.translate_output_events(&protocol.translate_input_events(&payload.events));
        assert_eq!(
            result.generated_event_bytes,
            expected_output_events.encoded_bytes()
        );

        let stored_outcome = protocol
            .read_block_outcome(&broker, &transport, &dispatch)
            .expect("read block outcome");
        assert_eq!(stored_outcome.result.slot.state, CompletionState::Completed);
        assert_eq!(stored_outcome.input, payload);
        assert_eq!(stored_outcome.output.audio, stored_outcome.input.audio);
        assert_eq!(stored_outcome.output.events, expected_output_events);

        harness
            .teardown_active_transport()
            .expect("teardown transport");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_harness_round_trips_multi_block_payload_sequence() {
        let root = test_broker_root("multi-block");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-multi-block",
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            1024,
        );
        let mut harness = ClapSandboxLifecycleHarness::default();
        let messages = protocol
            .lifecycle_sequence(&broker, "sandbox-multi-block", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        let responses = messages
            .into_iter()
            .map(|message| harness.handle(message).expect("accepted request"))
            .collect::<Vec<_>>();
        let transport = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::PrepareInstanceResponse {
                    shared_memory_transport,
                    ..
                } => Some(shared_memory_transport.clone()),
                _ => None,
            })
            .expect("prepare transport");

        let mut aggregated_output_events = EventPacket::new(Vec::new());
        for block_sequence in 0..4 {
            let dispatch = protocol.block_dispatch(
                1,
                block_sequence,
                512,
                protocol.default_render_context(512),
            );
            let payload = protocol.test_input_payload(block_sequence, 512);
            protocol
                .write_block_payload(&broker, &transport, &dispatch, &payload)
                .expect("write block payload");
            let result = harness.process_pending_block().expect("process block");
            assert_eq!(result.slot.block_sequence, block_sequence);
            assert_eq!(result.slot.state, CompletionState::Completed);

            let outcome = protocol
                .read_block_outcome(&broker, &transport, &dispatch)
                .expect("read block outcome");
            assert_eq!(outcome.input, payload);
            let expected_output_events =
                protocol.translate_output_events(&protocol.translate_input_events(&payload.events));
            assert_eq!(outcome.output.audio, outcome.input.audio);
            assert_eq!(outcome.output.events, expected_output_events);
            assert_eq!(
                outcome.output.audio.first_sample(),
                Some(block_sequence as f32)
            );
            assert_eq!(outcome.output.events.event_count(), 11);
            aggregated_output_events
                .events
                .extend(outcome.output.events.events.iter().copied());
        }

        let automation = aggregated_output_events
            .parameter_automation_summary(protocol.automation_parameter_id());
        assert_eq!(automation.value_events, 4);
        assert_eq!(automation.modulation_events, 4);
        assert_eq!(automation.gesture_begin_events, 1);
        assert_eq!(automation.gesture_end_events, 3);
        assert_eq!(automation.first_value, Some(0.1));
        assert_eq!(automation.last_value, Some(0.25));
        assert_eq!(automation.last_modulation, Some(-0.02));

        harness
            .teardown_active_transport()
            .expect("teardown transport");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clap_harness_marks_deadline_miss_in_completion_region() {
        let root = test_broker_root("timeout");
        let broker = SharedMemoryBroker::new(&root);
        let protocol = ClapBlockProtocol::new(
            "plugin:clap:test",
            "instance-timeout",
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
            .lifecycle_sequence(&broker, "sandbox-timeout", 48_000, 512, 1)
            .expect("build lifecycle sequence");
        let responses = messages
            .into_iter()
            .map(|message| harness.handle(message).expect("accepted request"))
            .collect::<Vec<_>>();
        let transport = responses
            .iter()
            .find_map(|response| match &response.payload {
                PluginMessagePayload::PrepareInstanceResponse {
                    shared_memory_transport,
                    ..
                } => Some(shared_memory_transport.clone()),
                _ => None,
            })
            .expect("prepare transport");

        let dispatch = protocol.block_dispatch(1, 5, 512, protocol.default_render_context(512));
        protocol
            .write_block_dispatch(&broker, &transport, &dispatch)
            .expect("write block dispatch");
        let result = harness.mark_deadline_miss().expect("mark deadline miss");
        assert_eq!(result.slot.state, CompletionState::TimedOut);
        assert!(result.fallback_applied);

        let stored_result = protocol
            .read_block_result(&broker, &transport, 512)
            .expect("read block result");
        assert_eq!(stored_result.slot.state, CompletionState::TimedOut);

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

    #[test]
    fn sandbox_failure_event_exposes_timeout_kind() {
        let failure = sandbox_failure_event(ClapSandboxFailureInput {
            sandbox_id: "sandbox-a".to_string(),
            instance_id: Some("instance-a".into()),
            stage: "processBlock".to_string(),
            error_kind: "timeout".to_string(),
            detail: "sandbox exceeded block deadline".to_string(),
            processing_epoch: Some(3),
            shared_memory_lease_id: Some("lease-a".into()),
            correlation_id: None,
            instance_state: None,
        });

        match failure.payload {
            PluginMessagePayload::SandboxFailure { error_kind, .. } => {
                assert_eq!(error_kind, "timeout");
            }
            other => panic!("expected sandbox failure, got {other:?}"),
        }
    }

    #[test]
    fn classify_sandbox_failure_maps_process_attach_errors() {
        let failure = sandbox_failure_event(ClapSandboxFailureInput {
            sandbox_id: "sandbox-a".to_string(),
            instance_id: Some("instance-a".into()),
            stage: "processBlock".to_string(),
            error_kind: "resourceUnavailable".to_string(),
            detail: "failed to attach shared-memory region: stale mapping".to_string(),
            processing_epoch: Some(3),
            shared_memory_lease_id: Some("lease-a".into()),
            correlation_id: None,
            instance_state: None,
        });

        let classification = classify_sandbox_failure(&failure).expect("classification");
        assert_eq!(classification.stage, ClapSandboxFailureStage::ProcessAttach);
        assert_eq!(classification.operation, "processBlock");
        assert_eq!(classification.lease_id.as_deref(), Some("lease-a"));
    }
}
