use signal_ipc::SharedMemoryBroker;
use signal_plugin::PluginIoLayout;
use signal_plugin_clap::{ClapBlockProtocol, ClapSandboxLifecycleHarness};

fn main() {
    let sandbox_id = "plugin-sandbox-shell";
    let broker = SharedMemoryBroker::default();
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:sandbox",
        "instance:sandbox:default",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        2048,
    );
    let requests = protocol
        .lifecycle_sequence(&broker, sandbox_id, 48_000, 512, 1)
        .expect("sandbox lifecycle sequence");
    let mut harness = ClapSandboxLifecycleHarness::default();
    let responses = requests
        .iter()
        .cloned()
        .map(|request| harness.handle(request))
        .collect::<Result<Vec<_>, _>>()
        .expect("sandbox lifecycle");
    let (lease_id, region_id, transport) = responses
        .iter()
        .find_map(|response| match &response.payload {
            signal_ipc::PluginMessagePayload::PrepareInstanceResponse {
                shared_memory_lease_id,
                shared_memory_transport,
                ..
            } => Some((
                shared_memory_lease_id.clone(),
                shared_memory_transport.region_id.clone(),
                shared_memory_transport.clone(),
            )),
            _ => None,
        })
        .map(|(lease_id, region_id, transport)| (Some(lease_id), Some(region_id), Some(transport)))
        .unwrap_or((None, None, None));
    let loaded_descriptor = responses
        .iter()
        .find_map(|response| match &response.payload {
            signal_ipc::PluginMessagePayload::LoadPluginTypeResponse { descriptor, .. } => {
                Some(descriptor.clone())
            }
            _ => None,
        });
    let last_instance_state = responses
        .last()
        .and_then(|response| match &response.payload {
            signal_ipc::PluginMessagePayload::ActivateInstanceResponse {
                instance_state, ..
            }
            | signal_ipc::PluginMessagePayload::ResetInstanceResponse { instance_state, .. }
            | signal_ipc::PluginMessagePayload::DeactivateInstanceResponse {
                instance_state, ..
            }
            | signal_ipc::PluginMessagePayload::CreateInstanceResponse { instance_state, .. }
            | signal_ipc::PluginMessagePayload::PrepareInstanceResponse {
                instance_state, ..
            }
            | signal_ipc::PluginMessagePayload::DestroyInstanceResponse {
                instance_state, ..
            } => Some(instance_state.clone()),
            _ => None,
        });
    let _ = harness
        .handle(protocol.heartbeat_request(sandbox_id, Some(1)))
        .expect("heartbeat");
    let mut processed_blocks = 0usize;
    let mut last_output_event_count = 0usize;
    let mut last_parameter_event_count = 0usize;
    let mut last_parameter_gesture_event_count = 0usize;
    let mut last_parameter_modulation_event_count = 0usize;
    let mut last_note_event_count = 0usize;
    let mut last_note_expression_event_count = 0usize;
    let mut last_midi_event_count = 0usize;
    let mut last_output_first_sample = None;
    let mut last_completion = None;
    if let Some(transport) = &transport {
        for block_sequence in 0..8 {
            let dispatch = protocol.block_dispatch(
                1,
                block_sequence,
                512,
                protocol.default_render_context(512),
            );
            let payload = protocol.test_input_payload(block_sequence, 512);
            protocol
                .write_block_payload(&broker, transport, &dispatch, &payload)
                .expect("write block payload");
            harness.process_pending_block().expect("process block");
            let outcome = protocol
                .read_block_outcome(&broker, transport, &dispatch)
                .expect("read block outcome");
            let event_summary = outcome.output.events.summary();
            processed_blocks += 1;
            last_output_event_count = outcome.output.events.event_count();
            last_parameter_event_count = event_summary.parameter_value_events;
            last_parameter_gesture_event_count = event_summary.parameter_gesture_events;
            last_parameter_modulation_event_count = event_summary.parameter_modulation_events;
            last_note_event_count = event_summary.note_events;
            last_note_expression_event_count = event_summary.note_expression_events;
            last_midi_event_count = event_summary.midi_events;
            last_output_first_sample = outcome.output.audio.first_sample();
            last_completion = Some(outcome.result.slot.state);
        }
    }

    println!(
        "signal-plugin-sandbox sandbox_id={:?} requests={} responses={} first_request={:?} last_response={:?} descriptor={:?}/{:?}/{:?}/{:?} lease_id={:?} region_id={:?} state={:?}/{:?} heartbeats={} processed_blocks={} output_events={} parameter_events={} parameter_gesture_events={} parameter_modulation_events={} note_events={} note_expression_events={} midi_events={} first_output_sample={:?} completion={:?}",
        sandbox_id,
        requests.len(),
        responses.len(),
        requests.first().map(|request| request.message.name.clone()),
        responses.last().map(|response| response.message.name.clone()),
        loaded_descriptor
            .as_ref()
            .map(|descriptor| descriptor.plugin_id.as_str()),
        loaded_descriptor
            .as_ref()
            .map(|descriptor| descriptor.vendor.as_str()),
        loaded_descriptor
            .as_ref()
            .map(|descriptor| descriptor.name.as_str()),
        loaded_descriptor
            .as_ref()
            .map(|descriptor| descriptor.format.as_str()),
        lease_id,
        region_id,
        last_instance_state
            .as_ref()
            .map(|state| state.lifecycle_state.as_str()),
        last_instance_state
            .as_ref()
            .map(|state| state.readiness_state.as_str()),
        harness.heartbeat_count(),
        processed_blocks,
        last_output_event_count,
        last_parameter_event_count,
        last_parameter_gesture_event_count,
        last_parameter_modulation_event_count,
        last_note_event_count,
        last_note_expression_event_count,
        last_midi_event_count,
        last_output_first_sample,
        last_completion,
    );
}
