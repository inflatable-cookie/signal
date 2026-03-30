use super::*;

#[test]
fn runtime_tempo_map_projection_drives_warp_ratio_and_export_reports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let imported_path = temp_capture_path("warp-tempo-map");
    write_test_wav(&imported_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:warp-tempo-map".to_string(),
            content_hash: "warp-tempo-map".to_string(),
            source_path: imported_path.display().to_string(),
            file_name: "warp-tempo-map.wav".to_string(),
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
            clip_id: "clip:warp-tempo-map".to_string(),
            media_asset_id: Some("asset:sha256:warp-tempo-map".to_string()),
            mode: RuntimeWarpMode::Repitch,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 72_000,
            tempo_bpm: 90.0,
            loop_state: None,
        })
        .unwrap();

    let tempo_map = runtime.get_tempo_map_snapshot();
    assert_eq!(tempo_map.segment_count, 2);
    assert_eq!(tempo_map.active_segment_id.as_deref(), Some("tempo:lift"));
    assert_eq!(tempo_map.active_segment_index, Some(1));
    assert_eq!(tempo_map.tempo_source, RuntimeTempoSource::TempoMapSegment);
    assert!((tempo_map.resolved_tempo_bpm - 150.0).abs() < 0.000_1);

    let warp = runtime.get_warp_pipeline_snapshot();
    assert_eq!(warp.clip_count, 1);
    assert_eq!(warp.ready_clip_count, 1);
    assert_eq!(warp.degraded_clip_count, 0);
    assert_eq!(
        warp.resolved_project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        warp.resolved_project_tempo_segment_id.as_deref(),
        Some("tempo:lift")
    );
    assert!((warp.resolved_project_tempo_bpm - 150.0).abs() < 0.000_1);
    assert_eq!(
        warp.clips[0].project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        warp.clips[0].project_tempo_segment_id.as_deref(),
        Some("tempo:lift")
    );
    assert!((warp.clips[0].project_tempo_bpm - 150.0).abs() < 0.000_1);
    assert!((warp.clips[0].realized_ratio - 1.25).abs() < 0.000_1);

    let report = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        report.tempo_map_snapshot.tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert_eq!(
        report.warp_pipeline_snapshot.resolved_project_tempo_source,
        RuntimeTempoSource::TempoMapSegment
    );
    assert!(report.render_compact().contains("tempo_map_segments=2"));
    assert!(report
        .render_compact()
        .contains("tempo_map_source=TempoMapSegment"));
    assert!(report.render_compact().contains("warp_clips=1/1/0/0"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("tempo_map_source=TempoMapSegment"));
    assert!(multiline.contains("warp_resolved_project_tempo_source=TempoMapSegment"));
    let json = supervisor.render_json();
    assert!(json.contains("\"tempo_map_snapshot\":{\"segment_count\":2"));
    assert!(json.contains("\"resolved_project_tempo_source\":\"TempoMapSegment\""));

    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 120_000,
            tempo_bpm: 90.0,
            loop_state: None,
        })
        .unwrap();
    let fallback_tempo_map = runtime.get_tempo_map_snapshot();
    assert_eq!(fallback_tempo_map.active_segment_id, None);
    assert_eq!(
        fallback_tempo_map.tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert!((fallback_tempo_map.resolved_tempo_bpm - 90.0).abs() < 0.000_1);
    let fallback_warp = runtime.get_warp_pipeline_snapshot();
    assert_eq!(
        fallback_warp.resolved_project_tempo_source,
        RuntimeTempoSource::TransportProjection
    );
    assert_eq!(fallback_warp.resolved_project_tempo_segment_id, None);
    assert!((fallback_warp.clips[0].realized_ratio - 0.75).abs() < 0.000_1);

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
