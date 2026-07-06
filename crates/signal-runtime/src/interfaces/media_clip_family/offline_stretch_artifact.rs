use signal_dsp_stretch::{StretchBackendTier, StretchCacheIdentityInput};

/// Offline destination that may consume a stretch artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineStretchArtifactScope {
    /// Final exported render output.
    Export,
    /// Frozen track or clip output.
    Freeze,
    /// Internal post-warp render cache reuse.
    RenderCache,
}

/// Runtime-owned readiness for an offline stretch artifact plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineStretchArtifactReadiness {
    /// The plan has a stable identity, but the tier is not implemented yet.
    AwaitingImplementation,
    /// The tier exists, but corpus evidence has not accepted product-facing use.
    AwaitingCorpusEvidence,
    /// The artifact may feed product-facing render/export/freeze consumers.
    Ready,
    /// The plan could not be validated.
    Invalid,
}

/// Registration for a runtime-observed offline stretch artifact plan.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineStretchArtifactPlanRegistration {
    /// Stable plan identity supplied by the caller.
    pub plan_id: String,
    /// Clip this artifact belongs to, if known.
    pub clip_id: Option<String>,
    /// Media asset this artifact belongs to, if known.
    pub media_asset_id: Option<String>,
    /// Consumer scope for this artifact.
    pub scope: RuntimeOfflineStretchArtifactScope,
    /// Cache identity input shared with render/export planning.
    pub identity_input: StretchCacheIdentityInput,
    /// Whether corpus evidence has accepted product-facing use.
    pub corpus_evidence_accepted: bool,
}

/// Runtime-owned observation receipt for one offline stretch artifact plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineStretchArtifactPlanSnapshot {
    /// Stable plan identity supplied by the caller.
    pub plan_id: String,
    /// Clip this artifact belongs to, if known.
    pub clip_id: Option<String>,
    /// Media asset this artifact belongs to, if known.
    pub media_asset_id: Option<String>,
    /// Consumer scope for this artifact.
    pub scope: RuntimeOfflineStretchArtifactScope,
    /// Signal stretch tier named by the identity input.
    pub tier: StretchBackendTier,
    /// Stable cache identity hash, if validation succeeded.
    pub cache_identity_hash: Option<String>,
    /// Canonical cache identity key, if validation succeeded.
    pub cache_identity_key: Option<String>,
    /// Current runtime-owned readiness.
    pub readiness: RuntimeOfflineStretchArtifactReadiness,
    /// Whether the artifact is allowed to feed product-facing output.
    pub product_facing_allowed: bool,
    /// Validation or promotion-blocking reason.
    pub last_error: Option<String>,
}

/// Aggregate runtime-owned offline stretch artifact plan receipts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineStretchArtifactPlanSnapshotSet {
    /// Total number of observed plans.
    pub plan_count: usize,
    /// Number of plans that may feed product-facing output.
    pub ready_plan_count: usize,
    /// Number of plans waiting for implementation.
    pub awaiting_implementation_count: usize,
    /// Number of plans waiting for corpus acceptance.
    pub awaiting_corpus_evidence_count: usize,
    /// Number of invalid plans.
    pub invalid_plan_count: usize,
    /// Per-plan receipts.
    pub plans: Vec<RuntimeOfflineStretchArtifactPlanSnapshot>,
}
