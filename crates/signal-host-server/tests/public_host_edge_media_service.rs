#[path = "support/public_host_edge_media.rs"]
mod public_host_edge_media;

use std::fs;

use public_host_edge_media::{public_server_media_fixture_path, write_public_test_wav};
use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeLifecycleApi, RuntimeMediaPreviewState,
    RuntimeObservationApi, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_media_service_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-server-media-service".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge media-service handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("server host-edge media-service configure should succeed");

    let ready_path = public_server_media_fixture_path("ready");
    let missing_path = public_server_media_fixture_path("missing");
    write_public_test_wav(&ready_path);

    runtime
        .reconcile_media_assets(vec![
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-server-media-ready".into(),
                content_hash: "host-server-media-ready".into(),
                source_path: ready_path.display().to_string(),
                file_name: "host-server-media-ready.wav".into(),
                byte_size: fs::metadata(&ready_path)
                    .expect("public server media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
            signal_runtime::RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:host-server-media-missing".into(),
                content_hash: "host-server-media-missing".into(),
                source_path: missing_path.display().to_string(),
                file_name: "host-server-media-missing.wav".into(),
                byte_size: 0,
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 128,
                waveform_bin_count: 16,
            },
        ])
        .expect("server host-edge media assets should reconcile");
    runtime
        .start_media_preview("asset:sha256:host-server-media-ready")
        .expect("server host-edge media preview should start");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.media_pipeline_snapshot.asset_count, 2);
    assert_eq!(
        report.observation.media_pipeline_snapshot.ready_asset_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_pipeline_snapshot
            .invalid_asset_count,
        1
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .indexed_asset_count,
        2
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .waveform_ready_asset_count,
        1
    );
    assert_eq!(
        report.observation.media_service_snapshot.preview_state,
        RuntimeMediaPreviewState::Previewing
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .previewing_asset_id
            .as_deref(),
        Some("asset:sha256:host-server-media-ready")
    );
    assert_eq!(
        report
            .observation
            .media_service_snapshot
            .last_invalidated_asset_id
            .as_deref(),
        Some("asset:sha256:host-server-media-missing")
    );
    assert!(
        report
            .observation
            .media_service_snapshot
            .invalidation_active
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"media_pipeline_snapshot\":{"));
    assert!(rendered.contains("\"media_service_snapshot\":{"));
    assert!(rendered.contains("\"invalidated_asset_count\":1"));
    assert!(rendered.contains("\"preview_state\":\"Previewing\""));

    let _ = fs::remove_file(&ready_path);
    if let Some(path) = host
        .runtime()
        .get_media_pipeline_snapshot()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "asset:sha256:host-server-media-ready")
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
