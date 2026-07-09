use crate::{
    compare_synthetic_stretch_backends, prioritize_stretch_quality_work, OfflineHighQualityPath,
    StretchBackendTier, StretchBenchmarkComparisonOutcome,
    StretchSyntheticBenchmarkComparisonReport,
};

/// Promotion decision for a Signal-owned stretch tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchPromotionStatus {
    /// No promotion evidence has been recorded.
    NotEvaluated,
    /// The supplied evidence set passed its own acceptance policy.
    ///
    /// Product-facing use may still be blocked when the composite quality
    /// evidence on the receipt is incomplete.
    Accepted,
    /// The supplied evidence set failed its own acceptance policy.
    Rejected,
}

/// Evidence receipt evaluated by the product-facing stretch quality gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StretchPromotionReceipt {
    /// Stretch tier covered by this receipt.
    pub tier: StretchBackendTier,
    /// Offline high-quality renderer path covered by this receipt.
    pub offline_path: OfflineHighQualityPath,
    /// Promotion decision.
    pub status: StretchPromotionStatus,
    /// Stable evidence or benchmark run identifier.
    pub evidence_id: String,
    /// Whether this run compared against the current draft baseline.
    pub compared_to_draft_baseline: bool,
    /// Whether the absolute full-render integrity gate passed.
    pub absolute_integrity_passed: bool,
    /// Number of accepted external-comparator rows.
    pub comparator_row_count: u32,
    /// Number of external-comparator rows required by this receipt.
    pub required_comparator_row_count: u32,
    /// Number of corpus cases that passed acceptance.
    pub passed_case_count: u32,
    /// Number of required corpus cases for this promotion gate.
    pub required_case_count: u32,
    /// Number of required real-source listening families with completed notes.
    pub completed_listening_family_count: u32,
    /// Number of real-source listening families required for promotion.
    pub required_listening_family_count: u32,
    /// Operator or CI note associated with the decision.
    pub note: String,
}

/// Required real-source families for OfflineHighQuality promotion.
pub const REQUIRED_STRETCH_LISTENING_FAMILY_COUNT: u32 = 5;

/// Composite evidence required for product-quality stretch promotion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchProductQualityEvidence {
    /// Whether the evidence includes the draft-baseline regression comparison.
    pub compared_to_draft_baseline: bool,
    /// Whether every measured row passed the absolute integrity gate.
    pub absolute_integrity_passed: bool,
    /// Number of accepted external-comparator rows.
    pub comparator_row_count: u32,
    /// Minimum external-comparator rows required by this evidence set.
    pub required_comparator_row_count: u32,
    /// Number of corpus rows that passed their configured acceptance policy.
    pub passed_case_count: u32,
    /// Minimum corpus rows required by this evidence set.
    pub required_case_count: u32,
    /// Number of required real-source families with completed listening notes.
    pub completed_listening_family_count: u32,
    /// Number of real-source listening families required by this evidence set.
    pub required_listening_family_count: u32,
}

/// Acceptance policy for synthetic OfflineHighQuality regression evidence.
///
/// Passing this policy is necessary but cannot open product-facing promotion
/// without absolute integrity, comparator, and completed listening evidence.
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

