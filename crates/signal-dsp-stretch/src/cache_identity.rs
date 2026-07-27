use crate::{
    OfflineHighQualityPath, StretchBackendTier, StretchOfflineChunkConfig, DEFAULT_ANALYSIS_HOP,
    DEFAULT_WINDOW_SIZE,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Current Signal-owned stretch cache identity schema.
///
/// `v3` adds render geometry, chunk policy, and the crate-owned behavior
/// version. Every `v2` artifact is invalid: it was keyed without those inputs,
/// and its renderer predates the 2026-07-27 defect correction. There is no
/// migration, because a `v2` key cannot describe which render it holds.
pub const STRETCH_CACHE_IDENTITY_SCHEMA_VERSION: &str = "signal-stretch-cache-v3";

/// Version tag for the first-party Signal stretch engine implementation.
pub const SIGNAL_STRETCH_ENGINE_VERSION: &str = "signal-native-stretch-v3";

/// Crate-owned renderer behavior version.
///
/// This is not part of [`StretchCacheIdentityInput`] on purpose. A caller can
/// set any `engine_version` it likes, so a caller-supplied field cannot be
/// trusted to describe renderer behavior. This constant is written into the
/// canonical key by the crate itself.
///
/// Contract `046` requires it to advance in the same change that alters
/// renderer output. It last advanced for `g10.039` render-plane adoption, where
/// the offline artifact path moved to the state-carrying resumable renderer.
pub const SIGNAL_STRETCH_BEHAVIOR_VERSION: &str = "signal-stretch-behavior-2026-07-27-resumable";

impl StretchBackendTier {
    /// Stable key token for cache identity.
    ///
    /// Explicit rather than derived: `Debug` output is not a stability
    /// contract, so a variant rename would silently rekey every artifact.
    pub const fn cache_key_token(self) -> &'static str {
        match self {
            Self::Repitch => "repitch",
            Self::RealtimePreview => "realtime-preview",
            Self::OfflineHighQuality => "offline-high-quality",
        }
    }
}

impl OfflineHighQualityPath {
    /// Stable key token for cache identity.
    pub const fn cache_key_token(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::CompressionShortWindowSelector => "compression-short-window-selector",
            Self::ExpansionShortWindowSelector => "expansion-short-window-selector",
        }
    }
}

/// STFT geometry a render was produced with.
///
/// `OfflineHighQualityStretcher::with_window` is public, so two renders of one
/// source at different geometries are different audio and must not share a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchRenderGeometry {
    /// STFT window size in sample frames.
    pub window_size: usize,
    /// Analysis hop in sample frames, before the overlap coverage law adapts it.
    pub analysis_hop: usize,
}

impl StretchRenderGeometry {
    /// Construct a render geometry.
    pub const fn new(window_size: usize, analysis_hop: usize) -> Self {
        Self {
            window_size,
            analysis_hop,
        }
    }
}

impl Default for StretchRenderGeometry {
    fn default() -> Self {
        Self::new(DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP)
    }
}

/// One point on an output/input stretch-ratio curve.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchRatioPoint {
    /// Timeline sample frame where this ratio becomes active.
    pub timeline_frame: i64,
    /// Output/input duration ratio. `2.0` doubles duration.
    pub ratio: f64,
}

impl StretchRatioPoint {
    /// Construct a ratio curve point.
    pub fn new(timeline_frame: i64, ratio: f64) -> Self {
        Self {
            timeline_frame,
            ratio,
        }
    }
}

/// One point on an independent pitch-shift curve.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchPitchPoint {
    /// Timeline sample frame where this pitch shift becomes active.
    pub timeline_frame: i64,
    /// Pitch shift in semitones.
    pub semitones: f64,
}

impl StretchPitchPoint {
    /// Construct a pitch curve point.
    pub fn new(timeline_frame: i64, semitones: f64) -> Self {
        Self {
            timeline_frame,
            semitones,
        }
    }
}

/// Warp marker anchoring source media to projected timeline samples.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StretchWarpMarker {
    /// Source media sample frame.
    pub source_frame: u64,
    /// Projected timeline sample frame.
    pub timeline_frame: i64,
}

impl StretchWarpMarker {
    /// Construct a warp-marker identity point.
    pub fn new(source_frame: u64, timeline_frame: i64) -> Self {
        Self {
            source_frame,
            timeline_frame,
        }
    }
}

/// Source channel layout used by a cacheable stretch artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchChannelLayout {
    /// Source channel count.
    pub channels: u16,
    /// Source sample rate in hertz.
    pub sample_rate_hz: u32,
}

