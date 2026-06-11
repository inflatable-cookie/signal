use std::fs;

#[path = "support/public_contract_boundary_media.rs"]
mod public_contract_boundary_media_support;

use public_contract_boundary_media_support::{
    public_media_fixture_path, write_public_transient_test_wav,
};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeObservationApi, RuntimeObservationReport, RuntimeProjectionApi,
    RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_marker_analysis_boundary_reports_runtime_owned_analysis_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-marker-analysis-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime marker-analysis handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime marker-analysis configure should succeed");

    let ready_path = public_media_fixture_path("marker-analysis-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![signal_runtime::RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:public-marker-analysis-ready".into(),
            content_hash: "public-marker-analysis-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "public-marker-analysis-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public marker-analysis media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
            clip_id: "clip:public-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:public-marker-analysis-ready".into()),
            mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public marker-analysis warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![signal_runtime::RuntimeClipProcessingRegistration {
            clip_id: "clip:public-marker-analysis".into(),
            media_asset_id: Some("asset:sha256:public-marker-analysis-ready".into()),
            warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
            fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
            clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
        }])
        .expect("public marker-analysis clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(signal_runtime::TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public marker-analysis transport projection should apply");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(observation.marker_analysis_snapshot.clip_count, 1);
    assert_eq!(observation.marker_analysis_snapshot.ready_clip_count, 1);
    assert_eq!(
        observation
            .marker_analysis_snapshot
            .tempo_assist_ready_clip_count,
        1
    );
    assert!(observation.marker_analysis_snapshot.warp_marker_count > 0);
    assert!(observation.marker_analysis_snapshot.transient_anchor_count > 0);
    assert_eq!(
        observation.marker_analysis_snapshot.clips[0].tempo_assist_posture,
        signal_runtime::RuntimeTempoAssistPosture::Ready
    );
    assert_eq!(
        observation.marker_analysis_snapshot.clips[0].tempo_assist_hint_source,
        signal_runtime::RuntimeTempoAssistHintSource::SourceTempo
    );
    assert_eq!(
        observation.marker_analysis_snapshot.clips[0].tempo_assist_hint_bpm,
        Some(120.0)
    );

    let _supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
