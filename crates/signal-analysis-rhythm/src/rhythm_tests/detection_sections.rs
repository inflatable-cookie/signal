use super::*;

#[test]
fn beat_tracker_infers_four_four_bar_phase_from_accent_pattern() {
    let bpm = 120.0;
    let audio = grid_click_track(48_000, bpm, 1, 12.0, &[1.0, 0.35, 0.55, 0.4], None);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);
    let meter = result.meter.as_ref().expect("meter estimate");

    assert_eq!(meter.beats_per_bar, 4);
    assert!(
        meter.confidence.0 > 0.2,
        "confidence {}",
        meter.confidence.0
    );
    let bar_seconds = 60.0 / bpm * 4.0;
    assert!(meter
        .downbeat_positions_seconds
        .iter()
        .take(4)
        .all(|downbeat| {
            let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
            (nearest_bar - *downbeat).abs() < 0.08
        }));
}

#[test]
fn beat_tracker_infers_three_four_bar_phase_from_waltz_pattern() {
    let bpm = 120.0;
    let audio = grid_click_track(48_000, bpm, 1, 12.0, &[1.0, 0.4, 0.45], None);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);
    let meter = result.meter.as_ref().expect("meter estimate");

    assert_eq!(meter.beats_per_bar, 3);
    assert!(
        meter.confidence.0 > 0.2,
        "confidence {}",
        meter.confidence.0
    );
    let bar_seconds = 60.0 / bpm * 3.0;
    assert!(meter
        .downbeat_positions_seconds
        .iter()
        .take(4)
        .all(|downbeat| {
            let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
            (nearest_bar - *downbeat).abs() < 0.08
        }));
}

#[test]
fn beat_tracker_infers_four_four_after_two_beat_pickup() {
    let bpm = 120.0;
    let mut beats = vec![0.45, 0.7];
    beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
    beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
    beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);

    let audio = beat_sequence_track(48_000, bpm, &beats, &[]);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);
    let meter = result.meter.as_ref().expect("meter estimate");

    assert_eq!(meter.beats_per_bar, 4);
    assert!(
        meter.confidence.0 > 0.2,
        "confidence {}",
        meter.confidence.0
    );
    let beat_seconds = 60.0 / bpm;
    assert!((meter.downbeat_positions_seconds[0] - 2.0 * beat_seconds).abs() < 0.08);
}

#[test]
fn beat_tracker_uses_spectral_change_to_support_weak_four_four_meter() {
    let bpm = 120.0;
    let beats = [
        0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4, 0.35, 0.45, 0.35, 0.4, 0.35,
    ];
    let tone_events: &[(usize, &'static [f32], f32)] = &[
        (0, &[220.0, 277.18, 329.63], 0.85),
        (4, &[261.63, 329.63, 392.0], 0.85),
        (8, &[196.0, 246.94, 293.66], 0.85),
        (12, &[246.94, 311.13, 369.99], 0.85),
    ];
    let audio = beat_sequence_track(48_000, bpm, &beats, tone_events);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);
    let meter = result.meter.as_ref().expect("meter estimate");

    assert_eq!(meter.beats_per_bar, 4);
    assert!(
        meter.confidence.0 > 0.18,
        "confidence {}",
        meter.confidence.0
    );
}

#[test]
fn beat_tracker_suppresses_meter_on_mixed_bar_lengths() {
    let bpm = 120.0;
    let beats = [
        1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45, 1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45,
    ];
    let audio = beat_sequence_track(48_000, bpm, &beats, &[]);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert!(result.meter.is_none());
}

#[test]
fn beat_tracker_handles_realistic_weak_backbeat_fixture() {
    let bpm = 118.0;
    let mut fixture = FixtureBuilder::new();
    fixture.push_four_four_section(GrooveSection {
        bars: 8,
        beat_pattern: [0.42, 0.24, 0.34, 0.22],
        chord_cycle: &[CHORD_A, CHORD_B, CHORD_C, CHORD_D],
        chord_every_bars: 2,
        section_marker: None,
        bar_patterns: None,
        bar_chords: None,
        dropout_bars: &[],
    });

    let audio = fixture.build(48_000, bpm);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);
    let meter = result.meter.as_ref().expect("meter estimate");

    assert!(
        (result.bpm - bpm).abs() < 3.0,
        "detected bpm {}",
        result.bpm
    );
    assert_eq!(meter.beats_per_bar, 4);
    assert!(
        meter.confidence.0 > 0.2,
        "confidence {}",
        meter.confidence.0
    );
}

#[test]
fn beat_tracker_preserves_four_four_across_section_transition_fixture() {
    let bpm = 122.0;
    let mut fixture = FixtureBuilder::new();
    fixture.push_four_four_section(GrooveSection {
        bars: 4,
        beat_pattern: [0.48, 0.22, 0.36, 0.26],
        chord_cycle: &[CHORD_A, CHORD_B],
        chord_every_bars: 2,
        section_marker: Some((16, CHORD_C, 0.9)),
        bar_patterns: None,
        bar_chords: None,
        dropout_bars: &[],
    });
    fixture.push_four_four_section(GrooveSection {
        bars: 4,
        beat_pattern: [0.55, 0.26, 0.38, 0.28],
        chord_cycle: &[CHORD_C, CHORD_D],
        chord_every_bars: 2,
        section_marker: None,
        bar_patterns: None,
        bar_chords: None,
        dropout_bars: &[],
    });

    let audio = fixture.build(48_000, bpm);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);
    let meter = result.meter.as_ref().expect("meter estimate");

    assert!(
        (result.bpm - bpm).abs() < 3.0,
        "detected bpm {}",
        result.bpm
    );
    assert_eq!(meter.beats_per_bar, 4);
    assert!(
        meter.confidence.0 > 0.2,
        "confidence {}",
        meter.confidence.0
    );
    let bar_seconds = 60.0 / bpm * 4.0;
    assert!(meter
        .downbeat_positions_seconds
        .iter()
        .take(6)
        .all(|downbeat| {
            let nearest_bar = (*downbeat / bar_seconds).round() * bar_seconds;
            (nearest_bar - *downbeat).abs() < 0.09
        }));
}