impl StretchChannelLayout {
    /// Construct a channel-layout identity.
    pub fn new(channels: u16, sample_rate_hz: u32) -> Self {
        Self {
            channels,
            sample_rate_hz,
        }
    }
}

/// Inputs that define one cacheable Signal stretch artifact.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCacheIdentityInput {
    /// Signal stretch engine version.
    pub engine_version: String,
    /// Stretch tier that produced the artifact.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path used for the artifact.
    pub offline_path: OfflineHighQualityPath,
    /// Content hash of the decoded source media identity.
    pub source_content_hash: String,
    /// Source channel layout.
    pub channel_layout: StretchChannelLayout,
    /// Ratio curve sampled from the canonical tick/sample projection.
    pub ratio_curve: Vec<StretchRatioPoint>,
    /// Independent pitch curve sampled from the canonical tick/sample projection.
    pub pitch_curve: Vec<StretchPitchPoint>,
    /// Warp markers included in this artifact.
    pub warp_markers: Vec<StretchWarpMarker>,
    /// Projection epoch for the ADR-001 tick/sample mapping used by this render.
    pub projection_epoch: String,
    /// STFT geometry the render used.
    pub render_geometry: StretchRenderGeometry,
    /// Bounded-memory chunk policy the render used.
    ///
    /// Chunk boundaries move where segment renders restart phase, so two
    /// chunk policies produce different audio from one source. Measured at
    /// correlation `-0.296620` between a single-chunk and an eight-chunk render
    /// of the same identity.
    pub chunk_policy: StretchOfflineChunkConfig,
}

impl StretchCacheIdentityInput {
    /// Construct an input using the current Signal stretch engine version.
    pub fn signal_native(
        tier: StretchBackendTier,
        source_content_hash: impl Into<String>,
        channel_layout: StretchChannelLayout,
        projection_epoch: impl Into<String>,
    ) -> Self {
        Self {
            engine_version: SIGNAL_STRETCH_ENGINE_VERSION.to_string(),
            tier,
            offline_path: OfflineHighQualityPath::Default,
            source_content_hash: source_content_hash.into(),
            channel_layout,
            ratio_curve: Vec::new(),
            pitch_curve: Vec::new(),
            warp_markers: Vec::new(),
            projection_epoch: projection_epoch.into(),
            render_geometry: StretchRenderGeometry::default(),
            chunk_policy: StretchOfflineChunkConfig::default(),
        }
    }

    /// Set the offline high-quality renderer path.
    pub fn with_offline_path(mut self, offline_path: OfflineHighQualityPath) -> Self {
        self.offline_path = offline_path;
        self
    }

    /// Set the STFT geometry the render used.
    pub fn with_render_geometry(mut self, render_geometry: StretchRenderGeometry) -> Self {
        self.render_geometry = render_geometry;
        self
    }

    /// Set the bounded-memory chunk policy the render used.
    pub fn with_chunk_policy(mut self, chunk_policy: StretchOfflineChunkConfig) -> Self {
        self.chunk_policy = chunk_policy;
        self
    }

    /// Set the ratio curve.
    pub fn with_ratio_curve(mut self, ratio_curve: Vec<StretchRatioPoint>) -> Self {
        self.ratio_curve = ratio_curve;
        self
    }

    /// Set the pitch curve.
    pub fn with_pitch_curve(mut self, pitch_curve: Vec<StretchPitchPoint>) -> Self {
        self.pitch_curve = pitch_curve;
        self
    }

    /// Set the warp markers.
    pub fn with_warp_markers(mut self, warp_markers: Vec<StretchWarpMarker>) -> Self {
        self.warp_markers = warp_markers;
        self
    }

    /// Validate and materialize a stable cache identity.
    pub fn identity(&self) -> Result<StretchCacheIdentity, StretchCacheIdentityError> {
        StretchCacheIdentity::from_input(self)
    }
}

/// Stable cache identity for one stretch artifact candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StretchCacheIdentity {
    /// Line-oriented canonical identity string.
    pub canonical_key: String,
    /// Stable FNV-1a hash of `canonical_key`, formatted as lowercase hex.
    pub stable_hash: String,
}

impl StretchCacheIdentity {
    /// Build a deterministic identity from validated stretch artifact inputs.
    pub fn from_input(
        input: &StretchCacheIdentityInput,
    ) -> Result<Self, StretchCacheIdentityError> {
        validate_input(input)?;
        let canonical_key = canonical_key(input);
        let stable_hash = stable_hash_hex(canonical_key.as_bytes());
        Ok(Self {
            canonical_key,
            stable_hash,
        })
    }
}

