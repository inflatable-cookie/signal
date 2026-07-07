use signal_dsp_stretch::{
    OfflineHighQualityPath, StretchBackendTier, StretchCacheIdentityInput, StretchPromotionReceipt,
    StretchPromotionStatus,
};

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
    /// The tier exists, but corpus evidence or prototype promotion has not
    /// accepted product-facing use.
    AwaitingCorpusEvidence,
    /// The artifact may feed product-facing render/export/freeze consumers.
    Ready,
    /// The plan could not be validated.
    Invalid,
}

/// Runtime-owned render-cache decision kind for offline stretch artifacts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineStretchArtifactCacheDecisionKind {
    /// A matching render-cache identity was reused.
    Hit,
    /// A new render-cache artifact was written.
    Written,
    /// A retained render-cache artifact was invalidated.
    Invalidated,
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
    /// Promotion evidence associated with this artifact plan.
    pub promotion_receipt: StretchPromotionReceipt,
}

/// Registration for a runtime-observed materialized offline stretch artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineStretchArtifactMaterializationRegistration {
    /// Stable materialized artifact identity supplied by the caller.
    pub artifact_id: String,
    /// Plan that produced this artifact.
    pub plan_id: String,
    /// Clip this artifact belongs to, if known.
    pub clip_id: Option<String>,
    /// Media asset this artifact belongs to, if known.
    pub media_asset_id: Option<String>,
    /// Consumer scope this artifact was materialized for.
    pub scope: RuntimeOfflineStretchArtifactScope,
    /// Signal stretch tier used to produce the artifact.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path used to produce the artifact.
    pub offline_path: OfflineHighQualityPath,
    /// Stable cache identity hash for the materialized artifact.
    pub cache_identity_hash: String,
    /// Canonical cache identity key for the materialized artifact.
    pub cache_identity_key: String,
    /// Accepted promotion evidence used for product-facing materialization.
    pub promotion_evidence_id: String,
    /// Source frame count consumed from the decoded source buffer.
    pub input_frame_count: usize,
    /// Output frame count produced for cache/export/freeze consumption.
    pub output_frame_count: usize,
    /// Output channel count.
    pub channels: u16,
    /// Output sample rate.
    pub sample_rate_hz: u32,
    /// Whether this materialized artifact may feed product-facing output.
    pub product_facing_allowed: bool,
}

/// Registration for a runtime-observed render-cache decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineStretchArtifactCacheDecisionRegistration {
    /// Stable cache decision identity supplied by the caller.
    pub decision_id: String,
    /// Plan the cache decision belongs to.
    pub plan_id: String,
    /// Clip this artifact belongs to, if known.
    pub clip_id: Option<String>,
    /// Media asset this artifact belongs to, if known.
    pub media_asset_id: Option<String>,
    /// Consumer scope this cache decision served.
    pub scope: RuntimeOfflineStretchArtifactScope,
    /// Cache lookup/write/invalidation decision.
    pub kind: RuntimeOfflineStretchArtifactCacheDecisionKind,
    /// Signal stretch tier used to produce the cache handoff.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path used to produce the cache handoff.
    pub offline_path: OfflineHighQualityPath,
    /// Stable cache identity hash selected by this decision.
    pub cache_identity_hash: String,
    /// Canonical cache identity key selected by this decision.
    pub cache_identity_key: String,
    /// Accepted promotion evidence used for the selected cache handoff.
    pub promotion_evidence_id: String,
    /// Output frame count selected by the cache decision.
    pub output_frame_count: usize,
    /// Whether the selected cache handoff may feed product-facing output.
    pub product_facing_allowed: bool,
}

