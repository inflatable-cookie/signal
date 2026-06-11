use std::fs;
use std::path::PathBuf;

use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
use signal_runtime::{
    RuntimeObservationApi, RuntimePreviewTransformReadiness, RuntimePreviewTransformServiceClass,
    RuntimeSupervisorReport, SignalRuntime,
};

pub(crate) fn assert_preview_transform_render_and_preview(runtime: &SignalRuntime) {
    let rendered = runtime
        .render_clip_processing_buffer(signal_runtime::RuntimeClipRenderRequest {
            clip_id: "clip:public-preview-transform".into(),
            timeline_start_samples: 0,
            input_stage: signal_runtime::RuntimeClipRenderInputStage::PostWarp,
            buffer: AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.25; 8],
            ),
        })
        .expect("public preview-transform clip render should succeed");
    assert_eq!(
        rendered.preview_transform_snapshot.service_class,
        RuntimePreviewTransformServiceClass::ArtifactBacked
    );
    assert_eq!(
        rendered.preview_transform_snapshot.readiness,
        RuntimePreviewTransformReadiness::Ready
    );
    assert!(rendered.preview_transform_snapshot.audition_active);
}

pub(crate) fn assert_preview_transform_supervisor(_supervisor: &RuntimeSupervisorReport) {}

pub(crate) fn cleanup_preview_transform_runtime(runtime: &SignalRuntime, ready_path: &PathBuf) {
    let _ = fs::remove_file(ready_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
