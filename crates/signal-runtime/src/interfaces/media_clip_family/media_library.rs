/// Processing state of a media asset in the pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMediaAssetState {
    /// Asset is being ingested from disk.
    Ingesting,
    /// Asset is being transcoded or conformed to the project sample rate.
    Conforming,
    /// Asset is ready for use.
    Ready,
    /// Asset is in an unrecoverable error state.
    Invalid,
    /// Asset is being rebuilt after invalidation.
    Rebuilding,
}

/// Registration record for a media asset (source file metadata).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMediaAssetRegistration {
    /// Unique identifier for this asset.
    pub asset_id: String,
    /// Content hash of the source file.
    pub content_hash: String,
    /// Absolute path to the source file.
    pub source_path: String,
    /// File name of the source file.
    pub file_name: String,
    /// Size of the source file in bytes.
    pub byte_size: u64,
    /// Sample rate of the source audio in Hz.
    pub sample_rate_hz: u32,
    /// Number of audio channels in the source file.
    pub channel_count: u16,
    /// Duration of the source audio in samples.
    pub duration_samples: u64,
    /// Number of waveform preview bins available for this asset.
    pub waveform_bin_count: usize,
}

/// Full snapshot of a media asset including state, cache path, and rebuild count.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaAssetSnapshot {
    /// Unique identifier for this asset.
    pub asset_id: String,
    /// Content hash of the source file.
    pub content_hash: String,
    /// Absolute path to the source file.
    pub source_path: String,
    /// File name of the source file.
    pub file_name: String,
    /// Size of the source file in bytes.
    pub byte_size: u64,
    /// Sample rate of the source audio in Hz.
    pub sample_rate_hz: u32,
    /// Number of audio channels in the source file.
    pub channel_count: u16,
    /// Duration of the source audio in samples.
    pub duration_samples: u64,
    /// Number of waveform preview bins available for this asset.
    pub waveform_bin_count: usize,
    /// Current processing state of this asset.
    pub state: Option<RuntimeMediaAssetState>,
    /// Absolute path to the cached/conformed file, if available.
    pub cache_path: Option<String>,
    /// Size of the cache file in bytes, if available.
    pub cache_byte_size: Option<u64>,
    /// Number of times this asset has been rebuilt.
    pub rebuild_count: u32,
    /// Last error message if the asset pipeline encountered a problem, if any.
    pub last_error: Option<String>,
}

/// Aggregate snapshot of all assets in the media pipeline.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaPipelineSnapshot {
    /// Root path of the media asset cache directory.
    pub cache_root_path: String,
    /// Total number of assets registered in the pipeline.
    pub asset_count: usize,
    /// Number of assets in the ready state.
    pub ready_asset_count: usize,
    /// Number of assets in the invalid state.
    pub invalid_asset_count: usize,
    /// Number of assets currently ingesting.
    pub ingesting_asset_count: usize,
    /// Number of assets currently conforming.
    pub conforming_asset_count: usize,
    /// Number of assets currently rebuilding.
    pub rebuilding_asset_count: usize,
    /// Per-asset snapshots.
    pub assets: Vec<RuntimeMediaAssetSnapshot>,
}

/// State of the media asset index.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaIndexingState {
    #[default]
    /// No assets indexed.
    Empty,
    /// Index is being updated.
    Syncing,
    /// Index is up to date and ready.
    Ready,
    /// Index has been invalidated and needs a refresh.
    Invalidated,
}

/// State of the media preview/audition path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaPreviewState {
    #[default]
    /// Preview path is not available.
    Unavailable,
    /// Preview path is ready to audition an asset.
    Ready,
    /// An asset is currently being auditioned.
    Previewing,
    /// Preview path has been invalidated.
    Invalidated,
}

/// Aggregate snapshot of the media service: indexed assets, invalidation
/// state, indexing and preview states.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMediaServiceSnapshot {
    /// Number of assets currently indexed.
    pub indexed_asset_count: usize,
    /// Number of assets with analysis data ready.
    pub analysis_ready_asset_count: usize,
    /// Number of assets with waveform preview data ready.
    pub waveform_ready_asset_count: usize,
    /// Number of assets with waveform preview generation pending.
    pub waveform_pending_asset_count: usize,
    /// Number of assets available for preview/audition.
    pub previewable_asset_count: usize,
    /// Number of assets currently in an invalidated state.
    pub invalidated_asset_count: usize,
    /// Whether an invalidation sweep is actively running.
    pub invalidation_active: bool,
    /// Current state of the asset index.
    pub indexing_state: RuntimeMediaIndexingState,
    /// Current state of the preview/audition path.
    pub preview_state: RuntimeMediaPreviewState,
    /// ID of the asset currently being previewed, if any.
    pub previewing_asset_id: Option<String>,
    /// ID of the most recently invalidated asset, if any.
    pub last_invalidated_asset_id: Option<String>,
    /// Last error encountered during asset invalidation, if any.
    pub last_invalidation_error: Option<String>,
    /// Last error encountered during preview/audition, if any.
    pub last_preview_error: Option<String>,
}

