use super::*;

#[test]
fn runtime_metering_snapshot_reports_loudness_for_non_silent_output() {
    let mut metering = RuntimeMeteringStateModel::default();
    let output = AudioBuffer::from_interleaved(
        SampleRate(48_000),
        ChannelLayout::Stereo,
        vec![0.5, -0.5, 0.25, -0.25, 0.75, -0.75, 0.125, -0.125],
    );

    metering.capture(
        48_000,
        &output,
        vec![RuntimeMeterSourceSnapshot {
            bus_id: "main:out".into(),
            topology_role: RuntimeMeterSourceRole::Bus,
            track_lane_id: None,
            bus_group_id: Some("mix:master".into()),
            console_group_id: None,
            send_return_id: None,
            producer_node_ids: vec!["bus-main".into()],
            peak_level: 0.75,
            rms_level: 0.4677072,
            latency_samples: 0,
            tail_samples: 0,
            summary: "main output".into(),
        }],
    );

    let snapshot = metering.snapshot();
    assert_eq!(snapshot.meter_count, 1);
    assert_eq!(snapshot.main_output_peak_level, Some(0.75));
    assert_eq!(snapshot.main_output_rms_level, Some(0.4677072));
    assert!(snapshot.momentary_loudness_lufs.is_some());
    assert!(snapshot.integrated_loudness_lufs.is_some());
    assert_eq!(snapshot.clipped_sample_count, 0);
    assert!(snapshot
        .meters
        .iter()
        .any(|meter| meter.bus_id == "main:out"));
}

#[test]
fn runtime_automation_projection_drives_within_block_parameter_events() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 6));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:automation-playback".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "gain".into(),
                execution_class: GraphNodeExecutionClass::PureTransform,
                latency_samples: 0,
                stages: vec![GraphStageSpec::Gain { linear: 1.0 }],
            }],
        })
        .expect("apply automation playback graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:automation-playback".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "gain".into(),
                buffer_contract: GraphNodeBufferContractProjection {
                    input: GraphNodeBusEndpointProjection {
                        bus_id: "main:in".into(),
                        channels: ChannelLayout::Mono,
                    },
                    output: GraphNodeBusEndpointProjection {
                        bus_id: "main:out".into(),
                        channels: ChannelLayout::Mono,
                    },
                    ..GraphNodeBufferContractProjection::default()
                },
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::Utility),
                    track_lane_id: None,
                    bus_group_id: None,
                    console_group_id: None,
                    send_return_id: None,
                },
            }],
        })
        .expect("apply automation playback contract");
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:automation-playback".into(),
            stream_count: 1,
        })
        .expect("apply automation playback schedule");
    let receipt = runtime
        .apply_automation_projection(RuntimeAutomationProjection {
            lane_count: 1,
            point_count: 2,
            lanes: vec![RuntimeAutomationLaneProjection {
                automation_lane_id: "automation-lane:gain".into(),
                target: RuntimeAutomationTargetProjection {
                    node_id: "gain".into(),
                    parameter_id: "gain".into(),
                },
                base_normalized_value: 0.0,
                interpolation: crate::interfaces::RuntimeAutomationInterpolation::Hold,
                resolution: RuntimeAutomationResolution::default(),
                point_count: 2,
                points: vec![
                    RuntimeAutomationPointProjection {
                        time_samples: 2,
                        normalized_value: 0.5,
                    },
                    RuntimeAutomationPointProjection {
                        time_samples: 4,
                        normalized_value: 1.0,
                    },
                ],
            }],
        })
        .expect("apply automation projection");
    runtime
        .apply_parameter_batch(ParameterBatch {
            epoch: receipt.accepted_epoch,
            events: Vec::new(),
        })
        .expect("apply automation epoch batch");
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 0,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .expect("apply transport");

    let input =
        AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![1.0; 6]);
    let result = runtime
        .process_engine_block(1, 1, input)
        .expect("process automated block");

    assert_eq!(
        result.snapshot.parameter_epoch,
        Some(receipt.accepted_epoch)
    );
    assert_eq!(result.snapshot.parameter_event_count, 3);
    assert_eq!(result.snapshot.parameter_sub_block_count, 3);
    assert_eq!(result.snapshot.parameter_ignored_event_count, 0);
    let expected = [0.0_f32, 0.0, 0.5, 0.5, 1.0, 1.0];
    for (actual, expected) in result.output.samples().iter().zip(expected.iter()) {
        assert!((actual - expected).abs() < 1.0e-6);
    }

    let automation = runtime.get_automation_snapshot();
    assert_eq!(automation.lane_count, 1);
    assert_eq!(automation.point_count, 2);
    assert_eq!(automation.mapped_lane_count, 1);
    assert_eq!(automation.unmapped_lane_count, 0);
    assert_eq!(automation.last_batch_epoch, Some(receipt.accepted_epoch));
    assert_eq!(automation.last_batch_event_count, 3);
    assert_eq!(automation.last_batch_sub_block_count, 3);
    assert_eq!(automation.last_batch_ignored_event_count, 0);
    assert_eq!(automation.last_batch_coalesced_event_count, 0);
    assert_eq!(automation.last_batch_max_sample_offset, Some(4));
    assert_eq!(automation.last_block_sequence, Some(1));
    assert_eq!(automation.last_timeline_position_samples, Some(0));
    assert_eq!(automation.transport_playing, Some(true));
}
