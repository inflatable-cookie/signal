use super::super::*;

#[test]
fn local_host_builds_plugin_block_request_from_runtime_transport_and_parameter_truth() {
    let (mut host, protocol, _lifecycle, run) = prepare_local_host_with_lifecycle();
    let frame_count = host.runtime.config().graph.block_size as u32;
    let plugin_dispatch_state = host
        .runtime
        .prepare_plugin_dispatch_state_for_block(run.processing_epoch, 7)
        .expect("prepare plugin dispatch state");
    let (dispatch, payload) = host
        .build_plugin_block_request(
            &protocol,
            run.processing_epoch,
            7,
            frame_count,
            &plugin_dispatch_state,
        )
        .expect("build plugin block request");

    assert_eq!(dispatch.render_context.sample_rate_hz, 48_000);
    assert_eq!(dispatch.render_context.tempo_bpm, 126.0);
    assert_eq!(dispatch.render_context.timeline_position_samples, 7 * 512);
    assert!(dispatch.render_context.playing);
    assert_eq!(
        dispatch.render_context.loop_range,
        Some(LoopRange {
            start_samples: 0,
            end_samples: 16 * 512,
        })
    );
    let automation_value = payload
        .events
        .events
        .iter()
        .find_map(|event| match event {
            PluginEvent::ParameterValue(event)
                if event.parameter_id == protocol.automation_parameter_id() =>
            {
                Some(event.normalized_value)
            }
            _ => None,
        })
        .expect("automation value event");
    assert!((automation_value - 1.0).abs() < 1.0e-6);
}

#[test]
fn local_host_routes_sandbox_plugin_audio_through_bound_engine_node() {
    let (mut host, protocol, mut lifecycle, mut run) = prepare_local_host_with_lifecycle();

    let outcome = host
        .execute_block(&protocol, &mut run, 1, &mut lifecycle, false)
        .expect("execute realtime block");
    let snapshot = host.runtime.get_engine_block_snapshot();

    assert_eq!(outcome.output.audio.first_sample(), Some(1.0));
    assert_eq!(run.last_engine_graph_id.as_deref(), Some(LOCAL_DEMO_GRAPH_ID));
    assert_eq!(snapshot.graph_id.as_deref(), Some(LOCAL_DEMO_GRAPH_ID));
    assert_eq!(snapshot.output_tail_samples, LOCAL_DEMO_PLUGIN_TAIL_SAMPLES);
    assert_eq!(snapshot.last_first_output_sample, Some(0.8));
    assert!(run.last_engine_output_peak.unwrap_or_default() >= 0.79);
}

#[test]
fn local_host_timeout_block_bypasses_plugin_node_without_detaching_graph_binding() {
    let (mut host, protocol, mut lifecycle, mut run) = prepare_local_host_with_lifecycle();

    let outcome = host
        .execute_block(&protocol, &mut run, 1, &mut lifecycle, true)
        .expect("execute timeout block");
    let snapshot = host.runtime.get_engine_block_snapshot();

    assert_eq!(outcome.result.slot.state, CompletionState::TimedOut);
    assert_eq!(run.last_completion_state, CompletionState::TimedOut);
    assert_eq!(run.last_engine_graph_id.as_deref(), Some(LOCAL_DEMO_GRAPH_ID));
    assert!(snapshot.planned_nodes.iter().any(|node| {
        node.node_id == "plugin-insert"
            && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")
    }));
    assert_eq!(
        run.last_plugin_render_context
            .as_ref()
            .map(|context| context.tempo_bpm),
        Some(126.0)
    );
    assert_eq!(
        run.last_plugin_render_context
            .as_ref()
            .map(|context| context.timeline_position_samples),
        Some(512)
    );
    assert_eq!(run.last_plugin_automation_value, Some(1.0 / 7.0));
    assert_eq!(run.plugin_render_bypass_count, 1);
    assert!(run.last_plugin_render_bypassed);
    assert_eq!(
        run.last_plugin_render_latency_samples,
        LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES
    );
    assert_eq!(
        run.last_plugin_render_tail_samples,
        LOCAL_DEMO_PLUGIN_TAIL_SAMPLES
    );
    assert!(run.last_engine_output_peak.unwrap_or_default() > 0.05);
    assert!(run.last_engine_output_peak.unwrap_or_default() < 0.1);
}
