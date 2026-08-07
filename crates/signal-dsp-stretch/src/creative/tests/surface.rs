use super::*;

/// Contract `085`, 2026-07-27: creative renders are uncacheable. The
/// surface must not grow a key, receipt, or artifact vocabulary without
/// the enumeration Contract `046` requires of the transparent identity.
#[test]
fn creative_surface_carries_no_cache_or_artifact_vocabulary() {
    // Scan the production module only: this owner names the forbidden
    // identifiers itself in the sibling test module.
    let source = include_str!("../mod.rs");
    for forbidden in [
        "CacheIdentity",
        "cache_key",
        "canonical_key",
        "stable_hash",
        "PromotionReceipt",
        "OfflineStretchArtifact",
        "StretchOfflineChunk",
    ] {
        assert!(
            !source.contains(forbidden),
            "creative surface mentions `{forbidden}`; Contract `085` declares \
             creative renders uncacheable, so a cache surface needs a contract \
             change and a named consumer first"
        );
    }
}

/// No stretch tier describes a creative render, so no cache identity can
/// name one. Adding a tier variant breaks this match and this owner.
#[test]
fn no_stretch_tier_describes_a_creative_render() {
    fn is_transparent_tier(tier: StretchBackendTier) -> bool {
        match tier {
            StretchBackendTier::Repitch
            | StretchBackendTier::RealtimePreview
            | StretchBackendTier::OfflineHighQuality => true,
        }
    }

    for tier in [
        StretchBackendTier::Repitch,
        StretchBackendTier::RealtimePreview,
        StretchBackendTier::OfflineHighQuality,
    ] {
        assert!(is_transparent_tier(tier));
        let token = tier.cache_key_token();
        for creative_word in ["creative", "dream", "cyclic"] {
            assert!(
                !token.contains(creative_word),
                "tier token `{token}` names a creative render"
            );
        }
    }
}
