use super::*;

#[test]
fn detector_exposes_stable_local_key_tracking_for_c_major_sections() {
    let audio = tonal_sequence_mix(
        48_000,
        &[
            (&[261.63, 329.63, 392.0], 6.0),
            (&[261.63, 329.63, 392.0], 6.0),
        ],
    );
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
    let result = detector.analyze(&audio);

    assert!(result.local_tracking.segments.len() >= 2);
    assert!(result.local_tracking.changes.is_empty());
    assert!(
        result.local_tracking.ambiguities.is_empty(),
        "unexpected ambiguities: {:?}",
        result.local_tracking
    );
    assert!(result
        .local_tracking
        .segments
        .iter()
        .all(|segment| segment.key
            == Some(crate::Key {
                tonic: Tonic::C,
                mode: KeyMode::Major
            })));
}

#[test]
fn detector_exposes_local_key_shift_and_harmonic_change_for_modulation() {
    let audio = tonal_sequence_mix(
        48_000,
        &[
            (&[261.63, 329.63, 392.0], 6.0),
            (&[196.0, 246.94, 293.66], 6.0),
        ],
    );
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
    let result = detector.analyze(&audio);

    assert!(result.local_tracking.segments.len() >= 2);
    let first = result
        .local_tracking
        .segments
        .first()
        .expect("first local segment");
    let last = result
        .local_tracking
        .segments
        .last()
        .expect("last local segment");
    assert_eq!(
        first.key,
        Some(crate::Key {
            tonic: Tonic::C,
            mode: KeyMode::Major,
        })
    );
    assert_eq!(
        last.key,
        Some(crate::Key {
            tonic: Tonic::G,
            mode: KeyMode::Major,
        })
    );
    let change = result
        .local_tracking
        .changes
        .iter()
        .find(|change| change.kind == HarmonicChangeKind::ConfirmedKeyChange)
        .expect("confirmed key change");
    assert_eq!(
        change.from_key,
        Some(crate::Key {
            tonic: Tonic::C,
            mode: KeyMode::Major,
        })
    );
    assert_eq!(
        change.to_key,
        Some(crate::Key {
            tonic: Tonic::G,
            mode: KeyMode::Major,
        })
    );
    assert!(change.confidence.0 > 0.1);
    assert!(change.chroma_distance.0 > 0.2);
    let ambiguity = result
        .local_tracking
        .ambiguities
        .iter()
        .find(|ambiguity| ambiguity.kind == TonalAmbiguityKind::Modulation)
        .expect("modulation ambiguity");
    assert_eq!(
        ambiguity.primary_key,
        Some(crate::Key {
            tonic: Tonic::C,
            mode: KeyMode::Major,
        })
    );
    assert_eq!(
        ambiguity.alternate_key,
        Some(crate::Key {
            tonic: Tonic::G,
            mode: KeyMode::Major,
        })
    );
}

#[test]
fn detector_surfaces_weak_tonal_centre_ambiguity() {
    let audio = tonal_mix(
        48_000,
        &[
            261.63, 277.18, 293.66, 311.13, 329.63, 349.23, 369.99, 392.0, 415.3, 440.0, 466.16,
            493.88,
        ],
        8.0,
    );
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());
    let result = detector.analyze(&audio);

    let ambiguity = result
        .local_tracking
        .ambiguities
        .iter()
        .find(|ambiguity| ambiguity.kind == TonalAmbiguityKind::WeakTonalCenter)
        .unwrap_or_else(|| panic!("weak tonal-centre ambiguity: {:?}", result.local_tracking));
    assert!(ambiguity.confidence.0 >= 0.5);
    assert!(result
        .local_tracking
        .segments
        .iter()
        .all(|segment| matches!(
            segment.ambiguity,
            Some(crate::TonalSegmentAmbiguitySummary {
                kind: TonalAmbiguityKind::WeakTonalCenter,
                ..
            })
        )));
}

#[test]
fn detector_surfaces_mixed_tonality_ambiguity_for_competing_sections() {
    let audio = tonal_sequence_mix(
        48_000,
        &[
            (&[261.63, 329.63, 392.0], 4.0),
            (&[196.0, 246.94, 293.66], 4.0),
            (&[261.63, 329.63, 392.0], 4.0),
        ],
    );
    let mut config = KeyDetectorConfig::medium();
    config.section_window_seconds = 4;
    config.section_hop_seconds = 2;
    let mut detector = KeyDetector::new(config);
    let result = detector.analyze(&audio);

    let ambiguity = result
        .local_tracking
        .ambiguities
        .iter()
        .find(|ambiguity| ambiguity.kind == TonalAmbiguityKind::MixedTonality)
        .unwrap_or_else(|| panic!("mixed-tonality ambiguity: {:?}", result.local_tracking));
    assert!(
        ambiguity.confidence.0 > 0.1,
        "mixed ambiguity too weak: {:?}",
        result.local_tracking
    );
    assert_eq!(
        ambiguity.primary_key,
        Some(crate::Key {
            tonic: Tonic::C,
            mode: KeyMode::Major,
        })
    );
    assert_eq!(
        ambiguity.alternate_key,
        Some(crate::Key {
            tonic: Tonic::G,
            mode: KeyMode::Major,
        })
    );
}

#[test]
fn harness_tonal_cases_meet_frozen_acceptance_thresholds() {
    let cases = tonal_acceptance_cases();
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());

    let report =
        run_audio_acceptance_harness(&cases, |audio| detector.analyze(audio), tonal_metrics);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert!(report
        .cases
        .iter()
        .all(|case| case.status == AcceptanceStatus::Pass));
}

#[test]
fn frozen_tonal_acceptance_report_remains_interpretable_for_closeout() {
    let cases = tonal_acceptance_cases();
    let mut detector = KeyDetector::new(KeyDetectorConfig::medium());

    let report =
        run_audio_acceptance_harness(&cases, |audio| detector.analyze(audio), tonal_metrics);

    println!("tonal_acceptance_report={:#?}", report);

    assert_eq!(report.status, AcceptanceStatus::Pass);
    assert_eq!(report.cases.len(), 3);
}
