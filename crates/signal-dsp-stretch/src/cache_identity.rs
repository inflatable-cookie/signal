use crate::{OfflineHighQualityPath, StretchBackendTier};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Current Signal-owned stretch cache identity schema.
pub const STRETCH_CACHE_IDENTITY_SCHEMA_VERSION: &str = "signal-stretch-cache-v2";

/// Version tag for the first-party Signal stretch engine implementation.
pub const SIGNAL_STRETCH_ENGINE_VERSION: &str = "signal-native-stretch-v2";

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
        }
    }

    /// Set the offline high-quality renderer path.
    pub fn with_offline_path(mut self, offline_path: OfflineHighQualityPath) -> Self {
        self.offline_path = offline_path;
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
    push_field(&mut key, "engine", input.engine_version.clone());
    push_field(&mut key, "tier", format!("{:?}", input.tier));
    push_field(
        &mut key,
        "offline_path",
        format!("{:?}", input.offline_path),
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
            .contains("schema=signal-stretch-cache-v2"));
        assert!(left.canonical_key.contains("tier=OfflineHighQuality"));
        assert!(left.canonical_key.contains("offline_path=Default"));
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
            .contains("offline_path=CompressionShortWindowSelector"));
        assert!(changed_expansion_path
            .canonical_key
            .contains("offline_path=ExpansionShortWindowSelector"));
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
