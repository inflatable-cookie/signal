use super::*;

#[test]
fn backend_plan_tracks_signal_owned_tiers() {
    assert_eq!(SIGNAL_STRETCH_BACKEND_PLAN.len(), 3);
    assert_eq!(
        stretch_backend_plan(StretchBackendTier::Repitch).status,
        StretchBackendStatus::Implemented
    );
    let preview = stretch_backend_plan(StretchBackendTier::RealtimePreview);
    assert_eq!(preview.status, StretchBackendStatus::Prototype);
    assert!(preview.independent_tempo_and_pitch);
    assert!(preview.dynamic_ratio);
    assert!(!preview.audio_thread_safe);

    let offline = stretch_backend_plan(StretchBackendTier::OfflineHighQuality);
    assert_eq!(offline.status, StretchBackendStatus::Implemented);
    assert!(offline.transient_preservation);
    assert!(offline.vertical_phase_coherence);
    assert!(offline.deterministic_output);
}
