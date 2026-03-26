use super::fixtures::ArcSurfaceCases;

pub(super) fn assert_cross_surface_relationships(cases: &ArcSurfaceCases) {
    assert_eq!(
        cases
            .core_window
            .continuity
            .arc_decision
            .expiry
            .max_failed_revalidations,
        2
    );
    assert_eq!(
        cases
            .guarded_refined
            .continuity
            .arc_decision
            .expiry
            .fallback_after_beats,
        12
    );
    assert!(
        cases.integer.continuity.arc_support.refresh_strength.0
            > cases.core_window.continuity.arc_support.refresh_strength.0
    );
    assert!(
        cases.core_window.continuity.arc_support.drift_pressure.0
            > cases.integer.continuity.arc_support.drift_pressure.0
    );
    assert!(
        cases.deferred.continuity.arc_support.instability_pressure.0
            > cases
                .guarded_refined
                .continuity
                .arc_support
                .instability_pressure
                .0
    );
    assert!(
        cases.integer.continuity.arc_decision.confidence.0
            > cases.guarded_refined.continuity.arc_decision.confidence.0
    );
    assert!(
        cases.deferred.continuity.arc_decision.confidence.0
            > cases.core_window.continuity.arc_decision.confidence.0
    );
    assert!(
        cases
            .core_window
            .continuity
            .arc_decision
            .downgrade_trend_support
            .next_stage_pressure
            .0
            > cases
                .core_window
                .continuity
                .arc_decision
                .downgrade_trend_support
                .current_pressure
                .0
    );
    assert!(
        cases
            .integer
            .continuity
            .arc_decision
            .downgrade_trend_support
            .terminal_pressure
            .0
            > cases
                .integer
                .continuity
                .arc_decision
                .downgrade_trend_support
                .current_pressure
                .0
    );
    assert!(
        cases
            .core_window
            .continuity
            .arc_decision
            .downgrade_support
            .failed_revalidation_pressure
            .0
            > cases
                .integer
                .continuity
                .arc_decision
                .downgrade_support
                .failed_revalidation_pressure
                .0
    );
}
