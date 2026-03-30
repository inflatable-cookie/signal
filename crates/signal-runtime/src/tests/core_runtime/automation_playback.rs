use super::super::*;

#[test]
fn runtime_linear_automation_projection_drives_multi_block_gain_playback() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 8));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 8))
        .unwrap();
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:automation-linear".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "gain".into(),
                execution_class: GraphNodeExecutionClass::PureTransform,
                latency_samples: 0,
                stages: vec![GraphStageSpec::Gain { linear: 0.0 }],
            }],
        })
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 0,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    runtime
        .apply_automation_projection(RuntimeAutomationProjection {
            lane_count: 1,
            point_count: 3,
            lanes: vec![RuntimeAutomationLaneProjection {
                automation_lane_id: "lane:gain:linear".into(),
                target: RuntimeAutomationTargetProjection {
                    node_id: "gain".into(),
                    parameter_id: "gain".into(),
                },
                base_normalized_value: 0.0,
                interpolation: RuntimeAutomationInterpolation::Linear,
                resolution: RuntimeAutomationResolution {
                    ramp_step_samples: 2,
                    max_sub_blocks: 8,
                },
                point_count: 3,
                points: vec![
                    RuntimeAutomationPointProjection {
                        time_samples: 0,
                        normalized_value: 0.0,
                    },
                    RuntimeAutomationPointProjection {
                        time_samples: 8,
                        normalized_value: 1.0,
                    },
                    RuntimeAutomationPointProjection {
                        time_samples: 16,
                        normalized_value: 0.0,
                    },
                ],
            }],
        })
        .unwrap();

    let first = runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![1.0; 8]),
        )
        .expect("first automation block should process");
    let second = runtime
        .process_engine_block(
            2,
            2,
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![1.0; 8]),
        )
        .expect("second automation block should process");

    assert_eq!(
        first.output.samples(),
        &[0.0, 0.0, 0.0, 0.0, 0.25, 0.25, 0.25, 0.25, 0.5, 0.5, 0.5, 0.5, 0.75, 0.75, 0.75, 0.75,]
    );
    assert_eq!(
        second.output.samples(),
        &[1.0, 1.0, 1.0, 1.0, 0.75, 0.75, 0.75, 0.75, 0.5, 0.5, 0.5, 0.5, 0.25, 0.25, 0.25, 0.25,]
    );
    assert_eq!(first.snapshot.parameter_event_count, 4);
    assert_eq!(first.snapshot.parameter_sub_block_count, 4);
    assert_eq!(second.snapshot.parameter_event_count, 4);
    assert_eq!(second.snapshot.parameter_sub_block_count, 4);

    let automation = runtime.get_automation_snapshot();
    assert_eq!(automation.lane_count, 1);
    assert_eq!(automation.point_count, 3);
    assert_eq!(automation.projected_segment_count, 2);
    assert_eq!(automation.mapped_lane_count, 1);
    assert_eq!(automation.unmapped_lane_count, 0);
    assert_eq!(automation.hold_lane_count, 0);
    assert_eq!(automation.linear_lane_count, 1);
    assert_eq!(automation.last_batch_event_count, 4);
    assert_eq!(automation.last_batch_sub_block_count, 4);
    assert_eq!(automation.last_batch_strategy_max_sub_blocks, 8);
    assert_eq!(automation.last_batch_min_ramp_step_samples, Some(2));
    assert_eq!(automation.last_batch_max_sample_offset, Some(6));
    assert_eq!(automation.last_block_sequence, Some(2));
    assert_eq!(automation.last_timeline_position_samples, Some(8));
    assert_eq!(automation.transport_playing, Some(true));

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert!(observation
        .render_compact()
        .contains("automation_projection=1/3/2"));
    assert!(observation
        .render_compact()
        .contains("automation_shapes=0/1"));
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert!(supervisor
        .render_multiline()
        .contains("automation_linear_lanes=1"));
    assert!(supervisor
        .render_multiline()
        .contains("automation_last_batch_min_ramp_step_samples=Some(2)"));
    assert!(supervisor
        .render_json()
        .contains("\"automation\":{\"lane_count\":1"));
    assert!(supervisor
        .render_json()
        .contains("\"last_batch_min_ramp_step_samples\":2"));
}

#[test]
fn runtime_hold_automation_projection_drives_plugin_backed_threshold_fixture() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 4));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 4))
        .unwrap();
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:automation-plugin".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "plugin".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 0,
                stages: vec![GraphStageSpec::HardClip { threshold: 1.0 }],
            }],
        })
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 0,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    runtime
        .apply_automation_projection(RuntimeAutomationProjection {
            lane_count: 1,
            point_count: 1,
            lanes: vec![RuntimeAutomationLaneProjection {
                automation_lane_id: "lane:plugin:threshold".into(),
                target: RuntimeAutomationTargetProjection {
                    node_id: "plugin".into(),
                    parameter_id: "threshold".into(),
                },
                base_normalized_value: 1.0,
                interpolation: RuntimeAutomationInterpolation::Hold,
                resolution: RuntimeAutomationResolution::default(),
                point_count: 1,
                points: vec![RuntimeAutomationPointProjection {
                    time_samples: 2,
                    normalized_value: 0.5,
                }],
            }],
        })
        .unwrap();

    let result = runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.7, 0.7, 0.7, 0.7],
            ),
        )
        .expect("plugin-backed automation block should process");

    assert_eq!(
        result.output.samples(),
        &[0.7, 0.7, 0.7, 0.7, 0.5, 0.5, 0.5, 0.5]
    );
    assert_eq!(result.snapshot.plugin_backed_node_count, 1);
    assert_eq!(result.snapshot.parameter_event_count, 2);
    assert_eq!(result.snapshot.parameter_sub_block_count, 2);

    let automation = runtime.get_automation_snapshot();
    assert_eq!(automation.hold_lane_count, 1);
    assert_eq!(automation.linear_lane_count, 0);
    assert_eq!(automation.mapped_lane_count, 1);
    assert_eq!(automation.projected_segment_count, 0);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert!(observation
        .render_compact()
        .contains("automation_shapes=1/0"));
    assert!(
        RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default())
            .render_json()
            .contains("\"linear_lane_count\":0")
    );
}
