use crate::{OfflineHighQualityPath, StretchBackendTier, StretchOfflineChunkConfig};

use super::{
    StretchCacheIdentityError, StretchCacheIdentityInput, StretchChannelLayout, StretchPitchPoint,
    StretchRatioPoint, StretchRenderGeometry, StretchWarpMarker, SIGNAL_STRETCH_BEHAVIOR_VERSION,
    SIGNAL_STRETCH_ENGINE_VERSION, STRETCH_CACHE_IDENTITY_SCHEMA_VERSION,
};

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
