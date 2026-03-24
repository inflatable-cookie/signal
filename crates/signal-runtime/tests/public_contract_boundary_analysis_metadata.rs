use std::fs;

#[path = "support/public_contract_boundary_media.rs"]
mod public_contract_boundary_media_support;

use public_contract_boundary_media_support::{public_media_fixture_path, write_public_test_wav};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeMediaAnalysisDescriptorState, RuntimeMediaAnalysisFamilyState,
    RuntimeObservationApi, RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_analysis_metadata_boundary_reports_runtime_owned_library_descriptors() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-analysis-metadata".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime analysis-metadata handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime analysis-metadata configure should succeed");
    let recorder = RuntimeEventRecorder::default();

    let ready_path = public_media_fixture_path("analysis-ready");
    let missing_path = public_media_fixture_path("analysis-missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-analysis-ready".into(),
                content_hash: "public-analysis-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "public-analysis-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public analysis fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:public-analysis-missing".into(),
                content_hash: "public-analysis-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "public-analysis-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("public runtime analysis metadata assets should reconcile");

    let library_snapshot = runtime.get_media_library_service_snapshot();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    assert_eq!(library_snapshot.indexed_asset_count, 2);
    assert_eq!(library_snapshot.ready_descriptor_count, 1);
    assert_eq!(library_snapshot.invalidated_descriptor_count, 1);
    assert_eq!(library_snapshot.unavailable_descriptor_count, 0);
    assert_eq!(library_snapshot.loudness_ready_descriptor_count, 1);
    assert_eq!(library_snapshot.character_ready_descriptor_count, 1);
    let ready = library_snapshot
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:public-analysis-ready")
        .expect("public ready analysis descriptor");
    assert_eq!(
        ready.metadata_state,
        RuntimeMediaAnalysisDescriptorState::Ready
    );
    assert_eq!(ready.loudness_state, RuntimeMediaAnalysisFamilyState::Ready);
    assert_eq!(
        ready.character_state,
        RuntimeMediaAnalysisFamilyState::Ready
    );
    assert_eq!(
        ready.rhythm_state,
        RuntimeMediaAnalysisFamilyState::Deferred
    );
    assert_eq!(ready.tonal_state, RuntimeMediaAnalysisFamilyState::Deferred);
    assert_eq!(
        ready.embedding_state,
        RuntimeMediaAnalysisFamilyState::Deferred
    );
    assert!(ready.loudness.is_some());
    assert!(ready.character.is_some());
    let loudness = ready.loudness.as_ref().expect("public loudness descriptor");
    assert!(loudness.integrated_lufs.is_finite());
    assert!(loudness.true_peak_dbtp.is_finite());
    let character = ready
        .character
        .as_ref()
        .expect("public character descriptor");
    assert!(character.centroid_hz.is_finite());
    assert!(character.dynamic_range.is_finite());

    let invalidated = library_snapshot
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:public-analysis-missing")
        .expect("public invalidated analysis descriptor");
    assert_eq!(
        invalidated.metadata_state,
        RuntimeMediaAnalysisDescriptorState::Invalidated
    );
    assert!(invalidated.last_error.is_some());

    assert_eq!(
        observation.media_library_snapshot.ready_descriptor_count,
        library_snapshot.ready_descriptor_count
    );
    assert_eq!(
        supervisor
            .observation
            .media_library_snapshot
            .invalidated_descriptor_count,
        library_snapshot.invalidated_descriptor_count
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"media_library_snapshot\":{"));
    assert!(observation_json.contains("\"ready_descriptor_count\":1"));
    assert!(observation_json.contains("\"invalidated_descriptor_count\":1"));
    assert!(observation_json.contains("\"loudness_ready_descriptor_count\":1"));
    assert!(observation_json.contains("\"character_ready_descriptor_count\":1"));
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"media_library_snapshot\":{"));
    assert!(supervisor_json.contains("\"metadata_state\":\"Ready\""));
    assert!(supervisor_json.contains("\"metadata_state\":\"Invalidated\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:public-analysis-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