/// Cache identity validation failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StretchCacheIdentityError {
    /// Engine version was empty.
    EmptyEngineVersion,
    /// Source content hash was empty.
    EmptySourceContentHash,
    /// Projection epoch was empty.
    EmptyProjectionEpoch,
    /// Source channel count was zero.
    InvalidChannelCount,
    /// Source sample rate was zero.
    InvalidSampleRate,
    /// Ratio curve contained a non-finite or non-positive value.
    InvalidRatio,
    /// Pitch curve contained a non-finite value.
    InvalidPitch,
    /// Render geometry had a zero window size or analysis hop.
    InvalidRenderGeometry,
}

fn validate_input(input: &StretchCacheIdentityInput) -> Result<(), StretchCacheIdentityError> {
    if input.engine_version.is_empty() {
        return Err(StretchCacheIdentityError::EmptyEngineVersion);
    }
    if input.source_content_hash.is_empty() {
        return Err(StretchCacheIdentityError::EmptySourceContentHash);
    }
    if input.projection_epoch.is_empty() {
        return Err(StretchCacheIdentityError::EmptyProjectionEpoch);
    }
    if input.channel_layout.channels == 0 {
        return Err(StretchCacheIdentityError::InvalidChannelCount);
    }
    if input.channel_layout.sample_rate_hz == 0 {
        return Err(StretchCacheIdentityError::InvalidSampleRate);
    }
    if input.render_geometry.window_size == 0 || input.render_geometry.analysis_hop == 0 {
        return Err(StretchCacheIdentityError::InvalidRenderGeometry);
    }
    if input
        .ratio_curve
        .iter()
        .any(|point| !point.ratio.is_finite() || point.ratio <= 0.0)
    {
        return Err(StretchCacheIdentityError::InvalidRatio);
    }
    if input
        .pitch_curve
        .iter()
        .any(|point| !point.semitones.is_finite())
    {
        return Err(StretchCacheIdentityError::InvalidPitch);
    }
    Ok(())
}

fn canonical_key(input: &StretchCacheIdentityInput) -> String {
    let mut key = String::new();
    push_field(
        &mut key,
        "schema",
        STRETCH_CACHE_IDENTITY_SCHEMA_VERSION.to_string(),
    );
    // Crate-owned, never caller-supplied: a caller can set any engine_version,
    // so renderer behavior needs a field the caller cannot get wrong.
    push_field(
        &mut key,
        "behavior",
        SIGNAL_STRETCH_BEHAVIOR_VERSION.to_string(),
    );
    push_field(&mut key, "engine", input.engine_version.clone());
    push_field(&mut key, "tier", input.tier.cache_key_token().to_string());
    push_field(
        &mut key,
        "offline_path",
        input.offline_path.cache_key_token().to_string(),
    );
    push_field(
        &mut key,
        "window_size",
        input.render_geometry.window_size.to_string(),
    );
    push_field(
        &mut key,
        "analysis_hop",
        input.render_geometry.analysis_hop.to_string(),
    );
    push_field(
        &mut key,
        "chunk_max_source_frames",
        input.chunk_policy.max_source_frames.to_string(),
    );
    push_field(
        &mut key,
        "chunk_overlap_frames",
        input.chunk_policy.overlap_frames.to_string(),
    );
    push_field(
        &mut key,
        "source_content_hash",
        input.source_content_hash.clone(),
    );
    push_field(
        &mut key,
        "channels",
        input.channel_layout.channels.to_string(),
    );
    push_field(
        &mut key,
        "sample_rate_hz",
        input.channel_layout.sample_rate_hz.to_string(),
    );
    push_field(&mut key, "projection_epoch", input.projection_epoch.clone());
    push_field(
        &mut key,
        "ratio_curve",
        input
            .ratio_curve
            .iter()
            .map(|point| {
                format!(
                    "{}:{}",
                    point.timeline_frame,
                    canonical_f64_bits(point.ratio)
                )
            })
            .collect::<Vec<_>>()
            .join(","),
    );
    push_field(
        &mut key,
        "pitch_curve",
        input
            .pitch_curve
            .iter()
            .map(|point| {
                format!(
                    "{}:{}",
                    point.timeline_frame,
                    canonical_f64_bits(point.semitones)
                )
            })
            .collect::<Vec<_>>()
            .join(","),
    );
    push_field(
        &mut key,
        "warp_markers",
        input
            .warp_markers
            .iter()
            .map(|marker| format!("{}:{}", marker.source_frame, marker.timeline_frame))
            .collect::<Vec<_>>()
            .join(","),
    );
    key
}

