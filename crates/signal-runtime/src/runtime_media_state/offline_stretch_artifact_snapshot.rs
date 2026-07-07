use super::*;
use signal_dsp_stretch::{stretch_backend_plan, StretchBackendStatus, StretchBackendTier};

impl RuntimeOfflineStretchArtifactPlanStateModel {
    pub(crate) fn reconcile_plans(
        &mut self,
        plans: Vec<RuntimeOfflineStretchArtifactPlanRegistration>,
    ) {
        let retained_ids = plans
            .iter()
            .map(|plan| plan.plan_id.clone())
            .collect::<BTreeSet<_>>();
        self.plans
            .retain(|plan_id, _| retained_ids.contains(plan_id));
        for plan in plans {
            self.plans.insert(plan.plan_id.clone(), plan);
        }
    }

    pub(crate) fn reconcile_materialized_artifacts(
        &mut self,
        artifacts: Vec<RuntimeOfflineStretchArtifactMaterializationRegistration>,
    ) {
        let retained_ids = artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.clone())
            .collect::<BTreeSet<_>>();
        self.materialized_artifacts
            .retain(|artifact_id, _| retained_ids.contains(artifact_id));
        for artifact in artifacts {
            self.materialized_artifacts
                .insert(artifact.artifact_id.clone(), artifact);
        }
    }

    pub(crate) fn reconcile_cache_decisions(
        &mut self,
        decisions: Vec<RuntimeOfflineStretchArtifactCacheDecisionRegistration>,
    ) {
        let retained_ids = decisions
            .iter()
            .map(|decision| decision.decision_id.clone())
            .collect::<BTreeSet<_>>();
        self.cache_decisions
            .retain(|decision_id, _| retained_ids.contains(decision_id));
        for decision in decisions {
            self.cache_decisions
                .insert(decision.decision_id.clone(), decision);
        }
    }

    pub(crate) fn snapshot(&self) -> RuntimeOfflineStretchArtifactPlanSnapshotSet {
        let plans = self.plans.values().map(snapshot_plan).collect::<Vec<_>>();
        let materialized_artifacts = self
            .materialized_artifacts
            .values()
            .map(snapshot_materialized_artifact)
            .collect::<Vec<_>>();
        let cache_decisions = self
            .cache_decisions
            .values()
            .map(snapshot_cache_decision)
            .collect::<Vec<_>>();
        let ready_plan_count = plans
            .iter()
            .filter(|plan| plan.readiness == RuntimeOfflineStretchArtifactReadiness::Ready)
            .count();
        let awaiting_implementation_count = plans
            .iter()
            .filter(|plan| {
                plan.readiness == RuntimeOfflineStretchArtifactReadiness::AwaitingImplementation
            })
            .count();
        let awaiting_corpus_evidence_count = plans
            .iter()
            .filter(|plan| {
                plan.readiness == RuntimeOfflineStretchArtifactReadiness::AwaitingCorpusEvidence
            })
            .count();
        let invalid_plan_count = plans
            .iter()
            .filter(|plan| plan.readiness == RuntimeOfflineStretchArtifactReadiness::Invalid)
            .count();
        let product_facing_materialized_artifact_count = materialized_artifacts
            .iter()
            .filter(|artifact| artifact.product_facing_allowed)
            .count();
        let cache_hit_count = cache_decisions
            .iter()
            .filter(|decision| decision.kind == RuntimeOfflineStretchArtifactCacheDecisionKind::Hit)
            .count();
        let cache_write_count = cache_decisions
            .iter()
            .filter(|decision| {
                decision.kind == RuntimeOfflineStretchArtifactCacheDecisionKind::Written
            })
            .count();
        let cache_invalidation_count = cache_decisions
            .iter()
            .filter(|decision| {
                decision.kind == RuntimeOfflineStretchArtifactCacheDecisionKind::Invalidated
            })
            .count();

        RuntimeOfflineStretchArtifactPlanSnapshotSet {
            plan_count: plans.len(),
            ready_plan_count,
            awaiting_implementation_count,
            awaiting_corpus_evidence_count,
            invalid_plan_count,
            materialized_artifact_count: materialized_artifacts.len(),
            product_facing_materialized_artifact_count,
            cache_decision_count: cache_decisions.len(),
            cache_hit_count,
            cache_write_count,
            cache_invalidation_count,
            plans,
            materialized_artifacts,
            cache_decisions,
        }
    }
}

