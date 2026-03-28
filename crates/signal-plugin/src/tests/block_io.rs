use super::*;

#[test]
fn block_dispatch_round_trips_through_shared_memory_regions() {
    let dispatch = BlockDispatch::new(
        PluginInstanceId("instance-dispatch".into()),
        5,
        9,
        256,
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        test_render_context(),
        512,
    );
    let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];

    dispatch
        .write_to_shared_memory(&mut bytes)
        .expect("write dispatch");
    let decoded = BlockDispatch::read_from_shared_memory(
        PluginInstanceId("instance-dispatch".into()),
        dispatch.io_layout,
        dispatch.layout,
        &bytes,
    )
    .expect("decode dispatch");

    assert_eq!(decoded.header, dispatch.header);
    assert_eq!(decoded.render_context, dispatch.render_context);
}

#[test]
fn block_process_result_round_trips_through_completion_region() {
    let dispatch = BlockDispatch::new(
        PluginInstanceId("instance-result".into()),
        3,
        4,
        128,
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        test_render_context(),
        256,
    );
    let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];
    let result = BlockProcessResult {
        slot: CompletionSlot {
            state: CompletionState::Completed,
            processing_epoch: 3,
            block_sequence: 4,
        },
        generated_event_bytes: 64,
        fallback_applied: false,
    };

    result
        .write_to_shared_memory(dispatch.layout, &mut bytes)
        .expect("write result");
    let decoded = BlockProcessResult::read_from_shared_memory(dispatch.layout, &bytes)
        .expect("decode result");

    assert_eq!(decoded, result);
}

#[test]
fn block_payload_round_trips_through_audio_and_event_regions() {
    let dispatch = BlockDispatch::new(
        PluginInstanceId("instance-payload".into()),
        11,
        6,
        128,
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        test_render_context(),
        256,
    );
    let payload = test_payload(&dispatch);
    let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];

    dispatch
        .write_input_payload(&mut bytes, &payload)
        .expect("write input payload");
    let decoded_input = dispatch
        .read_input_payload(&bytes)
        .expect("decode input payload");
    assert_eq!(decoded_input, payload);

    dispatch
        .write_output_payload(&mut bytes, &payload)
        .expect("write output payload");
    let decoded_output = dispatch
        .read_output_payload(&bytes)
        .expect("decode output payload");
    assert_eq!(decoded_output, payload);
}
