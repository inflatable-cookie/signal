use super::*;

pub(super) fn build_bar_transition_basic(
    sample_rate: u32,
    bpm: f32,
    variant: BarTransitionVariant,
) -> Option<AudioBuffer> {
    match variant {
        BarTransitionVariant::Pickup => {
            let mut beats = vec![0.45, 0.7];
            for _ in 0..5 {
                beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
            }

            let mut tone_events = Vec::new();
            for (bar_index, chord) in CHORD_CYCLE_ABCD.iter().copied().cycle().take(5).enumerate() {
                tone_events.push((2 + bar_index * 4, chord, 0.82));
            }

            Some(beat_sequence_track(sample_rate, bpm, &beats, &tone_events))
        }
        BarTransitionVariant::PickupExtended => {
            let mut beats = vec![0.32, 0.58, 0.38, 0.68, 0.42, 0.72];
            for _ in 0..6 {
                beats.extend_from_slice(&[1.0, 0.35, 0.55, 0.4]);
            }

            let mut tone_events = Vec::new();
            for (bar_index, chord) in CHORD_CYCLE_ABCD.iter().copied().cycle().take(6).enumerate() {
                tone_events.push((6 + bar_index * 4, chord, 0.84));
            }

            Some(beat_sequence_track(sample_rate, bpm, &beats, &tone_events))
        }
        BarTransitionVariant::LateShift => {
            let mut fixture = FixtureBuilder::new();
            fixture.push_four_four_section(GrooveSection {
                bars: 6,
                beat_pattern: [0.5, 0.26, 0.38, 0.24],
                chord_cycle: CHORD_CYCLE_ABCD,
                chord_every_bars: 1,
                section_marker: Some((10, CHORD_C, 0.9)),
                bar_patterns: Some(LATE_SHIFT_BAR_PATTERNS),
                bar_chords: Some(LATE_SHIFT_BAR_CHORDS),
                dropout_bars: &[],
            });
            Some(fixture.build(sample_rate, bpm))
        }
        BarTransitionVariant::MixedLength => {
            let beats = [
                1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 0.95, 0.38, 0.48, 1.0, 0.35, 0.55, 0.4,
                0.92, 0.38, 0.46, 1.0, 0.35, 0.55, 0.4,
            ];
            let tone_events: &[(usize, &'static [f32], f32)] = &[
                (0, CHORD_A, 0.8),
                (4, CHORD_B, 0.8),
                (8, CHORD_C, 0.82),
                (11, CHORD_D, 0.78),
                (15, CHORD_C, 0.78),
                (18, CHORD_A, 0.8),
            ];
            Some(beat_sequence_track(sample_rate, bpm, &beats, tone_events))
        }
        BarTransitionVariant::Modulation => {
            let beats = [
                1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45, 1.0, 0.42, 0.48, 1.0,
                0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4,
            ];
            let tone_events: &[(usize, &'static [f32], f32)] = &[
                (0, CHORD_A, 0.8),
                (4, CHORD_B, 0.8),
                (8, CHORD_C, 0.82),
                (11, CHORD_D, 0.84),
                (14, CHORD_C, 0.8),
                (18, CHORD_A, 0.82),
            ];
            Some(beat_sequence_track(sample_rate, bpm, &beats, tone_events))
        }
        BarTransitionVariant::CadentialElongation => {
            let beats = [
                1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 0.9, 0.32, 0.48,
                0.38, 0.62, 1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4,
            ];
            let tone_events: &[(usize, &'static [f32], f32)] = &[
                (0, CHORD_A, 0.8),
                (4, CHORD_B, 0.8),
                (8, CHORD_C, 0.82),
                (12, CHORD_D, 0.88),
                (17, CHORD_A, 0.86),
                (21, CHORD_B, 0.82),
            ];
            Some(beat_sequence_track(sample_rate, bpm, &beats, tone_events))
        }
        BarTransitionVariant::ModulationDenseFill => {
            let beats = [
                1.0, 0.35, 0.55, 0.4, 1.0, 0.35, 0.55, 0.4, 1.0, 0.4, 0.45, 1.0, 0.42, 0.48, 1.0,
                0.36, 0.44, 0.92, 0.34, 0.58, 1.0, 0.36, 0.46, 0.96, 0.34, 0.56,
            ];
            let tone_events: &[(usize, &'static [f32], f32)] = &[
                (0, CHORD_A, 0.8),
                (4, CHORD_B, 0.8),
                (8, CHORD_C, 0.86),
                (11, CHORD_D, 0.88),
                (14, CHORD_A, 0.9),
                (18, CHORD_C, 0.9),
                (21, CHORD_D, 0.88),
            ];
            Some(beat_sequence_track(sample_rate, bpm, &beats, tone_events))
        }
        BarTransitionVariant::ModulationDenseFillExtended => {
            let beats = [
                1.0, 0.35, 0.55, 0.4, 0.98, 0.36, 0.44, 0.92, 0.34, 0.58, 1.0, 0.42, 0.48, 0.94,
                0.34, 0.56, 1.0, 0.36, 0.46, 0.96, 0.34, 0.58, 0.9, 0.32, 0.46, 0.4, 0.64, 1.0,
                0.36, 0.46, 0.98, 0.34, 0.6,
            ];
            let tone_events: &[(usize, &'static [f32], f32)] = &[
                (0, CHORD_A, 0.82),
                (4, CHORD_C, 0.86),
                (7, CHORD_B, 0.84),
                (10, CHORD_D, 0.88),
                (13, CHORD_C, 0.9),
                (17, CHORD_A, 0.88),
                (22, CHORD_D, 0.9),
                (27, CHORD_B, 0.86),
            ];
            Some(beat_sequence_track(sample_rate, bpm, &beats, tone_events))
        }
        _ => None,
    }
}
