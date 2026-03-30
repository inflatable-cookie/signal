use super::*;

pub(super) fn build_request(runtime: &SignalRuntime) -> RuntimeOfflineRenderRequest {
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    RuntimeOfflineRenderRequest {
        request_id: "render:preview".into(),
        timeline_start_samples: 0,
        duration_samples: 48_000,
        export_sample_rate_hz: 48_000,
        include_main_mix: true,
        artifact_root_path: None,
        stem_targets: vec![RuntimeOfflineRenderStemTarget {
            stem_id: "stem:track:lead".into(),
            target_kind: RuntimeOfflineRenderTargetKind::TrackLane,
            target_id: Some("track:lead".into()),
        }],
        freeze_artifacts: vec![RuntimeOfflineFreezeArtifactRequest {
            artifact_id: "freeze:track:lead".into(),
            source_stem_id: "stem:track:lead".into(),
            recall_selection: RuntimePluginRecallHandoffSelection {
                stage_count: 2,
                stage_ids: handoff
                    .stages
                    .iter()
                    .map(|stage| stage.stage_id.clone())
                    .collect(),
            },
        }],
    }
}