fn snapshot_materialized_artifact(
    registration: &RuntimeOfflineStretchArtifactMaterializationRegistration,
) -> RuntimeOfflineStretchArtifactMaterializationSnapshot {
    RuntimeOfflineStretchArtifactMaterializationSnapshot {
        artifact_id: registration.artifact_id.clone(),
        plan_id: registration.plan_id.clone(),
        clip_id: registration.clip_id.clone(),
        media_asset_id: registration.media_asset_id.clone(),
        scope: registration.scope,
        tier: registration.tier,
        offline_path: registration.offline_path,
        cache_identity_hash: registration.cache_identity_hash.clone(),
        cache_identity_key: registration.cache_identity_key.clone(),
        promotion_evidence_id: registration.promotion_evidence_id.clone(),
        input_frame_count: registration.input_frame_count,
        output_frame_count: registration.output_frame_count,
        channels: registration.channels,
        sample_rate_hz: registration.sample_rate_hz,
        product_facing_allowed: registration.product_facing_allowed,
    }
}

fn snapshot_cache_decision(
    registration: &RuntimeOfflineStretchArtifactCacheDecisionRegistration,
) -> RuntimeOfflineStretchArtifactCacheDecisionSnapshot {
    RuntimeOfflineStretchArtifactCacheDecisionSnapshot {
        decision_id: registration.decision_id.clone(),
        plan_id: registration.plan_id.clone(),
        clip_id: registration.clip_id.clone(),
        media_asset_id: registration.media_asset_id.clone(),
        scope: registration.scope,
        kind: registration.kind,
        tier: registration.tier,
        offline_path: registration.offline_path,
        cache_identity_hash: registration.cache_identity_hash.clone(),
        cache_identity_key: registration.cache_identity_key.clone(),
        promotion_evidence_id: registration.promotion_evidence_id.clone(),
        output_frame_count: registration.output_frame_count,
        product_facing_allowed: registration.product_facing_allowed,
    }
}

