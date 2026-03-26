use super::*;

impl RuntimeMediaPipelineStateModel {
    pub(crate) fn snapshot(&self) -> RuntimeMediaPipelineSnapshot {
        let assets = self
            .assets
            .values()
            .map(|asset| RuntimeMediaAssetSnapshot {
                asset_id: asset.registration.asset_id.clone(),
                content_hash: asset.registration.content_hash.clone(),
                source_path: asset.registration.source_path.clone(),
                file_name: asset.registration.file_name.clone(),
                byte_size: asset.registration.byte_size,
                sample_rate_hz: asset.registration.sample_rate_hz,
                channel_count: asset.registration.channel_count,
                duration_samples: asset.registration.duration_samples,
                waveform_bin_count: asset.registration.waveform_bin_count,
                state: Some(asset.state),
                cache_path: asset.cache_path.clone(),
                cache_byte_size: asset.cache_byte_size,
                rebuild_count: asset.rebuild_count,
                last_error: asset.last_error.clone(),
                summary: format!(
                    "state={:?} cache={} rebuilds={} error={}",
                    asset.state,
                    asset.cache_path.as_deref().unwrap_or("none"),
                    asset.rebuild_count,
                    asset.last_error.as_deref().unwrap_or("none"),
                ),
            })
            .collect::<Vec<_>>();
        let ready_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Ready))
            .count();
        let invalid_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Invalid))
            .count();
        let ingesting_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Ingesting))
            .count();
        let conforming_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Conforming))
            .count();
        let rebuilding_asset_count = assets
            .iter()
            .filter(|asset| asset.state == Some(RuntimeMediaAssetState::Rebuilding))
            .count();

        RuntimeMediaPipelineSnapshot {
            cache_root_path: self.policy.cache_root.display().to_string(),
            asset_count: assets.len(),
            ready_asset_count,
            invalid_asset_count,
            ingesting_asset_count,
            conforming_asset_count,
            rebuilding_asset_count,
            assets,
            summary: format!(
                "assets={} ready={} invalid={} rebuilding={} cache_root={}",
                self.assets.len(),
                ready_asset_count,
                invalid_asset_count,
                rebuilding_asset_count,
                self.policy.cache_root.display(),
            ),
        }
    }

    pub(crate) fn service_snapshot(&self) -> RuntimeMediaServiceSnapshot {
        let indexed_asset_count = self.assets.len();
        let analysis_ready_asset_count = self
            .assets
            .values()
            .filter(|asset| asset.state == RuntimeMediaAssetState::Ready)
            .count();
        let invalidated_assets = self
            .assets
            .values()
            .filter(|asset| asset.state == RuntimeMediaAssetState::Invalid)
            .collect::<Vec<_>>();
        let invalidated_asset_count = invalidated_assets.len();
        let waveform_ready_asset_count = self
            .assets
            .values()
            .filter(|asset| {
                asset.registration.waveform_bin_count > 0
                    && asset.state == RuntimeMediaAssetState::Ready
            })
            .count();
        let waveform_pending_asset_count = self
            .assets
            .values()
            .filter(|asset| {
                asset.registration.waveform_bin_count > 0
                    && matches!(
                        asset.state,
                        RuntimeMediaAssetState::Ingesting
                            | RuntimeMediaAssetState::Conforming
                            | RuntimeMediaAssetState::Rebuilding
                    )
            })
            .count();
        let previewable_asset_count = analysis_ready_asset_count;
        let invalidation_active = invalidated_asset_count > 0;
        let indexing_state = if indexed_asset_count == 0 {
            RuntimeMediaIndexingState::Empty
        } else if self.assets.values().any(|asset| {
            matches!(
                asset.state,
                RuntimeMediaAssetState::Ingesting
                    | RuntimeMediaAssetState::Conforming
                    | RuntimeMediaAssetState::Rebuilding
            )
        }) {
            RuntimeMediaIndexingState::Syncing
        } else if invalidation_active {
            RuntimeMediaIndexingState::Invalidated
        } else {
            RuntimeMediaIndexingState::Ready
        };
        let preview_state = if self.previewing_asset_id.is_some() && previewable_asset_count > 0 {
            RuntimeMediaPreviewState::Previewing
        } else if previewable_asset_count > 0 {
            RuntimeMediaPreviewState::Ready
        } else if invalidation_active {
            RuntimeMediaPreviewState::Invalidated
        } else {
            RuntimeMediaPreviewState::Unavailable
        };
        let last_invalidated_asset = invalidated_assets.last().copied();

        RuntimeMediaServiceSnapshot {
            indexed_asset_count,
            analysis_ready_asset_count,
            waveform_ready_asset_count,
            waveform_pending_asset_count,
            previewable_asset_count,
            invalidated_asset_count,
            invalidation_active,
            indexing_state,
            preview_state,
            previewing_asset_id: self.previewing_asset_id.clone(),
            last_invalidated_asset_id: last_invalidated_asset
                .map(|asset| asset.registration.asset_id.clone()),
            last_invalidation_error: last_invalidated_asset
                .and_then(|asset| asset.last_error.clone()),
            last_preview_error: self.last_preview_error.clone(),
            summary: format!(
                "indexed={} ready={} waveform_ready={} waveform_pending={} previewable={} invalidated={} indexing={:?} preview={:?} previewing={} last_invalidated={} invalidation_error={} preview_error={}",
                indexed_asset_count,
                analysis_ready_asset_count,
                waveform_ready_asset_count,
                waveform_pending_asset_count,
                previewable_asset_count,
                invalidated_asset_count,
                indexing_state,
                preview_state,
                self.previewing_asset_id.as_deref().unwrap_or("none"),
                last_invalidated_asset
                    .map(|asset| asset.registration.asset_id.as_str())
                    .unwrap_or("none"),
                last_invalidated_asset
                    .and_then(|asset| asset.last_error.as_deref())
                    .unwrap_or("none"),
                self.last_preview_error.as_deref().unwrap_or("none"),
            ),
        }
    }
}
