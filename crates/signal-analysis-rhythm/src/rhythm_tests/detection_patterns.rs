use super::*;

#[test]
fn beat_tracker_exposes_non_empty_onset_envelope_for_click_track() {
    let audio = click_track(48_000, 120.0, 4.0);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert!(!result.onset_envelope.is_empty());
    assert!(result.onset_envelope.iter().any(|value| *value > 0.5));
}

#[test]
fn beat_tracker_returns_zero_for_silence() {
    let audio = AudioBuffer::new(
        SampleRate(48_000),
        ChannelLayout::Mono,
        signal_primitives::FrameCount(48_000),
    );
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert_eq!(result.bpm, 0.0);
    assert_eq!(result.confidence.0, 0.0);
    assert!(result.beat_positions_seconds.is_empty());
    assert!(result.tempo_candidates.is_empty());
    assert_eq!(result.tempo_ambiguity.0, 0.0);
    assert!(result.meter.is_none());
}

#[test]
fn beat_tracker_detects_swung_click_track_tempo() {
    let audio = grid_click_track(
        48_000,
        120.0,
        2,
        8.0,
        &[1.0, 0.45, 0.85, 0.35],
        Some(2.0 / 3.0),
    );
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert!(
        (result.bpm - 120.0).abs() < 3.0,
        "detected bpm {}",
        result.bpm
    );
    assert!(
        result.confidence.0 > 0.15,
        "confidence {}",
        result.confidence.0
    );
}

#[test]
fn beat_tracker_handles_syncopated_pattern_without_halving_tempo() {
    let audio = grid_click_track(
        48_000,
        120.0,
        2,
        8.0,
        &[1.0, 0.0, 0.35, 0.8, 0.95, 0.0, 0.3, 0.75],
        None,
    );
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert!(
        (result.bpm - 120.0).abs() < 3.5,
        "detected bpm {}",
        result.bpm
    );
}

#[test]
fn beat_tracker_prefers_base_tempo_over_double_time_subdivisions() {
    let audio = grid_click_track(48_000, 90.0, 2, 8.0, &[1.0, 0.3], None);
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert!(
        (result.bpm - 90.0).abs() < 3.0,
        "detected bpm {}",
        result.bpm
    );
    assert!(
        result.confidence.0 > 0.1,
        "confidence {}",
        result.confidence.0
    );
    assert!(result.tempo_candidates.len() >= 2);
    assert!(result
        .tempo_candidates
        .iter()
        .skip(1)
        .any(|candidate| (candidate.bpm - 180.0).abs() < 4.0));
    assert!(result.tempo_ambiguity.0 > 0.2);
}

#[test]
fn beat_tracker_selects_consistent_phase_over_single_loud_offbeat() {
    let sample_rate = 48_000;
    let bpm = 120.0;
    let mut audio = click_track(sample_rate, bpm, 8.0);
    let offbeat_index = (60.0 / bpm * sample_rate as f32 / 2.0).round() as usize;
    add_click(audio.samples_mut(), offbeat_index, 1.25);

    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    let result = tracker.analyze(&audio);

    assert!(
        (result.bpm - 120.0).abs() < 3.0,
        "detected bpm {}",
        result.bpm
    );
    let quarter_note_seconds = 60.0 / bpm;
    assert!(result.beat_positions_seconds.iter().take(6).all(|beat| {
        let nearest_grid = (*beat / quarter_note_seconds).round() * quarter_note_seconds;
        (nearest_grid - *beat).abs() < 0.08
    }));
}
