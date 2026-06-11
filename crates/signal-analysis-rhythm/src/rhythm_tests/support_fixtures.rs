const CLICK_LENGTH: usize = 64;
const TONE_BURST_LENGTH: usize = 2_048;
const KICK_TONES: &[f32] = &[60.0, 95.0];
const SNARE_TONES: &[f32] = &[220.0, 330.0, 1800.0];
const HAT_TONES: &[f32] = &[4000.0, 6200.0, 8400.0];
const CHORD_A: &[f32] = &[220.0, 277.18, 329.63];
const CHORD_B: &[f32] = &[261.63, 329.63, 392.0];
const CHORD_C: &[f32] = &[196.0, 246.94, 293.66];
const CHORD_D: &[f32] = &[246.94, 311.13, 369.99];
const CHORD_CYCLE_A: &[&[f32]] = &[CHORD_A];
const CHORD_CYCLE_AB: &[&[f32]] = &[CHORD_A, CHORD_B];
const CHORD_CYCLE_ABCD: &[&[f32]] = &[CHORD_A, CHORD_B, CHORD_C, CHORD_D];
const CHORD_CYCLE_CD: &[&[f32]] = &[CHORD_C, CHORD_D];
const FILL_BAR_PATTERNS: &[[f32; 4]] = &[
    [0.46, 0.24, 0.36, 0.24],
    [0.44, 0.22, 0.34, 0.22],
    [0.48, 0.24, 0.38, 0.24],
    [0.36, 0.32, 0.44, 0.62],
    [0.48, 0.24, 0.36, 0.26],
    [0.46, 0.24, 0.36, 0.24],
    [0.5, 0.26, 0.38, 0.24],
    [0.46, 0.24, 0.34, 0.24],
];
const FILL_BAR_CHORDS: &[&[f32]] = &[
    CHORD_A, CHORD_A, CHORD_B, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_D,
];
const DENSE_FILL_BAR_PATTERNS: &[[f32; 4]] = &[
    [0.48, 0.26, 0.38, 0.26],
    [0.42, 0.34, 0.38, 0.28],
    [0.5, 0.28, 0.4, 0.28],
    [0.34, 0.4, 0.48, 0.7],
    [0.5, 0.3, 0.4, 0.3],
    [0.38, 0.34, 0.42, 0.56],
    [0.52, 0.3, 0.4, 0.32],
    [0.36, 0.36, 0.42, 0.66],
];
const DENSE_FILL_BAR_CHORDS: &[&[f32]] = &[
    CHORD_A, CHORD_B, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_D, CHORD_A,
];
const REENTRY_HARMONIC_SHIFT_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_C, CHORD_B, CHORD_D];
const REENTRY_ACCELERATING_STAGE_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_C];
const REENTRY_DECELERATING_STAGE_BAR_CHORDS: &[&[f32]] = &[CHORD_A, CHORD_D];
const REENTRY_ACCELERATING_DENSE_BAR_PATTERNS: &[[f32; 4]] =
    &[[0.54, 0.28, 0.4, 0.3], [0.5, 0.32, 0.42, 0.32]];
const REENTRY_DECELERATING_DENSE_BAR_PATTERNS: &[[f32; 4]] =
    &[[0.58, 0.3, 0.44, 0.32], [0.54, 0.34, 0.46, 0.36]];
const REENTRY_ACCELERATING_ACCENT_SHIFT_BAR_PATTERNS: &[[f32; 4]] =
    &[[0.28, 0.66, 0.3, 0.58], [0.26, 0.62, 0.3, 0.6]];
const REENTRY_DECELERATING_ACCENT_SHIFT_BAR_PATTERNS: &[[f32; 4]] =
    &[[0.3, 0.6, 0.28, 0.62], [0.28, 0.64, 0.3, 0.58]];
const REENTRY_HARMONIC_RESET_BAR_PATTERNS: &[[f32; 4]] =
    &[[0.56, 0.24, 0.4, 0.24], [0.58, 0.24, 0.42, 0.24]];
const REENTRY_SUSTAINED_RESET_BAR_PATTERNS: &[[f32; 4]] = &[
    [0.6, 0.24, 0.42, 0.24],
    [0.62, 0.24, 0.44, 0.24],
    [0.64, 0.22, 0.44, 0.22],
    [0.62, 0.24, 0.42, 0.24],
    [0.66, 0.22, 0.46, 0.22],
    [0.64, 0.24, 0.44, 0.24],
];
const REENTRY_CADENTIAL_REANCHOR_BAR_PATTERNS: &[[f32; 4]] =
    &[[0.72, 0.22, 0.44, 0.24], [0.64, 0.24, 0.42, 0.24]];
