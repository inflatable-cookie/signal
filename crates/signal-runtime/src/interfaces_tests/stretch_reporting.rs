use super::*;

#[test]
fn runtime_stretch_engine_snapshot_derives_from_clip_processing_baselines() {
    let pipeline = RuntimeClipProcessingPipelineSnapshot {
        clip_count: 4,
        ready_clip_count: 3,
        pending_media_clip_count: 0,
        pending_warp_clip_count: 0,
        invalid_clip_count: 1,
        faded_clip_count: 0,
        gain_shaped_clip_count: 0,
        warped_clip_count: 3,
        treatment_stage_count: 3,
        clips: vec![
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-disabled".into(),
                media_asset_id: None,
                warp_mode: RuntimeWarpMode::Off,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: Vec::new(),
                realized_warp_ratio: None,
                project_tempo_source: None,
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
                summary: "disabled clip".into(),
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-ratio".into(),
                media_asset_id: Some("asset:ratio".into()),
                warp_mode: RuntimeWarpMode::Repitch,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(0.75),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
                summary: "ratio clip".into(),
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-sample-domain".into(),
                media_asset_id: Some("asset:sample-domain".into()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(1.5),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Ready,
                last_error: None,
                summary: "sample-domain clip".into(),
            },
            RuntimeClipProcessingSnapshot {
                clip_id: "clip:stretch-fallback".into(),
                media_asset_id: Some("asset:fallback".into()),
                warp_mode: RuntimeWarpMode::ElastiqueDraft,
                start_samples: 0,
                duration_samples: 64,
                fade_in: RuntimeClipFadeEnvelope::default(),
                fade_out: RuntimeClipFadeEnvelope::default(),
                fade_in_end_samples: 0,
                fade_out_start_samples: 64,
                clip_gain: RuntimeClipGainEnvelope::default(),
                treatment_stages: vec![RuntimeClipProcessingStage::Warp],
                realized_warp_ratio: Some(0.6),
                project_tempo_source: Some(RuntimeTempoSource::TransportProjection),
                project_tempo_segment_id: None,
                readiness: RuntimeClipProcessingReadiness::Invalid,
                last_error: Some("outside baseline support".into()),
                summary: "fallback clip".into(),
            },
        ],
        summary: "clip processing stretch baseline".into(),
    };

    let stretch = RuntimeStretchEngineSnapshot::from_clip_processing_pipeline(&pipeline);

    assert_eq!(stretch.clip_count, 4);
    assert_eq!(stretch.disabled_clip_count, 1);
    assert_eq!(stretch.ready_clip_count, 2);
    assert_eq!(stretch.pending_media_clip_count, 0);
    assert_eq!(stretch.pending_warp_clip_count, 0);
    assert_eq!(stretch.degraded_clip_count, 1);
    assert_eq!(stretch.sample_domain_clip_count, 1);
    assert_eq!(stretch.ratio_only_clip_count, 1);
    assert_eq!(stretch.fallback_clip_count, 1);
    assert_eq!(
        stretch.clips[0].engine_class,
        RuntimeStretchEngineClass::Disabled
    );
    assert_eq!(
        stretch.clips[1].engine_class,
        RuntimeStretchEngineClass::RatioOnly
    );
    assert_eq!(stretch.clips[1].readiness, RuntimeStretchReadiness::Ready);
    assert_eq!(
        stretch.clips[2].engine_class,
        RuntimeStretchEngineClass::SampleDomain
    );
    assert_eq!(stretch.clips[2].readiness, RuntimeStretchReadiness::Ready);
    assert_eq!(
        stretch.clips[3].engine_class,
        RuntimeStretchEngineClass::Fallback
    );
    assert_eq!(
        stretch.clips[3].readiness,
        RuntimeStretchReadiness::Degraded
    );
    assert_eq!(
        stretch.clips[3].fallback_kind,
        RuntimeStretchFallbackKind::RatioOnly
    );
    assert!(stretch.summary.contains("sample_domain=1"));
    assert!(stretch.summary.contains("fallback=1"));
}

#[test]
fn runtime_observation_report_render_json_surfaces_external_midi_snapshot() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
    let recorder = RuntimeEventRecorder::default();
    let report = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_external_midi_snapshot(RuntimeExternalMidiEndpointGraphSnapshot::empty(
            "signal-host-server",
        ));

    assert_eq!(
        report.external_midi_snapshot.discovery_state,
        RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.external_midi_snapshot.graph_state,
        RuntimeExternalMidiGraphState::Empty
    );

    let compact = report.render_compact();
    assert!(compact.contains("external_midi=Idle/Empty"));

    let json = report.render_json();
    assert!(json.contains("\"external_midi_snapshot\":{"));
    assert!(json.contains("\"control_surface_snapshot\":{"));
    assert!(json.contains("\"advanced_hardware_snapshot\":{"));
    assert!(json.contains("\"display_transport_device_count\":0"));
    assert!(json.contains("\"motor_transport_device_count\":0"));
    assert!(json.contains("\"haptic_transport_device_count\":0"));
    assert!(json.contains("\"scene_mapping_device_count\":0"));
    assert!(json.contains("\"feedback_page_device_count\":0"));
    assert!(json.contains("\"safe_action_graph_device_count\":0"));
    assert!(json.contains("\"stretch_engine_snapshot\":{\"clip_count\":0"));
    assert!(json.contains("\"discovery_state\":\"Idle\""));
    assert!(json.contains("\"graph_state\":\"Empty\""));
    assert!(json.contains("\"provider_name\":\"signal-host-server\""));
}
