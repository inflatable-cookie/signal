use super::super::*;

#[test]
fn stretch_quality_priorities_are_regression_only_and_sorted() {
    let report = compare_synthetic_stretch_backends();
    let priorities = prioritize_stretch_quality_work(&report, 8);
    let formatted = format_stretch_quality_priority_report(&priorities);

    assert!(priorities.is_empty());
    for priority in &priorities {
        assert!(matches!(
            priority.outcome,
            StretchBenchmarkComparisonOutcome::Regressed
                | StretchBenchmarkComparisonOutcome::Inconclusive
        ));
        assert!(priority.priority_score.is_finite());
        assert!(priority.priority_score > 0.0);
    }
    for pair in priorities.windows(2) {
        assert!(pair[0].priority_score >= pair[1].priority_score);
    }
    assert!(formatted.starts_with("stretch_quality_priorities count="));
    assert_eq!(formatted, "stretch_quality_priorities count=0");
}
