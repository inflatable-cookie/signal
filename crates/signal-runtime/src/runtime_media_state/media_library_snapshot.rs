use super::*;

impl RuntimeMediaPipelineStateModel {
    pub(crate) fn library_service_snapshot(&self) -> RuntimeMediaLibraryServiceSnapshot {
        let descriptors = self
            .assets
            .values()
            .map(|asset| self.asset_library_descriptor(asset))
            .collect::<Vec<_>>();
        let ready_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.metadata_state == RuntimeMediaAnalysisDescriptorState::Ready
            })
            .count();
        let pending_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.metadata_state == RuntimeMediaAnalysisDescriptorState::Pending
            })
            .count();
        let invalidated_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.metadata_state == RuntimeMediaAnalysisDescriptorState::Invalidated
            })
            .count();
        let unavailable_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.metadata_state == RuntimeMediaAnalysisDescriptorState::Unavailable
            })
            .count();
        let loudness_ready_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.loudness_state == RuntimeMediaAnalysisFamilyState::Ready
            })
            .count();
        let character_ready_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.character_state == RuntimeMediaAnalysisFamilyState::Ready
            })
            .count();
        let rhythm_deferred_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.rhythm_state == RuntimeMediaAnalysisFamilyState::Deferred
            })
            .count();
        let tonal_deferred_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.tonal_state == RuntimeMediaAnalysisFamilyState::Deferred
            })
            .count();
        let embedding_deferred_descriptor_count = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.embedding_state == RuntimeMediaAnalysisFamilyState::Deferred
            })
            .count();

        RuntimeMediaLibraryServiceSnapshot {
            indexed_asset_count: descriptors.len(),
            ready_descriptor_count,
            pending_descriptor_count,
            invalidated_descriptor_count,
            unavailable_descriptor_count,
            loudness_ready_descriptor_count,
            character_ready_descriptor_count,
            rhythm_deferred_descriptor_count,
            tonal_deferred_descriptor_count,
            embedding_deferred_descriptor_count,
            descriptors,
        }
    }

    fn asset_library_descriptor(
        &self,
        asset: &RuntimeMediaPipelineAsset,
    ) -> RuntimeMediaLibraryAssetDescriptor {
        RuntimeMediaLibraryAssetDescriptor {
            asset_id: asset.registration.asset_id.clone(),
            content_hash: asset.registration.content_hash.clone(),
            file_name: asset.registration.file_name.clone(),
            asset_state: Some(asset.state),
            metadata_state: asset.analysis.descriptor_state,
            loudness_state: media_family_state(
                asset.analysis.descriptor_state,
                asset.analysis.loudness.is_some(),
            ),
            character_state: media_family_state(
                asset.analysis.descriptor_state,
                asset.analysis.character.is_some(),
            ),
            rhythm_state: RuntimeMediaAnalysisFamilyState::Deferred,
            tonal_state: RuntimeMediaAnalysisFamilyState::Deferred,
            embedding_state: RuntimeMediaAnalysisFamilyState::Deferred,
            loudness: asset.analysis.loudness.clone(),
            character: asset.analysis.character.clone(),
            last_error: asset
                .analysis
                .last_error
                .clone()
                .or_else(|| asset.last_error.clone()),
        }
    }
}
