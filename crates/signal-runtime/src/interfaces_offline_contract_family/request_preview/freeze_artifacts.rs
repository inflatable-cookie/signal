use super::super::*;

pub(super) fn build_offline_freeze_artifact_previews(
    request: &RuntimeOfflineRenderRequest,
    stem_targets: &[RuntimeOfflineRenderStemPreview],
    recall_handoff: &RuntimePluginRecallHandoffSnapshot,
) -> Result<Vec<RuntimeOfflineFreezeArtifactPreview>, RuntimeError> {
    let mut freeze_artifacts = Vec::with_capacity(request.freeze_artifacts.len());
    for artifact in &request.freeze_artifacts {
        if artifact.artifact_id.trim().is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline freeze artifacts require a non-empty artifact id",
            ));
        }
        if !request.include_main_mix
            && !stem_targets
                .iter()
                .any(|stem| stem.stem_id == artifact.source_stem_id)
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "offline freeze artifact `{}` references unknown stem `{}`",
                    artifact.artifact_id, artifact.source_stem_id
                ),
            ));
        }
        let resolved_selection = recall_handoff
            .resolve_selection(&artifact.recall_selection)
            .ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline freeze artifact `{}` references an unknown recall handoff stage",
                        artifact.artifact_id
                    ),
                )
            })?;
        freeze_artifacts.push(RuntimeOfflineFreezeArtifactPreview {
            artifact_id: artifact.artifact_id.clone(),
            source_stem_id: artifact.source_stem_id.clone(),
            recall_stage_count: resolved_selection.len(),
            recall_stage_ids: resolved_selection
                .iter()
                .map(|stage| stage.stage_id.clone())
                .collect(),
            recall_states: resolved_selection
                .iter()
                .map(|stage| stage.recall_state)
                .collect(),
            summary: format!(
                "artifact={} source_stem={} recall_stages={} recall_states={:?}",
                artifact.artifact_id,
                artifact.source_stem_id,
                resolved_selection.len(),
                resolved_selection
                    .iter()
                    .map(|stage| stage.recall_state)
                    .collect::<Vec<_>>(),
            ),
        });
    }
    Ok(freeze_artifacts)
}
