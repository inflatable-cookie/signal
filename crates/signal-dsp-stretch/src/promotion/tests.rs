use super::*;

fn complete_product_quality_evidence(
    passed_case_count: u32,
    required_case_count: u32,
) -> StretchProductQualityEvidence {
    StretchProductQualityEvidence {
        compared_to_draft_baseline: true,
        absolute_integrity_passed: true,
        comparator_row_count: 18,
        required_comparator_row_count: 18,
        passed_case_count,
        required_case_count,
        completed_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
        required_listening_family_count: REQUIRED_STRETCH_LISTENING_FAMILY_COUNT,
    }
}

#[test]
fn accepted_offline_high_quality_receipt_allows_product_use() {
    let receipt = StretchPromotionReceipt::from_product_quality_evidence(
        "run:001",
        OfflineHighQualityPath::Default,
        complete_product_quality_evidence(8, 8),
    );

    assert!(receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    assert!(receipt.accepts_product_facing_path(
        StretchBackendTier::OfflineHighQuality,
        OfflineHighQualityPath::Default
    ));
    assert_eq!(
        receipt.product_facing_blocker(StretchBackendTier::OfflineHighQuality),
        None
    );
    assert_eq!(
        receipt.product_facing_path_blocker(
            StretchBackendTier::OfflineHighQuality,
            OfflineHighQualityPath::Default
        ),
        None
    );
}

#[test]
fn selector_receipt_allows_only_selector_path() {
    let receipt = StretchPromotionReceipt::from_product_quality_evidence(
        "fma-rubberband-selector:001",
        OfflineHighQualityPath::CompressionShortWindowSelector,
        complete_product_quality_evidence(20, 20),
    );

    assert!(receipt.accepts_product_facing_path(
        StretchBackendTier::OfflineHighQuality,
        OfflineHighQualityPath::CompressionShortWindowSelector
    ));
    assert!(!receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    assert_eq!(
        receipt.product_facing_path_blocker(
            StretchBackendTier::OfflineHighQuality,
            OfflineHighQualityPath::Default
        ),
        Some("promotion receipt offline path does not match artifact path")
    );
}

#[test]
fn expansion_selector_receipt_allows_only_expansion_selector_path() {
    let receipt = StretchPromotionReceipt::from_product_quality_evidence(
        "fma-rubberband-expansion-selector:001",
        OfflineHighQualityPath::ExpansionShortWindowSelector,
        complete_product_quality_evidence(40, 40),
    );

    assert!(receipt.accepts_product_facing_path(
        StretchBackendTier::OfflineHighQuality,
        OfflineHighQualityPath::ExpansionShortWindowSelector
    ));
    assert!(!receipt.accepts_product_facing_path(
        StretchBackendTier::OfflineHighQuality,
        OfflineHighQualityPath::CompressionShortWindowSelector
    ));
    assert!(!receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    assert_eq!(
        receipt.product_facing_path_blocker(
            StretchBackendTier::OfflineHighQuality,
            OfflineHighQualityPath::Default
        ),
        Some("promotion receipt offline path does not match artifact path")
    );
}

#[test]
fn promotion_receipt_blocks_incomplete_mismatched_or_wrong_path_evidence() {
    let not_evaluated = StretchPromotionReceipt::default();
    let incomplete = StretchPromotionReceipt::from_product_quality_evidence(
        "run:002",
        OfflineHighQualityPath::Default,
        complete_product_quality_evidence(7, 8),
    );
    let mismatched = StretchPromotionReceipt::from_product_quality_evidence(
        "run:003",
        OfflineHighQualityPath::Default,
        complete_product_quality_evidence(8, 8),
    );
    let default_path = StretchPromotionReceipt::from_product_quality_evidence(
        "run:004",
        OfflineHighQualityPath::Default,
        complete_product_quality_evidence(8, 8),
    );

    assert!(!not_evaluated.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    assert_eq!(
        not_evaluated.product_facing_blocker(StretchBackendTier::OfflineHighQuality),
        Some("promotion evidence has not accepted product-facing use")
    );
    assert!(!incomplete.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    assert_eq!(
        incomplete.note,
        "product-quality evidence did not pass corpus coverage"
    );
    assert!(!mismatched.accepts_product_facing_use(StretchBackendTier::RealtimePreview));
    assert_eq!(
        mismatched.product_facing_blocker(StretchBackendTier::RealtimePreview),
        Some("promotion receipt tier does not match artifact tier")
    );
    assert!(!default_path.accepts_product_facing_path(
        StretchBackendTier::OfflineHighQuality,
        OfflineHighQualityPath::CompressionShortWindowSelector
    ));
    assert!(!default_path.accepts_product_facing_path(
        StretchBackendTier::OfflineHighQuality,
        OfflineHighQualityPath::ExpansionShortWindowSelector
    ));
    assert_eq!(
        default_path.product_facing_path_blocker(
            StretchBackendTier::OfflineHighQuality,
            OfflineHighQualityPath::CompressionShortWindowSelector
        ),
        Some("promotion receipt offline path does not match artifact path")
    );
}

#[test]
fn synthetic_report_policy_accepts_regression_evidence_but_blocks_product_quality() {
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
    assert!(!receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    assert_eq!(
        receipt.product_facing_blocker(StretchBackendTier::OfflineHighQuality),
        Some("promotion evidence did not pass absolute render integrity")
    );
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
    assert!(!receipt.accepts_product_facing_use(StretchBackendTier::OfflineHighQuality));
    assert_eq!(receipt.completed_listening_family_count, 0);
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
        "synthetic regression report did not meet required comparison coverage"
    );
    assert_eq!(
        receipt.product_facing_blocker(StretchBackendTier::OfflineHighQuality),
        Some("promotion evidence has not accepted product-facing use")
    );
}
