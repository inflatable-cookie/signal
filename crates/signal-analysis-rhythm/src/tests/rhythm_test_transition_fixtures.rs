use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum FillDensityVariant {
    Medium,
    Dense,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum DropoutVariant {
    Light,
    Medium,
    Heavy,
}

pub(super) fn build_fill_transition_preset(
    sample_rate: u32,
    bpm: f32,
    density: FillDensityVariant,
) -> AudioBuffer {
    let (bar_patterns, bar_chords, section_marker) = match density {
        FillDensityVariant::Medium => (FILL_BAR_PATTERNS, FILL_BAR_CHORDS, (16, CHORD_C, 0.85)),
        FillDensityVariant::Dense => (
            DENSE_FILL_BAR_PATTERNS,
            DENSE_FILL_BAR_CHORDS,
            (16, CHORD_D, 0.95),
        ),
    };
    let mut fixture = FixtureBuilder::new();
    fixture.push_four_four_section(GrooveSection {
        bars: 8,
        beat_pattern: [0.46, 0.24, 0.36, 0.24],
        chord_cycle: CHORD_CYCLE_ABCD,
        chord_every_bars: 2,
        section_marker: Some(section_marker),
        bar_patterns: Some(bar_patterns),
        bar_chords: Some(bar_chords),
        dropout_bars: &[],
    });
    fixture.build(sample_rate, bpm)
}

pub(super) fn build_dropout_preset(
    sample_rate: u32,
    bpm: f32,
    variant: DropoutVariant,
) -> AudioBuffer {
    let (bar_patterns, dropout_bars, chord_cycle, chord_every_bars, section_marker) = match variant
    {
        DropoutVariant::Light => (
            Some(LIGHT_DROPOUT_BAR_PATTERNS),
            &[][..],
            CHORD_CYCLE_ABCD,
            2,
            Some((8, CHORD_C, 0.82)),
        ),
        DropoutVariant::Medium => (
            Some(MEDIUM_DROPOUT_BAR_PATTERNS),
            &[3][..],
            CHORD_CYCLE_ABCD,
            2,
            Some((8, CHORD_D, 0.84)),
        ),
        DropoutVariant::Heavy => (
            Some(DROPOUT_BAR_PATTERNS),
            &[1, 3, 5][..],
            &[CHORD_A][..],
            16,
            None,
        ),
    };
    let mut fixture = FixtureBuilder::new();
    fixture.push_four_four_section(GrooveSection {
        bars: 6,
        beat_pattern: [0.48, 0.24, 0.36, 0.24],
        chord_cycle,
        chord_every_bars,
        section_marker,
        bar_patterns,
        bar_chords: None,
        dropout_bars,
    });
    fixture.build(sample_rate, bpm)
}

pub(super) fn build_reentry_transition_fixture(
    sample_rate: u32,
    bpm: f32,
    recovery_sections: &[GrooveSection],
) -> AudioBuffer {
    let mut fixture = FixtureBuilder::new();
    fixture.push_four_four_section(GrooveSection {
        bars: 2,
        beat_pattern: [0.48, 0.24, 0.36, 0.24],
        chord_cycle: CHORD_CYCLE_AB,
        chord_every_bars: 1,
        section_marker: None,
        bar_patterns: None,
        bar_chords: None,
        dropout_bars: &[],
    });
    fixture.push_four_four_section(GrooveSection {
        bars: 2,
        beat_pattern: [0.48, 0.24, 0.36, 0.24],
        chord_cycle: CHORD_CYCLE_CD,
        chord_every_bars: 1,
        section_marker: Some((4, CHORD_A, 1.0)),
        bar_patterns: Some(MEDIUM_DROPOUT_BAR_PATTERNS),
        bar_chords: None,
        dropout_bars: &[0, 1],
    });
    for &section in recovery_sections {
        fixture.push_four_four_section(section);
    }
    fixture.build(sample_rate, bpm)
}