/// State of a media asset's analysis descriptor (metadata).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAnalysisDescriptorState {
    #[default]
    /// No descriptor exists for this asset.
    Missing,
    /// Descriptor analysis is in progress.
    Pending,
    /// Descriptor is complete and ready.
    Ready,
    /// Descriptor has been invalidated.
    Invalidated,
    /// Descriptor analysis is not supported for this asset type.
    Unavailable,
}

/// State of a specific analysis family (loudness, character, etc.) for an asset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAnalysisFamilyState {
    #[default]
    /// Analysis for this family has not yet been run.
    Deferred,
    /// Analysis for this family is complete and ready.
    Ready,
    /// Analysis for this family is not available for this asset.
    Unavailable,
}

/// Loudness analysis descriptor for a media asset (LUFS, true peak, etc.).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMediaLoudnessDescriptor {
    /// Integrated loudness in LUFS.
    pub integrated_lufs: f32,
    /// Loudness range in LU.
    pub loudness_range_lu: f32,
    /// True peak level in dBTP.
    pub true_peak_dbtp: f32,
    /// Offset from a loudness normalization target in LU.
    pub target_offset_lu: f32,
    /// Peak-to-loudness ratio in LU.
    pub peak_to_loudness_lu: f32,
    /// Confidence score for the loudness measurement (0–1).
    pub confidence: f32,
}

/// Spectral and dynamic character descriptor for a media asset.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMediaCharacterDescriptor {
    /// Spectral centroid frequency in Hz.
    pub centroid_hz: f32,
    /// 95th-percentile spectral rolloff frequency in Hz.
    pub rolloff_95_hz: f32,
    /// Spectral flatness measure (0–1).
    pub flatness: f32,
    /// Spectral contrast in dB.
    pub contrast_db: f32,
    /// Density of note onsets per second.
    pub onset_density: f32,
    /// Density of transient events per second.
    pub transient_density: f32,
    /// Ratio of sustained energy to total energy (0–1).
    pub sustain_ratio: f32,
    /// Root-mean-square energy level.
    pub rms_energy: f32,
    /// Dynamic range estimate in dB.
    pub dynamic_range: f32,
    /// Confidence score for the character measurement (0–1).
    pub confidence: f32,
}

/// Full descriptor for a media library asset: all analysis family states and
/// available loudness/character data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMediaLibraryAssetDescriptor {
    /// Unique identifier for this asset.
    pub asset_id: String,
    /// Content hash of the source file.
    pub content_hash: String,
    /// File name of the source file.
    pub file_name: String,
    /// Current processing state of the underlying media asset, if known.
    pub asset_state: Option<RuntimeMediaAssetState>,
    /// State of the overall analysis metadata descriptor.
    pub metadata_state: RuntimeMediaAnalysisDescriptorState,
    /// State of the loudness analysis family.
    pub loudness_state: RuntimeMediaAnalysisFamilyState,
    /// State of the spectral character analysis family.
    pub character_state: RuntimeMediaAnalysisFamilyState,
    /// State of the rhythm analysis family.
    pub rhythm_state: RuntimeMediaAnalysisFamilyState,
    /// State of the tonal analysis family.
    pub tonal_state: RuntimeMediaAnalysisFamilyState,
    /// State of the embedding/feature-vector analysis family.
    pub embedding_state: RuntimeMediaAnalysisFamilyState,
    /// Loudness descriptor, if analysis is complete.
    pub loudness: Option<RuntimeMediaLoudnessDescriptor>,
    /// Character descriptor, if analysis is complete.
    pub character: Option<RuntimeMediaCharacterDescriptor>,
    /// Last error message if library analysis encountered a problem, if any.
    pub last_error: Option<String>,
}

/// Aggregate snapshot of the media library service: descriptor counts by
/// analysis family and all asset descriptors.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeMediaLibraryServiceSnapshot {
    /// Total number of indexed assets tracked by the library service.
    pub indexed_asset_count: usize,
    /// Number of assets with a ready analysis descriptor.
    pub ready_descriptor_count: usize,
    /// Number of assets with a pending analysis descriptor.
    pub pending_descriptor_count: usize,
    /// Number of assets with an invalidated analysis descriptor.
    pub invalidated_descriptor_count: usize,
    /// Number of assets whose analysis descriptor is unavailable.
    pub unavailable_descriptor_count: usize,
    /// Number of assets with a ready loudness descriptor.
    pub loudness_ready_descriptor_count: usize,
    /// Number of assets with a ready character descriptor.
    pub character_ready_descriptor_count: usize,
    /// Number of assets with rhythm analysis deferred.
    pub rhythm_deferred_descriptor_count: usize,
    /// Number of assets with tonal analysis deferred.
    pub tonal_deferred_descriptor_count: usize,
    /// Number of assets with embedding analysis deferred.
    pub embedding_deferred_descriptor_count: usize,
    /// Per-asset library descriptors.
    pub descriptors: Vec<RuntimeMediaLibraryAssetDescriptor>,
}
