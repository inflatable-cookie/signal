use super::super::*;

#[test]
fn automation_projection_requires_explicit_targets_and_positive_linear_resolution() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 64));
    handshake_and_configure(&mut runtime);

    let error = runtime
        .apply_automation_projection(RuntimeAutomationProjection {
            lane_count: 1,
            point_count: 1,
            lanes: vec![RuntimeAutomationLaneProjection {
                automation_lane_id: "lane:invalid".into(),
                target: RuntimeAutomationTargetProjection {
                    node_id: String::new(),
                    parameter_id: "gain".into(),
                },
                base_normalized_value: 0.0,
                interpolation: RuntimeAutomationInterpolation::Linear,
                resolution: RuntimeAutomationResolution {
                    ramp_step_samples: 0,
                    max_sub_blocks: 0,
                },
                point_count: 1,
                points: vec![RuntimeAutomationPointProjection {
                    time_samples: 0,
                    normalized_value: 0.0,
                }],
            }],
        })
        .expect_err("invalid automation projection should be rejected");

    assert_eq!(error.kind, RuntimeErrorKind::InvalidRequest);
}

#[test]
fn tempo_map_projection_requires_bounded_non_overlapping_segments() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 64));
    handshake_and_configure(&mut runtime);

    let error = runtime
        .apply_tempo_map_projection(RuntimeTempoMapProjection {
            segment_count: 2,
            segments: vec![
                crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:intro".into(),
                    start_samples: 0,
                    end_samples: None,
                    start_tempo_bpm: 120.0,
                    end_tempo_bpm: None,
                    interpolation: RuntimeTempoMapInterpolation::Hold,
                },
                crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:lift".into(),
                    start_samples: 4_800,
                    end_samples: Some(9_600),
                    start_tempo_bpm: 132.0,
                    end_tempo_bpm: None,
                    interpolation: RuntimeTempoMapInterpolation::Hold,
                },
            ],
        })
        .expect_err("invalid tempo map projection should be rejected");

    assert_eq!(error.kind, RuntimeErrorKind::InvalidRequest);
    assert!(error.message.contains("open-ended tempo map segments"));
}