const REENTRY_CADENTIAL_REANCHOR_BAR_CHORDS: &[&[f32]] = &[CHORD_D, CHORD_A];
const LATE_SHIFT_BAR_PATTERNS: &[[f32; 4]] = &[
    [0.5, 0.26, 0.38, 0.24],
    [0.48, 0.24, 0.36, 0.24],
    [0.28, 0.72, 0.34, 0.22],
    [0.52, 0.28, 0.38, 0.24],
    [0.48, 0.24, 0.36, 0.24],
    [0.5, 0.26, 0.38, 0.24],
];
const LATE_SHIFT_BAR_CHORDS: &[&[f32]] =
    &[CHORD_A, CHORD_B, CHORD_C, CHORD_C, CHORD_D, CHORD_A];
const LIGHT_DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
    [0.48, 0.24, 0.36, 0.24],
    [0.48, 0.24, 0.36, 0.24],
    [0.3, 0.12, 0.24, 0.12],
    [0.5, 0.24, 0.38, 0.24],
    [0.46, 0.22, 0.34, 0.22],
    [0.48, 0.24, 0.36, 0.24],
];
const MEDIUM_DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
    [0.48, 0.24, 0.36, 0.24],
    [0.24, 0.1, 0.18, 0.08],
    [0.5, 0.24, 0.38, 0.24],
    [0.22, 0.08, 0.16, 0.08],
    [0.46, 0.22, 0.34, 0.22],
    [0.48, 0.24, 0.36, 0.24],
];
const DROPOUT_BAR_PATTERNS: &[[f32; 4]] = &[
    [0.48, 0.24, 0.36, 0.24],
    [0.04, 0.0, 0.03, 0.0],
    [0.5, 0.24, 0.38, 0.24],
    [0.03, 0.0, 0.02, 0.0],
    [0.46, 0.22, 0.34, 0.22],
    [0.02, 0.0, 0.02, 0.0],
];

#[derive(Clone, Copy)]
struct GrooveSection {
    bars: usize,
    beat_pattern: [f32; 4],
    chord_cycle: &'static [&'static [f32]],
    chord_every_bars: usize,
    section_marker: Option<(usize, &'static [f32], f32)>,
    bar_patterns: Option<&'static [[f32; 4]]>,
    bar_chords: Option<&'static [&'static [f32]]>,
    dropout_bars: &'static [usize],
}

#[derive(Default)]
struct FixtureBuilder {
    beats: Vec<f32>,
    tone_events: Vec<(usize, &'static [f32], f32)>,
}

impl FixtureBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn beat_len(&self) -> usize {
        self.beats.len()
    }

    fn push_four_four_section(&mut self, section: GrooveSection) {
        let start_beat = self.beat_len();
        push_four_four_groove(&mut self.beats, &mut self.tone_events, start_beat, section);
    }

    fn build(self, sample_rate: u32, bpm: f32) -> AudioBuffer {
        beat_sequence_track(sample_rate, bpm, &self.beats, &self.tone_events)
    }
}

fn analyze_fixture(audio: &AudioBuffer) -> super::BeatAnalysisResult {
    let mut tracker = BeatTracker::new(BeatTrackerConfig::default());
    tracker.analyze(audio)
}

fn analyze_preset(preset: RhythmPreset) -> (f32, super::BeatAnalysisResult) {
    let sample_rate = 48_000;
    let (bpm, audio) = render_preset(preset, sample_rate);
    (bpm, analyze_fixture(&audio))
}

fn rhythm_metrics(result: &super::BeatAnalysisResult) -> Vec<AnalysisMetricValue> {
    let structure = result.rhythm_structure_summary();
    let meter = result.meter.as_ref();
    let structure = structure.as_ref();
    let recovery_window_available = meter
        .and_then(|estimate| estimate.recovery.as_ref())
        .is_some()
        || result.structure_ambiguity.trailing_recovery_confidence.0 > 0.0;

    vec![
        AnalysisMetricValue::new("bpm", result.bpm),
        AnalysisMetricValue::new("confidence", result.confidence.0),
        AnalysisMetricValue::new("tempo_ambiguity", result.tempo_ambiguity.0),
        AnalysisMetricValue::new("has_meter", if meter.is_some() { 1.0 } else { 0.0 }),
        AnalysisMetricValue::new(
            "beats_per_bar",
            meter
                .map(|estimate| estimate.beats_per_bar as f32)
                .unwrap_or(0.0),
        ),
        AnalysisMetricValue::new(
            "meter_confidence",
            meter.map(|estimate| estimate.confidence.0).unwrap_or(0.0),
        ),
        AnalysisMetricValue::new(
            "structure_bar_count",
            structure
                .map(|summary| summary.bar_count as f32)
                .unwrap_or(0.0),
        ),
        AnalysisMetricValue::new(
            "recovered_bar_count",
            structure
                .map(|summary| summary.recovered_bar_count as f32)
                .unwrap_or(0.0),
        ),
        AnalysisMetricValue::new(
            "recovery_window_available",
            if recovery_window_available {
                1.0
            } else {
                0.0
            },
        ),
    ]
}

