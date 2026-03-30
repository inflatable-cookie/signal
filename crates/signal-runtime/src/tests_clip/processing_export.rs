use super::*;

#[test]
fn runtime_clip_processing_exports_treatment_surface_with_warp_and_automation() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("clip-processing-export");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:clip-processing-export".to_string(),
            content_hash: "clip-processing-export".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "clip-processing-export.wav".to_string(),
            byte_size: fs::metadata(&imported_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 8,
        }])
        .unwrap();
    runtime
        .apply_tempo_map_projection(RuntimeTempoMapProjection {
            segment_count: 2,
            segments: vec![
                crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:intro".to_string(),
                    start_samples: 0,
                    end_samples: Some(48_000),
                    start_tempo_bpm: 120.0,
                    end_tempo_bpm: None,
                    interpolation: RuntimeTempoMapInterpolation::Hold,
                },
                crate::interfaces::RuntimeTempoMapSegmentProjection {
                    segment_id: "tempo:lift".to_string(),
                    start_samples: 48_000,
                    end_samples: Some(96_000),
                    start_tempo_bpm: 120.0,
                    end_tempo_bpm: Some(180.0),
                    interpolation: RuntimeTempoMapInterpolation::Linear,
                },
            ],
        })
        .unwrap();
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:processing-export".to_string(),
            media_asset_id: Some("asset:sha256:clip-processing-export".to_string()),
            mode: RuntimeWarpMode::Repitch,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:processing-export".to_string(),
            media_asset_id: Some("asset:sha256:clip-processing-export".to_string()),
            warp_mode: RuntimeWarpMode::Repitch,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope {
                duration_samples: 1_024,
                shape: RuntimeClipFadeShape::SmoothStep,
            },
            fade_out: RuntimeClipFadeEnvelope {
                duration_samples: 2_048,
                shape: RuntimeClipFadeShape::EqualPower,
            },
            clip_gain: RuntimeClipGainEnvelope {
                start_linear: 1.0,
                end_linear: 0.5,
                shape: RuntimeClipGainShape::Linear,
            },
        }])
        .unwrap();
    runtime
        .apply_automation_projection(RuntimeAutomationProjection {
            lane_count: 1,
            point_count: 2,
            lanes: vec![RuntimeAutomationLaneProjection {
                automation_lane_id: "lane:clip:gain".into(),
                target: RuntimeAutomationTargetProjection {
                    node_id: "node:clip:gain".into(),
                    parameter_id: "gain".into(),
                },
                base_normalized_value: 1.0,
                interpolation: RuntimeAutomationInterpolation::Linear,
                resolution: RuntimeAutomationResolution {
                    ramp_step_samples: 4,
                    max_sub_blocks: 8,
                },
                point_count: 2,
                points: vec![
                    RuntimeAutomationPointProjection {
                        time_samples: 0,
                        normalized_value: 1.0,
                    },
                    RuntimeAutomationPointProjection {
                        time_samples: 48_000,
                        normalized_value: 0.5,
                    },
                ],
            }],
        })
        .unwrap();
    runtime.record_automation_summary(
        1,
        "lease:clip-processing-export",
        ParameterAutomationSummary {
            parameter_id: 4096,
            value_events: 2,
            modulation_events: 0,
            gesture_begin_events: 1,
            gesture_end_events: 1,
            first_value: Some(1.0),
            last_value: Some(0.5),
            last_modulation: None,
        },
    );
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 72_000,
            tempo_bpm: 90.0,
            loop_state: None,
        })
        .unwrap();

    let clip_processing = runtime.get_clip_processing_pipeline_snapshot();
    assert_eq!(clip_processing.clip_count, 1);
    assert_eq!(clip_processing.ready_clip_count, 1);
    assert_eq!(clip_processing.faded_clip_count, 1);
    assert_eq!(clip_processing.gain_shaped_clip_count, 1);
    assert_eq!(clip_processing.warped_clip_count, 1);
    assert_eq!(clip_processing.treatment_stage_count, 4);
    assert_eq!(
        clip_processing.clips[0].project_tempo_source,
        Some(RuntimeTempoSource::TempoMapSegment)
    );
    assert_eq!(
        clip_processing.clips[0].project_tempo_segment_id.as_deref(),
        Some("tempo:lift")
    );
    assert_eq!(
        clip_processing.clips[0].treatment_stages,
        vec![
            RuntimeClipProcessingStage::Warp,
            RuntimeClipProcessingStage::FadeIn,
            RuntimeClipProcessingStage::GainShape,
            RuntimeClipProcessingStage::FadeOut,
        ]
    );
    assert!(
        (clip_processing.clips[0]
            .realized_warp_ratio
            .unwrap_or_default()
            - 1.25)
            .abs()
            < 0.000_1
    );

    let report = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    let compact = report.render_compact();
    assert!(compact.contains("clip_processing_clips=1/1/0/0"));
    assert!(compact.contains("clip_processing_shapes=1/1/1"));
    assert!(compact.contains("clip_processing_treatment_stages=4"));
    assert!(compact.contains("automation_param=4096"));
    assert!(compact.contains("tempo_map_source=TempoMapSegment"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("clip_processing_clip_count=1"));
    assert!(multiline.contains(
            "clip_processing_clip_0=clip:processing-export/readiness=Ready/warp=Repitch/Some(1.25)/Some(TempoMapSegment)"
        ));
    assert!(multiline.contains("stages=[Warp, FadeIn, GainShape, FadeOut]"));
    let json = supervisor.render_json();
    assert!(json.contains("\"clip_processing_pipeline_snapshot\":{\"clip_count\":1"));
    assert!(json.contains("\"treatment_stages\":[\"Warp\",\"FadeIn\",\"GainShape\",\"FadeOut\"]"));

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
