use std::fs;
use std::path::PathBuf;

use signal_runtime::{
    HandshakeRequest, RuntimeClipFadeEnvelope, RuntimeClipGainEnvelope,
    RuntimeClipProcessingRegistration, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
    RuntimeProjectionApi, RuntimeWarpClipRegistration, RuntimeWarpMode, SignalRuntime,
    TransportProjection,
};

use crate::public_contract_boundary_media_support::{
    public_media_fixture_path, write_public_transient_test_wav,
};

pub(crate) fn configured_preview_transform_runtime() -> (SignalRuntime, PathBuf) {
    let mut runtime = SignalRuntime::new(signal_runtime::RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-preview-transform-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime preview-transform handshake should succeed");
    runtime
        .configure(signal_runtime::RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime preview-transform configure should succeed");

    let ready_path = public_media_fixture_path("preview-transform-ready");
    write_public_transient_test_wav(&ready_path);
    runtime
        .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
            asset_id: "asset:sha256:public-preview-transform-ready".into(),
            content_hash: "public-preview-transform-ready".into(),
            source_path: ready_path.display().to_string(),
            file_name: "public-preview-transform-ready.wav".into(),
            byte_size: fs::metadata(&ready_path).unwrap().len(),
            sample_rate_hz: 48_000,
            channel_count: 1,
            duration_samples: 48_000,
            waveform_bin_count: 8,
        }])
        .expect("public preview-transform media asset should reconcile");
    runtime
        .reconcile_warp_clips(vec![RuntimeWarpClipRegistration {
            clip_id: "clip:public-preview-transform".into(),
            media_asset_id: Some("asset:sha256:public-preview-transform-ready".into()),
            mode: RuntimeWarpMode::ElastiqueDraft,
            source_tempo_bpm: Some(120.0),
            anchor_timeline_samples: 0,
            start_samples: 0,
            duration_samples: 48_000,
        }])
        .expect("public preview-transform warp clip should reconcile");
    runtime
        .reconcile_clip_processing_clips(vec![RuntimeClipProcessingRegistration {
            clip_id: "clip:public-preview-transform".into(),
            media_asset_id: Some("asset:sha256:public-preview-transform-ready".into()),
            warp_mode: RuntimeWarpMode::ElastiqueDraft,
            start_samples: 0,
            duration_samples: 48_000,
            fade_in: RuntimeClipFadeEnvelope::default(),
            fade_out: RuntimeClipFadeEnvelope::default(),
            clip_gain: RuntimeClipGainEnvelope::default(),
        }])
        .expect("public preview-transform clip-processing clip should reconcile");
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 0,
            tempo_bpm: 180.0,
            loop_state: None,
        })
        .expect("public preview-transform transport projection should apply");
    runtime
        .start_media_preview("asset:sha256:public-preview-transform-ready")
        .expect("public preview-transform media preview should start");

    (runtime, ready_path)
}
