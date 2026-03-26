use super::*;

#[test]
fn beat_tracker_calibrates_meter_confidence_between_neutral_and_structured_fixtures() {
    let (_, neutral) = analyze_preset(RhythmPreset::NeutralClick120);
    let (_, structured) = analyze_preset(RhythmPreset::StructuredHarmony120(
        HarmonicRhythmVariant::Active,
    ));
    let structured_meter = structured.meter.as_ref().expect("structured meter");

    assert!(neutral.meter.is_none());
    assert_eq!(structured_meter.beats_per_bar, 4);
    assert!(structured_meter.confidence.0 > 0.2);
    assert_eq!(
        structured_meter.detection_kind,
        super::MeterDetectionKind::WholeTrack
    );
    assert_eq!(structured_meter.trust, super::MeterTrustLevel::Stable);
    assert!(structured_meter.recovery.is_none());
    assert!(structured_meter.confidence_breakdown.support > 0.6);
    assert!(
        structured_meter.support_profile.whole_track_strength.0
            > structured_meter.support_profile.segment_recovery_strength.0
    );
    assert_eq!(
        structured_meter
            .support_profile
            .recovery_duration_strength
            .0,
        0.0
    );
}

#[test]
fn beat_tracker_calibrates_tempo_ambiguity_between_stable_and_subdivided_fixtures() {
    let (_, stable) = analyze_preset(RhythmPreset::NeutralClick120);
    let (_, ambiguous) = analyze_preset(RhythmPreset::AmbiguousSubdivision90);

    assert!(ambiguous.tempo_ambiguity.0 > stable.tempo_ambiguity.0);
    assert!(ambiguous.tempo_candidates.len() >= 2);
    assert!(stable.confidence.0 >= ambiguous.confidence.0);
}
