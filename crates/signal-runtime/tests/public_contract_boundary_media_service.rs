use std::fs;

#[path = "support/public_contract_boundary_media.rs"]
mod public_contract_boundary_media_support;

use public_contract_boundary_media_support::{public_media_fixture_path, write_public_test_wav};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeMediaPreviewState, RuntimeObservationApi, RuntimeObservationReport,
    RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-media-service".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime media-service handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime media-service configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let ready_path = public_media_fixture_path("ready");
    let missing_path = public_media_fixture_path("missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-media-ready".into(),
                content_hash: "public-media-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "public-media-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-media-missing".into(),
                content_hash: "public-media-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "public-media-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("public runtime media assets should reconcile");
    runtime
        .start_media_preview("asset:sha256:public-media-ready")
        .expect("public runtime media preview should start");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(observation.media_pipeline_snapshot.asset_count, 2);
    assert_eq!(observation.media_pipeline_snapshot.ready_asset_count, 1);
    assert_eq!(observation.media_pipeline_snapshot.invalid_asset_count, 1);
    assert_eq!(observation.media_service_snapshot.indexed_asset_count, 2);
    assert_eq!(
        observation
            .media_service_snapshot
            .analysis_ready_asset_count,
        1
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .waveform_ready_asset_count,
        1
    );
    assert_eq!(
        observation.media_service_snapshot.previewable_asset_count,
        1
    );
    assert_eq!(
        observation.media_service_snapshot.invalidated_asset_count,
        1
    );
    assert!(observation.media_service_snapshot.invalidation_active);
    assert_eq!(
        observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .previewing_asset_id
            .as_deref(),
        Some("asset:sha256:public-media-ready")
    );
    assert_eq!(
        observation
            .media_service_snapshot
            .last_invalidated_asset_id
            .as_deref(),
        Some("asset:sha256:public-media-missing")
    );
    assert!(observation
        .media_service_snapshot
        .last_invalidation_error
        .is_some());
    assert_eq!(
        supervisor.observation.media_pipeline_snapshot.asset_count,
        observation.media_pipeline_snapshot.asset_count
    );
    assert_eq!(
        supervisor.observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"media_pipeline_snapshot\":{"));
    assert!(observation_json.contains("\"media_service_snapshot\":{"));
    assert!(observation_json.contains("\"invalidated_asset_count\":1"));
    assert!(observation_json.contains("\"preview_state\":\"Previewing\""));
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"media_pipeline_snapshot\":{"));
    assert!(supervisor_json.contains("\"media_service_snapshot\":{"));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:public-media-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
