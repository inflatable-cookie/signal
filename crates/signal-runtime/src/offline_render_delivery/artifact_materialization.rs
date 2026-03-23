use super::*;

fn render_artifact_receipt(
    artifact_id: String,
    artifact_kind: RuntimeOfflineRenderArtifactKind,
    output_path: String,
    byte_size: u64,
    buffer: &AudioBuffer,
) -> RuntimeOfflineRenderArtifactReceipt {
    let peak_level = peak_abs(buffer.samples());
    let rms_level = rms(buffer.samples());
    RuntimeOfflineRenderArtifactReceipt {
        artifact_id: artifact_id.clone(),
        artifact_kind,
        output_path: output_path.clone(),
        sample_rate_hz: buffer.sample_rate().0,
        channel_count: buffer.channel_count().0,
        frame_count: buffer.frames().0,
        byte_size,
        peak_level,
        rms_level,
        summary: format!(
            "artifact={} kind={:?} path={} sample_rate={} channels={} frames={} bytes={} peak={:.3} rms={:.3}",
            artifact_id,
            artifact_kind,
            output_path,
            buffer.sample_rate().0,
            buffer.channel_count().0,
            buffer.frames().0,
            byte_size,
            peak_level,
            rms_level,
        ),
    }
}

pub(super) fn materialize_offline_render_delivery(
    result: &RuntimeOfflineRenderResult,
) -> Result<RuntimeOfflineRenderManifest, RuntimeError> {
    let delegated_execution_request = result.manifest.delegated_execution_request.clone();
    let delegated_execution_receipt = result.manifest.delegated_execution_receipt.clone();
    let Some(root_path) = result.manifest.artifact_root_path.as_deref() else {
        return Ok(super::manifest_report::offline_render_manifest(
            &result.request_id,
            None,
            Vec::new(),
            None,
            delegated_execution_request,
            delegated_execution_receipt,
        ));
    };

    let root = Path::new(root_path);
    fs::create_dir_all(root).map_err(|error| {
        RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!("failed to create offline render artifact directory: {error}"),
        )
    })?;
    let request_slug = sanitize_asset_id(&result.request_id);
    let mut artifact_receipts = Vec::new();
    if let Some(main_mix) = result.main_mix.as_ref() {
        let path = root.join(format!("{request_slug}-main-mix.wav"));
        write_audio_buffer_wav(&path, main_mix)?;
        let byte_size = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .map_err(|error| {
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!(
                        "failed to inspect offline render artifact {}: {error}",
                        path.display()
                    ),
                )
            })?;
        artifact_receipts.push(render_artifact_receipt(
            "main_mix".into(),
            RuntimeOfflineRenderArtifactKind::MainMix,
            path.display().to_string(),
            byte_size,
            main_mix,
        ));
    }
    for stem in &result.stems {
        let path = root.join(format!(
            "{request_slug}-stem-{}.wav",
            sanitize_asset_id(&stem.stem_id)
        ));
        write_audio_buffer_wav(&path, &stem.output)?;
        let byte_size = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .map_err(|error| {
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!(
                        "failed to inspect offline render artifact {}: {error}",
                        path.display()
                    ),
                )
            })?;
        artifact_receipts.push(render_artifact_receipt(
            stem.stem_id.clone(),
            RuntimeOfflineRenderArtifactKind::Stem,
            path.display().to_string(),
            byte_size,
            &stem.output,
        ));
    }
    for artifact in &result.freeze_artifacts {
        let path = root.join(format!(
            "{request_slug}-freeze-{}.wav",
            sanitize_asset_id(&artifact.artifact_id)
        ));
        write_audio_buffer_wav(&path, &artifact.output)?;
        let byte_size = fs::metadata(&path)
            .map(|metadata| metadata.len())
            .map_err(|error| {
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!(
                        "failed to inspect offline render artifact {}: {error}",
                        path.display()
                    ),
                )
            })?;
        artifact_receipts.push(render_artifact_receipt(
            artifact.artifact_id.clone(),
            RuntimeOfflineRenderArtifactKind::FreezeArtifact,
            path.display().to_string(),
            byte_size,
            &artifact.output,
        ));
    }

    let report_path = root.join(format!("{request_slug}-report.json"));
    let result_for_report = RuntimeOfflineRenderResult {
        manifest: super::manifest_report::offline_render_manifest(
            &result.request_id,
            Some(root_path),
            artifact_receipts.clone(),
            None,
            delegated_execution_request.clone(),
            delegated_execution_receipt.clone(),
        ),
        ..result.clone()
    };
    super::manifest_report::write_offline_render_report(&report_path, &result_for_report)?;
    let report_receipt = super::manifest_report::offline_render_report_receipt(
        &result.request_id,
        &report_path,
        artifact_receipts.len(),
    )?;

    Ok(super::manifest_report::offline_render_manifest(
        &result.request_id,
        Some(root_path),
        artifact_receipts,
        Some(report_receipt),
        delegated_execution_request,
        delegated_execution_receipt,
    ))
}