fn snapshot_plan(
    registration: &RuntimeOfflineStretchArtifactPlanRegistration,
) -> RuntimeOfflineStretchArtifactPlanSnapshot {
    if registration.identity_input.tier != StretchBackendTier::OfflineHighQuality {
        return RuntimeOfflineStretchArtifactPlanSnapshot {
            plan_id: registration.plan_id.clone(),
            clip_id: registration.clip_id.clone(),
            media_asset_id: registration.media_asset_id.clone(),
            scope: registration.scope,
            tier: registration.identity_input.tier,
            offline_path: registration.identity_input.offline_path,
            cache_identity_hash: None,
            cache_identity_key: None,
            readiness: RuntimeOfflineStretchArtifactReadiness::Invalid,
            promotion_status: registration.promotion_receipt.status,
            promotion_evidence_id: promotion_evidence_id(&registration.promotion_receipt),
            promotion_passed_case_count: registration.promotion_receipt.passed_case_count,
            promotion_required_case_count: registration.promotion_receipt.required_case_count,
            promotion_compared_to_draft_baseline: registration
                .promotion_receipt
                .compared_to_draft_baseline,
            product_facing_allowed: false,
            last_error: Some(format!(
                "offline stretch artifacts require OfflineHighQuality, got {:?}",
                registration.identity_input.tier
            )),
        };
    }

    let (cache_identity_hash, cache_identity_key, identity_error) =
        match registration.identity_input.identity() {
            Ok(identity) => (
                Some(identity.stable_hash),
                Some(identity.canonical_key),
                None::<String>,
            ),
            Err(error) => (None, None, Some(format!("{error:?}"))),
        };
    if let Some(error) = identity_error {
        return RuntimeOfflineStretchArtifactPlanSnapshot {
            plan_id: registration.plan_id.clone(),
            clip_id: registration.clip_id.clone(),
            media_asset_id: registration.media_asset_id.clone(),
            scope: registration.scope,
            tier: registration.identity_input.tier,
            offline_path: registration.identity_input.offline_path,
            cache_identity_hash,
            cache_identity_key,
            readiness: RuntimeOfflineStretchArtifactReadiness::Invalid,
            promotion_status: registration.promotion_receipt.status,
            promotion_evidence_id: promotion_evidence_id(&registration.promotion_receipt),
            promotion_passed_case_count: registration.promotion_receipt.passed_case_count,
            promotion_required_case_count: registration.promotion_receipt.required_case_count,
            promotion_compared_to_draft_baseline: registration
                .promotion_receipt
                .compared_to_draft_baseline,
            product_facing_allowed: false,
            last_error: Some(error),
        };
    }

    let backend = stretch_backend_plan(registration.identity_input.tier);
    let promotion_accepted = registration.promotion_receipt.accepts_product_facing_path(
        registration.identity_input.tier,
        registration.identity_input.offline_path,
    );
    let promotion_blocker = registration.promotion_receipt.product_facing_path_blocker(
        registration.identity_input.tier,
        registration.identity_input.offline_path,
    );
    let (readiness, last_error) = match (backend.status, promotion_accepted) {
        (StretchBackendStatus::Planned, _) => (
            RuntimeOfflineStretchArtifactReadiness::AwaitingImplementation,
            Some("OfflineHighQuality is not implemented yet".to_string()),
        ),
        (StretchBackendStatus::Prototype, _) => (
            RuntimeOfflineStretchArtifactReadiness::AwaitingCorpusEvidence,
            Some(
                "OfflineHighQuality prototype has not accepted product-facing promotion"
                    .to_string(),
            ),
        ),
        (StretchBackendStatus::Implemented, false) => (
            RuntimeOfflineStretchArtifactReadiness::AwaitingCorpusEvidence,
            Some(
                promotion_blocker
                    .unwrap_or("OfflineHighQuality corpus evidence has not accepted promotion")
                    .to_string(),
            ),
        ),
        (StretchBackendStatus::Implemented, true) => {
            (RuntimeOfflineStretchArtifactReadiness::Ready, None)
        }
    };

    RuntimeOfflineStretchArtifactPlanSnapshot {
        plan_id: registration.plan_id.clone(),
        clip_id: registration.clip_id.clone(),
        media_asset_id: registration.media_asset_id.clone(),
        scope: registration.scope,
        tier: registration.identity_input.tier,
        offline_path: registration.identity_input.offline_path,
        cache_identity_hash,
        cache_identity_key,
        readiness,
        promotion_status: registration.promotion_receipt.status,
        promotion_evidence_id: promotion_evidence_id(&registration.promotion_receipt),
        promotion_passed_case_count: registration.promotion_receipt.passed_case_count,
        promotion_required_case_count: registration.promotion_receipt.required_case_count,
        promotion_compared_to_draft_baseline: registration
            .promotion_receipt
            .compared_to_draft_baseline,
        product_facing_allowed: readiness == RuntimeOfflineStretchArtifactReadiness::Ready,
        last_error,
    }
}

fn promotion_evidence_id(receipt: &signal_dsp_stretch::StretchPromotionReceipt) -> Option<String> {
    if receipt.evidence_id.is_empty() {
        None
    } else {
        Some(receipt.evidence_id.clone())
    }
}
