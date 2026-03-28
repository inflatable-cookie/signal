use super::super::super::*;

#[test]
fn server_host_shared_report_surfaces_runtime_media_service_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    host.runtime
        .handshake(HandshakeRequest {
            client_version: "signal-host-server".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("handshake");
    host.runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("configure");

    let imported_path = temp_media_fixture_path("server-media-service");
    fs::write(&imported_path, b"signal media fixture").expect("write media fixture");
    host.runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:server-media".into(),
            content_hash: "server-media".into(),
            source_path: imported_path.display().to_string(),
            file_name: "server-media.bin".into(),
            byte_size: fs::metadata(&imported_path)
                .expect("fixture metadata")
                .len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 128,
            waveform_bin_count: 12,
        }])
        .expect("media reconcile");
    host.runtime
        .start_media_preview("asset:sha256:server-media")
        .expect("start media preview");

    let report = host.supervisor_report();
    assert_eq!(report.observation.media_pipeline_snapshot.asset_count, 1);
    assert_eq!(
        report.observation.media_pipeline_snapshot.ready_asset_count,
        1
    );
    assert_eq!(report.observation.media_service_snapshot.indexed_asset_count, 1);
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
        Some("asset:sha256:server-media")
    );
    assert_eq!(report.observation.media_library_snapshot.indexed_asset_count, 1);
    assert_eq!(report.observation.media_library_snapshot.ready_descriptor_count, 0);
    assert_eq!(
        report.observation.media_library_snapshot.loudness_ready_descriptor_count,
        0
    );
    assert_eq!(
        report
            .observation
            .media_library_snapshot
            .character_ready_descriptor_count,
        0
    );
    assert_eq!(
        report.observation.media_library_snapshot.unavailable_descriptor_count,
        1
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"media_pipeline_snapshot\":{"));
    assert!(rendered.contains("\"media_service_snapshot\":{"));
    assert!(rendered.contains("\"media_library_snapshot\":{"));
    assert!(rendered.contains("\"preview_state\":\"Previewing\""));
    assert!(rendered.contains("\"unavailable_descriptor_count\":1"));

    let _ = fs::remove_file(&imported_path);
    if let Some(path) = host
        .runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
