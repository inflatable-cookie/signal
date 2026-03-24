#[path = "support/public_host_edge_runtime_surface.rs"]
mod public_host_edge_runtime_surface;

use std::fs;

use public_host_edge_runtime_surface::{public_server_media_fixture_path, write_public_test_wav};
use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi, RuntimeMediaAnalysisDescriptorState,
    RuntimeObservationApi, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_analysis_metadata_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-analysis-metadata".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge analysis-metadata handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("server host-edge analysis-metadata configure should succeed");

    let ready_path = public_server_media_fixture_path("analysis-ready");
    let missing_path = public_server_media_fixture_path("analysis-missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-server-analysis-ready".into(),
                content_hash: "host-server-analysis-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "host-server-analysis-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public server analysis fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-server-analysis-missing".into(),
                content_hash: "host-server-analysis-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "host-server-analysis-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("server host-edge analysis metadata assets should reconcile");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .indexed_asset_count,
        2
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .ready_descriptor_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .invalidated_descriptor_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .loudness_ready_descriptor_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .character_ready_descriptor_count,
        1
    );
    let ready = report
        .observation
        .media_library_snapshot
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:host-server-analysis-ready")
        .expect("server host-edge ready analysis descriptor");
    assert_eq!(
        ready.metadata_state,
        RuntimeMediaAnalysisDescriptorState::Ready
    );
    assert!(ready.loudness.is_some());
    assert!(ready.character.is_some());
    let invalidated = report
        .observation
        .media_library_snapshot
        .descriptors
        .iter()
        .find(|descriptor| descriptor.asset_id == "asset:sha256:host-server-analysis-missing")
        .expect("server host-edge invalidated analysis descriptor");
    assert_eq!(
        invalidated.metadata_state,
        RuntimeMediaAnalysisDescriptorState::Invalidated
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"media_library_snapshot\":{"));
    assert!(rendered.contains("\"ready_descriptor_count\":1"));
    assert!(rendered.contains("\"invalidated_descriptor_count\":1"));
    assert!(rendered.contains("\"loudness_ready_descriptor_count\":1"));
    assert!(rendered.contains("\"character_ready_descriptor_count\":1"));
    assert!(rendered.contains("\"metadata_state\":\"Ready\""));
    assert!(rendered.contains("\"metadata_state\":\"Invalidated\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:host-server-analysis-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
