use super::super::*;

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
        let dispatch =
            protocol.block_dispatch(1, block_sequence, 512, protocol.default_render_context(512));
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
        assert_eq!(outcome.output.audio.first_sample(), Some(block_sequence as f32));
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