/// Build the current OfflineHighQuality regression-evidence receipt from
/// Signal's synthetic comparison report and the default synthetic policy.
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
            offline_path: OfflineHighQualityPath::Default,
            status: StretchPromotionStatus::NotEvaluated,
            evidence_id: String::new(),
            compared_to_draft_baseline: false,
            absolute_integrity_passed: false,
            comparator_row_count: 0,
            required_comparator_row_count: 0,
            passed_case_count: 0,
            required_case_count: 0,
            completed_listening_family_count: 0,
            required_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
            note: "promotion evidence has not been recorded".to_string(),
        }
    }

    /// Build a receipt from composite product-quality evidence.
    pub fn from_product_quality_evidence(
        evidence_id: impl Into<String>,
        offline_path: OfflineHighQualityPath,
        evidence: StretchProductQualityEvidence,
    ) -> Self {
        let evidence_id = evidence_id.into();
        let rejection_note = product_quality_rejection_note(&evidence_id, evidence);
        Self {
            tier: StretchBackendTier::OfflineHighQuality,
            offline_path,
            status: if rejection_note.is_none() {
                StretchPromotionStatus::Accepted
            } else {
                StretchPromotionStatus::Rejected
            },
            evidence_id,
            compared_to_draft_baseline: evidence.compared_to_draft_baseline,
            absolute_integrity_passed: evidence.absolute_integrity_passed,
            comparator_row_count: evidence.comparator_row_count,
            required_comparator_row_count: evidence.required_comparator_row_count,
            passed_case_count: evidence.passed_case_count,
            required_case_count: evidence.required_case_count,
            completed_listening_family_count: evidence.completed_listening_family_count,
            required_listening_family_count: evidence.required_listening_family_count,
            note: rejection_note
                .unwrap_or("composite product-quality evidence accepted product-facing promotion")
                .to_string(),
        }
    }

    /// Rejected receipt for `OfflineHighQuality` regression evidence.
    pub fn rejected_offline_high_quality(
        evidence_id: impl Into<String>,
        passed_case_count: u32,
        required_case_count: u32,
        note: impl Into<String>,
    ) -> Self {
        Self {
            tier: StretchBackendTier::OfflineHighQuality,
            offline_path: OfflineHighQualityPath::Default,
            status: StretchPromotionStatus::Rejected,
            evidence_id: evidence_id.into(),
            compared_to_draft_baseline: true,
            absolute_integrity_passed: false,
            comparator_row_count: 0,
            required_comparator_row_count: 0,
            passed_case_count,
            required_case_count,
            completed_listening_family_count: 0,
            required_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
            note: note.into(),
        }
    }

    /// Build an OfflineHighQuality regression receipt from the synthetic
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
            Some("synthetic regression evidence id is empty")
        } else if report.comparisons.len() < policy.min_comparison_count {
            Some("synthetic regression report did not meet required comparison coverage")
        } else if report.regressed_count > policy.max_regressed_count {
            Some("synthetic regression report has regressed comparison rows")
        } else if report.inconclusive_count > policy.max_inconclusive_count {
            Some("synthetic regression report has inconclusive comparison rows")
        } else if priorities.len() > policy.max_priority_count {
            Some("synthetic regression report has open quality priorities")
        } else {
            None
        };

        if let Some(note) = rejection_note {
            Self::rejected_offline_high_quality(evidence_id, passed_count, required_count, note)
        } else {
            Self {
                tier: StretchBackendTier::OfflineHighQuality,
                offline_path: OfflineHighQualityPath::Default,
                status: StretchPromotionStatus::Accepted,
                evidence_id,
                compared_to_draft_baseline: true,
                absolute_integrity_passed: false,
                comparator_row_count: 0,
                required_comparator_row_count: 0,
                passed_case_count: passed_count,
                required_case_count: required_count,
                completed_listening_family_count: 0,
                required_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
                note: "synthetic regression evidence accepted; composite product-quality evidence remains required".to_string(),
            }
        }
    }

    /// Whether this receipt allows product-facing use for `tier`.
    pub fn accepts_product_facing_use(&self, tier: StretchBackendTier) -> bool {
        self.accepts_product_facing_path(tier, OfflineHighQualityPath::Default)
    }

    /// Whether this receipt allows product-facing use for `tier` and
    /// `offline_path`.
    pub fn accepts_product_facing_path(
        &self,
        tier: StretchBackendTier,
        offline_path: OfflineHighQualityPath,
    ) -> bool {
        self.tier == tier
            && self.offline_path == offline_path
            && self.status == StretchPromotionStatus::Accepted
            && !self.evidence_id.is_empty()
            && self.compared_to_draft_baseline
            && self.absolute_integrity_passed
            && self.required_comparator_row_count > 0
            && self.comparator_row_count >= self.required_comparator_row_count
            && self.required_case_count > 0
            && self.passed_case_count >= self.required_case_count
            && self.required_listening_family_count >= REQUIRED_STRETCH_LISTENING_FAMILY_COUNT
            && self.completed_listening_family_count >= self.required_listening_family_count
    }

    /// Human-readable blocking reason when product-facing use is not accepted.
    pub fn product_facing_blocker(&self, tier: StretchBackendTier) -> Option<&'static str> {
        self.product_facing_path_blocker(tier, OfflineHighQualityPath::Default)
    }

    /// Human-readable blocking reason when product-facing use is not accepted
    /// for `tier` and `offline_path`.
    pub fn product_facing_path_blocker(
        &self,
        tier: StretchBackendTier,
        offline_path: OfflineHighQualityPath,
    ) -> Option<&'static str> {
        if self.tier != tier {
            return Some("promotion receipt tier does not match artifact tier");
        }
        if self.offline_path != offline_path {
            return Some("promotion receipt offline path does not match artifact path");
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
        if !self.absolute_integrity_passed {
            return Some("promotion evidence did not pass absolute render integrity");
        }
        if self.required_comparator_row_count == 0 {
            return Some("promotion evidence has no required external comparator count");
        }
        if self.comparator_row_count < self.required_comparator_row_count {
            return Some("promotion evidence did not pass the required external comparator count");
        }
        if self.required_case_count == 0 {
            return Some("promotion evidence has no required corpus count");
        }
        if self.passed_case_count < self.required_case_count {
            return Some("promotion evidence did not pass the required corpus count");
        }
        if self.required_listening_family_count < REQUIRED_STRETCH_LISTENING_FAMILY_COUNT {
            return Some("promotion evidence does not require all real-source listening families");
        }
        if self.completed_listening_family_count < self.required_listening_family_count {
            return Some("promotion evidence has incomplete real-source listening findings");
        }
        None
    }
}

fn product_quality_rejection_note(
    evidence_id: &str,
    evidence: StretchProductQualityEvidence,
) -> Option<&'static str> {
    if evidence_id.is_empty() {
        Some("product-quality evidence id is empty")
    } else if !evidence.compared_to_draft_baseline {
        Some("product-quality evidence did not compare against the draft baseline")
    } else if !evidence.absolute_integrity_passed {
        Some("product-quality evidence did not pass absolute render integrity")
    } else if evidence.required_comparator_row_count == 0
        || evidence.comparator_row_count < evidence.required_comparator_row_count
    {
        Some("product-quality evidence did not pass external comparator coverage")
    } else if evidence.required_case_count == 0
        || evidence.passed_case_count < evidence.required_case_count
    {
        Some("product-quality evidence did not pass corpus coverage")
    } else if evidence.required_listening_family_count < REQUIRED_STRETCH_LISTENING_FAMILY_COUNT
        || evidence.completed_listening_family_count < evidence.required_listening_family_count
    {
        Some("product-quality evidence has incomplete real-source listening findings")
    } else {
        None
    }
}

impl Default for StretchPromotionReceipt {
    fn default() -> Self {
        Self::not_evaluated(StretchBackendTier::OfflineHighQuality)
    }
}

#[cfg(test)]
#[path = "promotion/tests.rs"]
mod tests;
