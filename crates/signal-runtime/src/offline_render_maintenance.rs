use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct OfflineRenderPurgeOutcome {
    pub(super) removed: bool,
    pub(super) file_count: usize,
    pub(super) byte_count: u64,
}

fn offline_render_file_size(path: &Path) -> Result<Option<u64>, RuntimeError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata.len())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!(
                "failed to inspect offline render file {}: {error}",
                path.display()
            ),
        )),
    }
}

fn offline_render_directory_stats(path: &Path) -> Result<Option<(usize, u64)>, RuntimeError> {
    let mut file_count = 0usize;
    let mut byte_count = 0u64;
    let mut directories = VecDeque::from([path.to_path_buf()]);
    while let Some(directory) = directories.pop_front() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return if directory == path {
                    Ok(None)
                } else {
                    Err(RuntimeError::new(
                        RuntimeErrorKind::ResourceUnavailable,
                        format!(
                            "offline render artifact root changed while purging {}",
                            path.display()
                        ),
                    ))
                };
            }
            Err(error) => {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!(
                        "failed to inspect offline render artifact root {}: {error}",
                        directory.display()
                    ),
                ));
            }
        };
        for entry in entries {
            let entry = entry.map_err(|error| {
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!(
                        "failed to inspect offline render artifact entry under {}: {error}",
                        path.display()
                    ),
                )
            })?;
            let entry_path = entry.path();
            let metadata = entry.metadata().map_err(|error| {
                RuntimeError::new(
                    RuntimeErrorKind::ResourceUnavailable,
                    format!(
                        "failed to inspect offline render artifact metadata for {}: {error}",
                        entry_path.display()
                    ),
                )
            })?;
            if metadata.is_dir() {
                directories.push_back(entry_path);
            } else {
                file_count = file_count.saturating_add(1);
                byte_count = byte_count.saturating_add(metadata.len());
            }
        }
    }
    Ok(Some((file_count, byte_count)))
}

pub(super) fn purge_offline_render_file(
    path: &Path,
) -> Result<OfflineRenderPurgeOutcome, RuntimeError> {
    let byte_count = offline_render_file_size(path)?.unwrap_or(0);
    match fs::remove_file(path) {
        Ok(()) => Ok(OfflineRenderPurgeOutcome {
            removed: true,
            file_count: 1,
            byte_count,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(OfflineRenderPurgeOutcome::default())
        }
        Err(error) => Err(RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!(
                "failed to remove offline render file {}: {error}",
                path.display()
            ),
        )),
    }
}

pub(super) fn purge_offline_render_directory(
    path: &Path,
) -> Result<OfflineRenderPurgeOutcome, RuntimeError> {
    let (file_count, byte_count) = offline_render_directory_stats(path)?.unwrap_or((0, 0));
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(OfflineRenderPurgeOutcome {
            removed: true,
            file_count,
            byte_count,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(OfflineRenderPurgeOutcome::default())
        }
        Err(error) => Err(RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            format!(
                "failed to remove offline render artifact root {}: {error}",
                path.display()
            ),
        )),
    }
}

fn refresh_offline_render_stem_result(stem: &mut RuntimeOfflineRenderStemResult) {
    stem.peak_level = peak_abs(stem.output.samples());
    stem.rms_level = rms(stem.output.samples());
}

fn refresh_offline_freeze_artifact_result(artifact: &mut RuntimeOfflineFreezeArtifactResult) {
    artifact.peak_level = peak_abs(artifact.output.samples());
    artifact.rms_level = rms(artifact.output.samples());
}

pub(super) fn refresh_offline_render_result(result: &mut RuntimeOfflineRenderResult) {
    if let Some(main_mix) = result.main_mix.as_ref() {
        result.main_mix_peak_level = Some(peak_abs(main_mix.samples()));
        result.main_mix_rms_level = Some(rms(main_mix.samples()));
    } else {
        result.main_mix_peak_level = None;
        result.main_mix_rms_level = None;
    }
    for stem in &mut result.stems {
        refresh_offline_render_stem_result(stem);
    }
    for artifact in &mut result.freeze_artifacts {
        refresh_offline_freeze_artifact_result(artifact);
    }
    result.rendered_frame_count = result
        .main_mix
        .as_ref()
        .map(|buffer| buffer.frames().0)
        .or_else(|| result.stems.first().map(|stem| stem.output.frames().0))
        .or_else(|| {
            result
                .freeze_artifacts
                .first()
                .map(|artifact| artifact.output.frames().0)
        })
        .unwrap_or(0);
}