/// Runtime-owned observation receipt for one materialized offline stretch artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineStretchArtifactMaterializationSnapshot {
    /// Stable materialized artifact identity supplied by the caller.
    pub artifact_id: String,
    /// Plan that produced this artifact.
    pub plan_id: String,
    /// Clip this artifact belongs to, if known.
    pub clip_id: Option<String>,
    /// Media asset this artifact belongs to, if known.
    pub media_asset_id: Option<String>,
    /// Consumer scope this artifact was materialized for.
    pub scope: RuntimeOfflineStretchArtifactScope,
    /// Signal stretch tier used to produce the artifact.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path used to produce the artifact.
    pub offline_path: OfflineHighQualityPath,
    /// Stable cache identity hash for the materialized artifact.
    pub cache_identity_hash: String,
    /// Canonical cache identity key for the materialized artifact.
    pub cache_identity_key: String,
    /// Accepted promotion evidence used for product-facing materialization.
    pub promotion_evidence_id: String,
    /// Source frame count consumed from the decoded source buffer.
    pub input_frame_count: usize,
    /// Output frame count produced for cache/export/freeze consumption.
    pub output_frame_count: usize,
    /// Output channel count.
    pub channels: u16,
    /// Output sample rate.
    pub sample_rate_hz: u32,
    /// Whether this materialized artifact may feed product-facing output.
    pub product_facing_allowed: bool,
}

/// Runtime-owned observation receipt for one render-cache decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineStretchArtifactCacheDecisionSnapshot {
    /// Stable cache decision identity supplied by the caller.
    pub decision_id: String,
    /// Plan the cache decision belongs to.
    pub plan_id: String,
    /// Clip this artifact belongs to, if known.
    pub clip_id: Option<String>,
    /// Media asset this artifact belongs to, if known.
    pub media_asset_id: Option<String>,
    /// Consumer scope this cache decision served.
    pub scope: RuntimeOfflineStretchArtifactScope,
    /// Cache lookup/write/invalidation decision.
    pub kind: RuntimeOfflineStretchArtifactCacheDecisionKind,
    /// Signal stretch tier used to produce the cache handoff.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path used to produce the cache handoff.
    pub offline_path: OfflineHighQualityPath,
    /// Stable cache identity hash selected by this decision.
    pub cache_identity_hash: String,
    /// Canonical cache identity key selected by this decision.
    pub cache_identity_key: String,
    /// Accepted promotion evidence used for the selected cache handoff.
    pub promotion_evidence_id: String,
    /// Output frame count selected by the cache decision.
    pub output_frame_count: usize,
    /// Whether the selected cache handoff may feed product-facing output.
    pub product_facing_allowed: bool,
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
    /// Offline high-quality renderer path named by the identity input.
    pub offline_path: OfflineHighQualityPath,
    /// Stable cache identity hash, if validation succeeded.
    pub cache_identity_hash: Option<String>,
    /// Canonical cache identity key, if validation succeeded.
    pub cache_identity_key: Option<String>,
    /// Current runtime-owned readiness.
    pub readiness: RuntimeOfflineStretchArtifactReadiness,
    /// Promotion decision from the attached receipt.
    pub promotion_status: StretchPromotionStatus,
    /// Stable evidence or benchmark run identifier.
    pub promotion_evidence_id: Option<String>,
    /// Number of corpus cases accepted by the promotion evidence.
    pub promotion_passed_case_count: u32,
    /// Number of required corpus cases in the promotion evidence.
    pub promotion_required_case_count: u32,
    /// Whether the promotion evidence compared against the draft baseline.
    pub promotion_compared_to_draft_baseline: bool,
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
    /// Number of materialized artifacts observed by render/export/freeze.
    pub materialized_artifact_count: usize,
    /// Number of materialized artifacts allowed to feed product-facing output.
    pub product_facing_materialized_artifact_count: usize,
    /// Number of render-cache bridge decisions observed.
    pub cache_decision_count: usize,
    /// Number of render-cache hit decisions observed.
    pub cache_hit_count: usize,
    /// Number of render-cache write decisions observed.
    pub cache_write_count: usize,
    /// Number of render-cache invalidation decisions observed.
    pub cache_invalidation_count: usize,
    /// Per-plan receipts.
    pub plans: Vec<RuntimeOfflineStretchArtifactPlanSnapshot>,
    /// Per-materialized-artifact receipts.
    pub materialized_artifacts: Vec<RuntimeOfflineStretchArtifactMaterializationSnapshot>,
    /// Per-render-cache decision receipts.
    pub cache_decisions: Vec<RuntimeOfflineStretchArtifactCacheDecisionSnapshot>,
}
