use super::named_preset_surface_cases::bar_transition_surface_cases;
use super::*;

#[test]
fn beat_tracker_matches_named_preset_surface_expectations() {
    let cases = vec![
        (RhythmPreset::NeutralClick120, 120.0, None, 0.85, 0.05),
        (
            RhythmPreset::StructuredHarmony120(HarmonicRhythmVariant::Active),
            120.0,
            Some(4),
            0.75,
            0.25,
        ),
        (RhythmPreset::AmbiguousSubdivision90, 90.0, None, 0.45, 0.2),
        (
            RhythmPreset::StructuredHarmony120(HarmonicRhythmVariant::Sparse),
            120.0,
            None,
            0.85,
            0.3,
        ),
        (RhythmPreset::WeakBackbeat118, 118.0, Some(4), 0.55, 0.15),
        (
            RhythmPreset::SectionTransition122,
            122.0,
            Some(4),
            0.55,
            0.1,
        ),
        (
            RhythmPreset::FillTransition124(FillDensityVariant::Medium),
            124.0,
            Some(4),
            0.55,
            0.1,
        ),
        (
            RhythmPreset::FillTransition124(FillDensityVariant::Dense),
            124.0,
            Some(4),
            0.5,
            0.12,
        ),
        (
            RhythmPreset::Dropout120(DropoutVariant::Light),
            120.0,
            None,
            0.85,
            0.05,
        ),
        (
            RhythmPreset::Dropout120(DropoutVariant::Medium),
            120.0,
            None,
            0.82,
            0.05,
        ),
        (
            RhythmPreset::Dropout120(DropoutVariant::Heavy),
            120.0,
            None,
            0.4,
            0.05,
        ),
    ]
    .into_iter()
    .chain(bar_transition_surface_cases())
    .collect::<Vec<_>>();

    for (preset, bpm, expected_meter, min_confidence, min_ambiguity) in cases {
        let (_, result) = analyze_preset(preset);
        assert_detected_bpm(preset, &result, bpm, 3.0);
        assert!(
            result.confidence.0 > min_confidence,
            "preset {:?} confidence {}",
            preset,
            result.confidence.0
        );
        assert!(
            result.tempo_ambiguity.0 >= min_ambiguity,
            "preset {:?} ambiguity {}",
            preset,
            result.tempo_ambiguity.0
        );
        if let Some(beats_per_bar) = expected_meter {
            assert_meter(preset, &result, beats_per_bar, 0.18);
        } else {
            assert!(
                result.meter.is_none(),
                "preset {:?} should be meterless",
                preset
            );
        }
    }
}
