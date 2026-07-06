use crate::StretchBackendTier;

/// Promotion decision for a Signal-owned stretch tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchPromotionStatus {
    /// No promotion evidence has been recorded.
    NotEvaluated,
    /// Evidence accepted product-facing use.
    Accepted,
    /// Evidence rejected product-facing use.
    Rejected,
}

/// Evidence receipt for product-facing stretch promotion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StretchPromotionReceipt {
    /// Stretch tier covered by this receipt.
    pub tier: StretchBackendTier,
    /// Promotion decision.
    pub status: StretchPromotionStatus,
    /// Stable evidence or benchmark run identifier.
    pub evidence_id: String,
    /// Whether this run compared against the current draft baseline.
    pub compared_to_draft_baseline: bool,
    /// Number of corpus cases that passed acceptance.
    pub passed_case_count: u32,
    /// Number of required corpus cases for this promotion gate.
    pub required_case_count: u32,
    /// Operator or CI note associated with the decision.
    pub note: String,
}

impl StretchPromotionReceipt {
    /// Empty receipt for a tier with no accepted evidence yet.
    pub fn not_evaluated(tier: StretchBackendTier) -> Self {
        Self {
            tier,
            status: StretchPromotionStatus::NotEvaluated,
            evidence_id: String::new(),
            compared_to_draft_baseline: false,
            passed_case_count: 0,
            required_case_count: 0,
            note: "promotion evidence has not been recorded".to_string(),
        }
    }

    /// Accepted receipt for `OfflineHighQuality` promotion evidence.
    pub fn accepted_offline_high_quality(
        evidence_id: impl Into<String>,
        passed_case_count: u32,
        required_case_count: u32,
    ) -> Self {
        Self {
            tier: StretchBackendTier::OfflineHighQuality,
            status: StretchPromotionStatus::Accepted,
            evidence_id: evidence_id.into(),
            compared_to_draft_baseline: true,
            passed_case_count,
            required_case_count,
            note: "OfflineHighQuality evidence accepted product-facing promotion".to_string(),
        }
    }

    /// Rejected receipt for `OfflineHighQuality` promotion evidence.
    pub fn rejected_offline_high_quality(
        evidence_id: impl Into<String>,
        passed_case_count: u32,
        required_case_count: u32,
        note: impl Into<String>,
    ) -> Self {
        Self {
            tier: StretchBackendTier::OfflineHighQuality,
            status: StretchPromotionStatus::Rejected,
            evidence_id: evidence_id.into(),
            compared_to_draft_baseline: true,
            passed_case_count,
            required_case_count,
            note: note.into(),
        }
    }

    /// Whether this receipt allows product-facing use for `tier`.
    pub fn accepts_product_facing_use(&self, tier: StretchBackendTier) -> bool {
        self.tier == tier
            && self.status == StretchPromotionStatus::Accepted
            && !self.evidence_id.is_empty()
            && self.compared_to_draft_baseline
            && self.required_case_count > 0
            && self.passed_case_count >= self.required_case_count
    }

    /// Human-readable blocking reason when product-facing use is not accepted.
    pub fn product_facing_blocker(&self, tier: StretchBackendTier) -> Option<&'static str> {
        if self.tier != tier {
            return Some("promotion receipt tier does not match artifact tier");
        }
        if self.status != StretchPromotionStatus::Accepted {
            return Some("promotion evidence has not accepted product-facing use");
        }
        if self.evidence_id.is_empty() {
            return Some("promotion evidence id is empty");
        }
        if !self.compared_to_draft_baseline {
            return Some("promotion evidence did not compare against the draft baseline");
        }
        if self.required_case_count == 0 {
            return Some("promotion evidence has no required corpus count");
        }
        if self.passed_case_count < self.required_case_count {
            return Some("promotion evidence did not pass the required corpus count");
        }
        None
    }
}

impl Default for StretchPromotionReceipt {
    fn default() -> Self {
        Self::not_evaluated(StretchBackendTier::OfflineHighQuality)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_offline_high_quality_receipt_allows_product_use() {
        let receipt = StretchPromotionReceipt::accepted_offline_high_quality("run:001", 8, 8);

        assert!(receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
        assert_eq!(
            receipt.product_facing_blocker(StretchBackendTier::OfflineHighQuality),
            None
        );
    }

    #[test]
    fn promotion_receipt_blocks_incomplete_or_mismatched_evidence() {
        let not_evaluated = StretchPromotionReceipt::default();
        let incomplete = StretchPromotionReceipt::accepted_offline_high_quality("run:002", 7, 8);
        let mismatched = StretchPromotionReceipt::accepted_offline_high_quality("run:003", 8, 8);

        assert!(!not_evaluated.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
        assert_eq!(
            not_evaluated.product_facing_blocker(StretchBackendTier::OfflineHighQuality),
            Some("promotion evidence has not accepted product-facing use")
        );
        assert!(!incomplete.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
        assert_eq!(
            incomplete.product_facing_blocker(StretchBackendTier::OfflineHighQuality),
            Some("promotion evidence did not pass the required corpus count")
        );
        assert!(!mismatched.accepts_product_facing_use(StretchBackendTier::RealtimePreview));
        assert_eq!(
            mismatched.product_facing_blocker(StretchBackendTier::RealtimePreview),
            Some("promotion receipt tier does not match artifact tier")
        );
    }
}
