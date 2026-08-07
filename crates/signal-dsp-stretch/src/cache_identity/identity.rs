use super::input::StretchCacheIdentityInput;
use super::types::{SIGNAL_STRETCH_BEHAVIOR_VERSION, STRETCH_CACHE_IDENTITY_SCHEMA_VERSION};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

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
