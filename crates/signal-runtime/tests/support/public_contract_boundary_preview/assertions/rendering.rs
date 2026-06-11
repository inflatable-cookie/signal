use std::fs;
use std::path::PathBuf;

use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
use signal_runtime::{
    RuntimeMediaAuditionContinuityOutcome, RuntimeObservationApi,
    RuntimeOfflineRenderContractPreview, RuntimeOfflineRenderRequest,
    RuntimePreviewBrowserQueueClass, RuntimePreviewBrowserQueueOutcome,
    RuntimePreviewBrowserQueuePosture, RuntimePreviewOutputRoutingPosture,
    RuntimePreviewTransformReadiness, RuntimePreviewTransformSchedulingOutcome,
    RuntimePreviewTransformServiceClass, RuntimeSupervisorReport, SignalRuntime,
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

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:public-preview-transform-preview".into(),
            timeline_start_samples: 0,
            duration_samples: 24_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        },
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &runtime.get_plugin_recall_handoff_snapshot(),
    )
    .expect("public preview-transform preview should build");
    assert_eq!(preview.preview_transform_snapshot.clip_count, 1);
    assert_eq!(preview.preview_transform_snapshot.ready_clip_count, 1);
    assert_eq!(
        preview
            .preview_transform_snapshot
            .artifact_backed_clip_count,
        1
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .active_audition_clip_count,
        0
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_device_policy
            .routing_posture,
        RuntimePreviewOutputRoutingPosture::NoPreviewOutputRouting
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_posture,
        RuntimePreviewBrowserQueuePosture::GuardedPreviewQueue
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_class,
        RuntimePreviewBrowserQueueClass::PreviewAssetSelectionQueue
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .queue_outcome,
        RuntimePreviewBrowserQueueOutcome::CollapseToSingleActivePreview
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .audition_continuity_outcome,
        RuntimeMediaAuditionContinuityOutcome::ResumePreviewAudition
    );
    assert_eq!(
        preview
            .preview_transform_snapshot
            .preview_workflow
            .transform_scheduling_outcome,
        RuntimePreviewTransformSchedulingOutcome::PreferArtifactBackedPreview
    );
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
