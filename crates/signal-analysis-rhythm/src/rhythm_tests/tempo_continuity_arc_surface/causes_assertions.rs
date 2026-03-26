use super::super::super::*;
use super::fixtures::ArcSurfaceCases;

pub(super) fn assert_causes_and_unresolved_spans(cases: &ArcSurfaceCases) {
    assert_eq!(
        cases.integer.continuity.trigger,
        TempoContinuityTrigger::StableRevalidation
    );
    assert_eq!(
        cases.integer.continuity.unresolved,
        TempoContinuityUnresolvedSpan {
            beats: 0,
            failed_revalidations: 0,
        }
    );
    assert_eq!(
        cases.integer.continuity.causes.primary,
        TempoContinuityCause::StableTempoEvidence
    );

    assert_eq!(
        cases.core_window.continuity.trigger,
        TempoContinuityTrigger::BoundaryDrift
    );
    assert_eq!(
        cases.core_window.continuity.unresolved,
        TempoContinuityUnresolvedSpan {
            beats: 8,
            failed_revalidations: 2,
        }
    );
    assert_eq!(
        cases.core_window.continuity.causes.primary,
        TempoContinuityCause::BoundaryDrift
    );
    assert!(cases
        .core_window
        .continuity
        .causes
        .secondary
        .into_iter()
        .flatten()
        .any(|cause| cause == TempoContinuityCause::CoreWindowCarry));

    assert_eq!(
        cases.guarded_refined.continuity.trigger,
        TempoContinuityTrigger::AmbiguityCarry
    );
    assert_eq!(
        cases.guarded_refined.continuity.unresolved,
        TempoContinuityUnresolvedSpan {
            beats: 4,
            failed_revalidations: 1,
        }
    );
    assert_eq!(
        cases.guarded_refined.continuity.causes.primary,
        TempoContinuityCause::TempoAmbiguity
    );

    assert_eq!(
        cases.deferred.continuity.trigger,
        TempoContinuityTrigger::EvidenceLoss
    );
    assert_eq!(cases.deferred.continuity.unresolved.beats, 0);
    assert_eq!(
        cases.deferred.continuity.causes.primary,
        TempoContinuityCause::TempoAmbiguity
    );
    assert!(cases
        .deferred
        .continuity
        .causes
        .secondary
        .into_iter()
        .flatten()
        .any(|cause| cause == TempoContinuityCause::EvidenceLoss));
}
