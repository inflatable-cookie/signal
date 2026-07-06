use crate::{
    compare_synthetic_stretch_backends, prioritize_stretch_quality_work, StretchBackendTier,
    StretchBenchmarkComparisonOutcome, StretchSyntheticBenchmarkComparisonReport,
};

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

/// Acceptance policy for synthetic OfflineHighQuality promotion evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchSyntheticPromotionPolicy {
    /// Minimum number of comparison rows required in the synthetic report.
    pub min_comparison_count: usize,
    /// Maximum allowed regressed comparison rows.
    pub max_regressed_count: usize,
    /// Maximum allowed inconclusive comparison rows.
    pub max_inconclusive_count: usize,
    /// Maximum allowed quality-priority rows derived from the report.
    pub max_priority_count: usize,
}

/// Build the current OfflineHighQuality promotion receipt from Signal's
/// synthetic comparison report and the default synthetic promotion policy.
pub fn current_synthetic_offline_high_quality_promotion_receipt(
    evidence_id: impl Into<String>,
) -> StretchPromotionReceipt {
    StretchPromotionReceipt::from_synthetic_offline_high_quality_report(
        evidence_id,
        &compare_synthetic_stretch_backends(),
        StretchSyntheticPromotionPolicy::default(),
    )
}

impl Default for StretchSyntheticPromotionPolicy {
    fn default() -> Self {
        Self {
            min_comparison_count: 27,
            max_regressed_count: 0,
            max_inconclusive_count: 0,
            max_priority_count: 0,
        }
    }
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

    /// Build an OfflineHighQuality promotion receipt from the synthetic
    /// comparison report and acceptance policy.
    pub fn from_synthetic_offline_high_quality_report(
        evidence_id: impl Into<String>,
        report: &StretchSyntheticBenchmarkComparisonReport,
        policy: StretchSyntheticPromotionPolicy,
    ) -> Self {
        let evidence_id = evidence_id.into();
        let passed_count = report
            .comparisons
            .iter()
            .filter(|comparison| {
                matches!(
                    comparison.outcome,
                    StretchBenchmarkComparisonOutcome::Improved
                        | StretchBenchmarkComparisonOutcome::Unchanged
                )
            })
            .count() as u32;
        let required_count = policy.min_comparison_count as u32;
        let priorities =
            prioritize_stretch_quality_work(report, policy.max_priority_count.saturating_add(1));

        let rejection_note = if evidence_id.is_empty() {
            Some("synthetic promotion evidence id is empty")
        } else if report.comparisons.len() < policy.min_comparison_count {
            Some("synthetic promotion report did not meet required comparison coverage")
        } else if report.regressed_count > policy.max_regressed_count {
            Some("synthetic promotion report has regressed comparison rows")
        } else if report.inconclusive_count > policy.max_inconclusive_count {
            Some("synthetic promotion report has inconclusive comparison rows")
        } else if priorities.len() > policy.max_priority_count {
            Some("synthetic promotion report has open quality priorities")
        } else {
            None
        };

        if let Some(note) = rejection_note {
            Self::rejected_offline_high_quality(evidence_id, passed_count, required_count, note)
        } else {
            Self::accepted_offline_high_quality(evidence_id, passed_count, required_count)
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

    #[test]
    fn synthetic_report_policy_accepts_current_offline_high_quality_evidence() {
        let report = compare_synthetic_stretch_backends();
        let receipt = StretchPromotionReceipt::from_synthetic_offline_high_quality_report(
            "synthetic:current",
            &report,
            StretchSyntheticPromotionPolicy::default(),
        );

        assert_eq!(receipt.status, StretchPromotionStatus::Accepted);
        assert_eq!(receipt.passed_case_count, report.comparisons.len() as u32);
        assert_eq!(
            receipt.required_case_count,
            StretchSyntheticPromotionPolicy::default().min_comparison_count as u32
        );
        assert!(receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    }

    #[test]
    fn current_synthetic_receipt_uses_default_policy_report() {
        let receipt =
            current_synthetic_offline_high_quality_promotion_receipt("synthetic:current-helper");

        assert_eq!(receipt.status, StretchPromotionStatus::Accepted);
        assert_eq!(receipt.evidence_id, "synthetic:current-helper");
        assert_eq!(
            receipt.required_case_count,
            StretchSyntheticPromotionPolicy::default().min_comparison_count as u32
        );
        assert!(receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    }

    #[test]
    fn synthetic_report_policy_rejects_missing_coverage() {
        let report = compare_synthetic_stretch_backends();
        let policy = StretchSyntheticPromotionPolicy {
            min_comparison_count: report.comparisons.len() + 1,
            ..StretchSyntheticPromotionPolicy::default()
        };
        let receipt = StretchPromotionReceipt::from_synthetic_offline_high_quality_report(
            "synthetic:short",
            &report,
            policy,
        );

        assert_eq!(receipt.status, StretchPromotionStatus::Rejected);
        assert_eq!(
            receipt.note,
            "synthetic promotion report did not meet required comparison coverage"
        );
        assert_eq!(
            receipt.product_facing_blocker(StretchBackendTier::OfflineHighQuality),
            Some("promotion evidence has not accepted product-facing use")
        );
    }
}