fn push_field(key: &mut String, name: &str, value: String) {
    key.push_str(name);
    key.push('=');
    key.push_str(&value);
    key.push('\n');
}

fn canonical_f64_bits(value: f64) -> String {
    let normalized = if value == 0.0 { 0.0 } else { value };
    format!("{:016x}", normalized.to_bits())
}

fn stable_hash_hex(bytes: &[u8]) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> StretchCacheIdentityInput {
        StretchCacheIdentityInput::signal_native(
            StretchBackendTier::OfflineHighQuality,
            "sha256:source-a",
            StretchChannelLayout::new(2, 48_000),
            "projection-7",
        )
        .with_ratio_curve(vec![
            StretchRatioPoint::new(0, 1.0),
            StretchRatioPoint::new(48_000, 1.5),
        ])
        .with_pitch_curve(vec![
            StretchPitchPoint::new(0, 0.0),
            StretchPitchPoint::new(96_000, -2.0),
        ])
        .with_warp_markers(vec![
            StretchWarpMarker::new(0, 0),
            StretchWarpMarker::new(24_000, 48_000),
        ])
    }

    #[test]
    fn cache_identity_is_deterministic_for_equal_inputs() {
        let left = base_input().identity().expect("valid identity");
        let right = base_input().identity().expect("valid identity");

        assert_eq!(left, right);
        assert!(left
            .canonical_key
            .contains("schema=signal-stretch-cache-v3"));
        assert!(left.canonical_key.contains("tier=offline-high-quality"));
        assert!(left.canonical_key.contains("offline_path=default"));
        assert_eq!(left.stable_hash.len(), 16);
    }

    #[test]
    fn cache_identity_changes_for_projection_curve_or_offline_path_changes() {
        let base = base_input().identity().expect("valid identity");
        let changed_projection = StretchCacheIdentityInput {
            projection_epoch: "projection-8".to_string(),
            ..base_input()
        }
        .identity()
        .expect("valid identity");
        let changed_ratio = base_input()
            .with_ratio_curve(vec![
                StretchRatioPoint::new(0, 1.0),
                StretchRatioPoint::new(48_000, 1.25),
            ])
            .identity()
            .expect("valid identity");
        let changed_compression_path = base_input()
            .with_offline_path(OfflineHighQualityPath::CompressionShortWindowSelector)
            .identity()
            .expect("valid identity");
        let changed_expansion_path = base_input()
            .with_offline_path(OfflineHighQualityPath::ExpansionShortWindowSelector)
            .identity()
            .expect("valid identity");

        assert_ne!(base.stable_hash, changed_projection.stable_hash);
        assert_ne!(base.stable_hash, changed_ratio.stable_hash);
        assert_ne!(base.stable_hash, changed_compression_path.stable_hash);
        assert_ne!(base.stable_hash, changed_expansion_path.stable_hash);
        assert_ne!(
            changed_compression_path.stable_hash,
            changed_expansion_path.stable_hash
        );
        assert!(changed_compression_path
            .canonical_key
            .contains("offline_path=compression-short-window-selector"));
        assert!(changed_expansion_path
            .canonical_key
            .contains("offline_path=expansion-short-window-selector"));
    }

    #[test]
    fn cache_identity_covers_render_geometry() {
        let base = base_input().identity().expect("valid identity");
        let shorter_window = base_input()
            .with_render_geometry(StretchRenderGeometry::new(1_024, 256))
            .identity()
            .expect("valid identity");
        let same_window_finer_hop = base_input()
            .with_render_geometry(StretchRenderGeometry::new(2_048, 256))
            .identity()
            .expect("valid identity");

        assert_ne!(base.stable_hash, shorter_window.stable_hash);
        assert_ne!(base.stable_hash, same_window_finer_hop.stable_hash);
        assert_ne!(
            shorter_window.stable_hash,
            same_window_finer_hop.stable_hash
        );
        assert!(base.canonical_key.contains("window_size=2048"));
        assert!(base.canonical_key.contains("analysis_hop=512"));
    }

    #[test]
    fn cache_identity_covers_chunk_policy() {
        let base = base_input().identity().expect("valid identity");
        let small_chunks = base_input()
            .with_chunk_policy(StretchOfflineChunkConfig::new(12_000, 2_048))
            .identity()
            .expect("valid identity");
        let same_chunks_more_overlap = base_input()
            .with_chunk_policy(StretchOfflineChunkConfig::new(12_000, 4_096))
            .identity()
            .expect("valid identity");

        assert_ne!(base.stable_hash, small_chunks.stable_hash);
        assert_ne!(
            small_chunks.stable_hash,
            same_chunks_more_overlap.stable_hash
        );
    }

    /// Key tokens are explicit strings, not `Debug` output. A variant rename
    /// must not silently rekey every artifact, so these literals are the
    /// contract and this owner fails if anyone changes them.
    #[test]
    fn cache_identity_uses_stable_key_tokens() {
        assert_eq!(StretchBackendTier::Repitch.cache_key_token(), "repitch");
        assert_eq!(
            StretchBackendTier::RealtimePreview.cache_key_token(),
            "realtime-preview"
        );
        assert_eq!(
            StretchBackendTier::OfflineHighQuality.cache_key_token(),
            "offline-high-quality"
        );
        assert_eq!(OfflineHighQualityPath::Default.cache_key_token(), "default");
        assert_eq!(
            OfflineHighQualityPath::CompressionShortWindowSelector.cache_key_token(),
            "compression-short-window-selector"
        );
        assert_eq!(
            OfflineHighQualityPath::ExpansionShortWindowSelector.cache_key_token(),
            "expansion-short-window-selector"
        );

        let identity = base_input().identity().expect("valid identity");
        assert!(identity.canonical_key.contains("tier=offline-high-quality"));
        assert!(identity.canonical_key.contains("offline_path=default"));
        assert!(!identity.canonical_key.contains("OfflineHighQuality"));
    }

    /// The behavior version is crate-owned. A caller can set any
    /// `engine_version`, so renderer behavior must not depend on a field the
    /// caller controls.
    #[test]
    fn cache_identity_carries_a_crate_owned_behavior_version() {
        let identity = base_input().identity().expect("valid identity");
        assert!(identity
            .canonical_key
            .contains(&format!("behavior={SIGNAL_STRETCH_BEHAVIOR_VERSION}")));

        let caller_overridden = StretchCacheIdentityInput {
            engine_version: "someone-elses-engine".to_string(),
            ..base_input()
        }
        .identity()
        .expect("valid identity");
        assert!(caller_overridden
            .canonical_key
            .contains(&format!("behavior={SIGNAL_STRETCH_BEHAVIOR_VERSION}")));
        assert_ne!(identity.stable_hash, caller_overridden.stable_hash);
    }

    #[test]
    fn cache_identity_schema_and_engine_are_v3() {
        assert_eq!(
            STRETCH_CACHE_IDENTITY_SCHEMA_VERSION,
            "signal-stretch-cache-v3"
        );
        assert_eq!(SIGNAL_STRETCH_ENGINE_VERSION, "signal-native-stretch-v3");

        // A v2 artifact cannot collide with a v3 one: both the schema line and
        // the engine line differ, and v2 had no geometry, chunk, or behavior
        // fields at all.
        let v3 = base_input().identity().expect("valid identity");
        assert!(v3.canonical_key.contains("schema=signal-stretch-cache-v3"));
        assert!(v3.canonical_key.contains("engine=signal-native-stretch-v3"));
    }

    #[test]
    fn cache_identity_rejects_invalid_inputs() {
        let invalid_ratio = base_input()
            .with_ratio_curve(vec![StretchRatioPoint::new(0, f64::NAN)])
            .identity();
        let invalid_pitch = base_input()
            .with_pitch_curve(vec![StretchPitchPoint::new(0, f64::INFINITY)])
            .identity();
        let invalid_layout = StretchCacheIdentityInput::signal_native(
            StretchBackendTier::OfflineHighQuality,
            "sha256:source-a",
            StretchChannelLayout::new(0, 48_000),
            "projection-7",
        )
        .identity();

        assert_eq!(invalid_ratio, Err(StretchCacheIdentityError::InvalidRatio));
        assert_eq!(invalid_pitch, Err(StretchCacheIdentityError::InvalidPitch));
        assert_eq!(
            invalid_layout,
            Err(StretchCacheIdentityError::InvalidChannelCount)
        );
    }

    #[test]
    fn cache_identity_normalizes_negative_zero_pitch() {
        let positive_zero = base_input()
            .with_pitch_curve(vec![StretchPitchPoint::new(0, 0.0)])
            .identity()
            .expect("valid identity");
        let negative_zero = base_input()
            .with_pitch_curve(vec![StretchPitchPoint::new(0, -0.0)])
            .identity()
            .expect("valid identity");

        assert_eq!(positive_zero, negative_zero);
    }
}
