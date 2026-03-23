#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMediaAssetState {
    Ingesting,
    Conforming,
    Ready,
    Invalid,
    Rebuilding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMediaAssetRegistration {
    pub asset_id: String,
    pub content_hash: String,
    pub source_path: String,
    pub file_name: String,
    pub byte_size: u64,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub duration_samples: u64,
    pub waveform_bin_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaAssetSnapshot {
    pub asset_id: String,
    pub content_hash: String,
    pub source_path: String,
    pub file_name: String,
    pub byte_size: u64,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub duration_samples: u64,
    pub waveform_bin_count: usize,
    pub state: Option<RuntimeMediaAssetState>,
    pub cache_path: Option<String>,
    pub cache_byte_size: Option<u64>,
    pub rebuild_count: u32,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaPipelineSnapshot {
    pub cache_root_path: String,
    pub asset_count: usize,
    pub ready_asset_count: usize,
    pub invalid_asset_count: usize,
    pub ingesting_asset_count: usize,
    pub conforming_asset_count: usize,
    pub rebuilding_asset_count: usize,
    pub assets: Vec<RuntimeMediaAssetSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaIndexingState {
    #[default]
    Empty,
    Syncing,
    Ready,
    Invalidated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaPreviewState {
    #[default]
    Unavailable,
    Ready,
    Previewing,
    Invalidated,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaServiceSnapshot {
    pub indexed_asset_count: usize,
    pub analysis_ready_asset_count: usize,
    pub waveform_ready_asset_count: usize,
    pub waveform_pending_asset_count: usize,
    pub previewable_asset_count: usize,
    pub invalidated_asset_count: usize,
    pub invalidation_active: bool,
    pub indexing_state: RuntimeMediaIndexingState,
    pub preview_state: RuntimeMediaPreviewState,
    pub previewing_asset_id: Option<String>,
    pub last_invalidated_asset_id: Option<String>,
    pub last_invalidation_error: Option<String>,
    pub last_preview_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAnalysisDescriptorState {
    #[default]
    Missing,
    Pending,
    Ready,
    Invalidated,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAnalysisFamilyState {
    #[default]
    Deferred,
    Ready,
    Unavailable,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMediaLoudnessDescriptor {
    pub integrated_lufs: f32,
    pub loudness_range_lu: f32,
    pub true_peak_dbtp: f32,
    pub target_offset_lu: f32,
    pub peak_to_loudness_lu: f32,
    pub confidence: f32,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMediaCharacterDescriptor {
    pub centroid_hz: f32,
    pub rolloff_95_hz: f32,
    pub flatness: f32,
    pub contrast_db: f32,
    pub onset_density: f32,
    pub transient_density: f32,
    pub sustain_ratio: f32,
    pub rms_energy: f32,
    pub dynamic_range: f32,
    pub confidence: f32,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMediaLibraryAssetDescriptor {
    pub asset_id: String,
    pub content_hash: String,
    pub file_name: String,
    pub asset_state: Option<RuntimeMediaAssetState>,
    pub metadata_state: RuntimeMediaAnalysisDescriptorState,
    pub loudness_state: RuntimeMediaAnalysisFamilyState,
    pub character_state: RuntimeMediaAnalysisFamilyState,
    pub rhythm_state: RuntimeMediaAnalysisFamilyState,
    pub tonal_state: RuntimeMediaAnalysisFamilyState,
    pub embedding_state: RuntimeMediaAnalysisFamilyState,
    pub loudness: Option<RuntimeMediaLoudnessDescriptor>,
    pub character: Option<RuntimeMediaCharacterDescriptor>,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMediaLibraryServiceSnapshot {
    pub indexed_asset_count: usize,
    pub ready_descriptor_count: usize,
    pub pending_descriptor_count: usize,
    pub invalidated_descriptor_count: usize,
    pub unavailable_descriptor_count: usize,
    pub loudness_ready_descriptor_count: usize,
    pub character_ready_descriptor_count: usize,
    pub rhythm_deferred_descriptor_count: usize,
    pub tonal_deferred_descriptor_count: usize,
    pub embedding_deferred_descriptor_count: usize,
    pub descriptors: Vec<RuntimeMediaLibraryAssetDescriptor>,
    pub summary: String,
}
