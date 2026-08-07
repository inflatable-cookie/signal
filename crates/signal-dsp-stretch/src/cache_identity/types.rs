use crate::{
    OfflineHighQualityPath, StretchBackendTier, DEFAULT_ANALYSIS_HOP, DEFAULT_WINDOW_SIZE,
};

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
/// This is not part of [`StretchCacheIdentityInput`](crate::StretchCacheIdentityInput) on purpose. A caller can
/// set any `engine_version` it likes, so a caller-supplied field cannot be
/// trusted to describe renderer behavior. This constant is written into the
/// canonical key by the crate itself.
///
/// Contract `046` requires it to advance in the same change that alters
/// renderer output. It last advanced for the `g10.036` defect correction, which
/// changed output at every ratio above `3.0` and for every dynamic-ratio curve.
pub const SIGNAL_STRETCH_BEHAVIOR_VERSION: &str =
    "signal-stretch-behavior-2026-08-05-pitch-resumable";

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
