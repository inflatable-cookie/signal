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

    pub(crate) fn snapshot(&self) -> RuntimeOfflineStretchArtifactPlanSnapshotSet {
        let plans = self.plans.values().map(snapshot_plan).collect::<Vec<_>>();
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

        RuntimeOfflineStretchArtifactPlanSnapshotSet {
            plan_count: plans.len(),
            ready_plan_count,
            awaiting_implementation_count,
            awaiting_corpus_evidence_count,
            invalid_plan_count,
            plans,
        }
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
            cache_identity_hash: None,
            cache_identity_key: None,
            readiness: RuntimeOfflineStretchArtifactReadiness::Invalid,
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
            cache_identity_hash,
            cache_identity_key,
            readiness: RuntimeOfflineStretchArtifactReadiness::Invalid,
            product_facing_allowed: false,
            last_error: Some(error),
        };
    }

    let backend = stretch_backend_plan(registration.identity_input.tier);
    let (readiness, last_error) = match (backend.status, registration.corpus_evidence_accepted) {
        (StretchBackendStatus::Planned, _) => (
            RuntimeOfflineStretchArtifactReadiness::AwaitingImplementation,
            Some("OfflineHighQuality is not implemented yet".to_string()),
        ),
        (StretchBackendStatus::Implemented, false) => (
            RuntimeOfflineStretchArtifactReadiness::AwaitingCorpusEvidence,
            Some("OfflineHighQuality corpus evidence has not accepted promotion".to_string()),
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
        cache_identity_hash,
        cache_identity_key,
        readiness,
        product_facing_allowed: readiness == RuntimeOfflineStretchArtifactReadiness::Ready,
        last_error,
    }
}
