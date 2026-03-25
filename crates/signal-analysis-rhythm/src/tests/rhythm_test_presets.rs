use super::bar_transition_basic::build_bar_transition_basic;
use super::bar_transition_reentry::build_bar_transition_reentry;
use super::bar_transition_reentry_extended::build_bar_transition_reentry_extended;
use super::transition_fixtures::{
    build_dropout_preset, build_fill_transition_preset, DropoutVariant, FillDensityVariant,
};
use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum HarmonicRhythmVariant {
    Sparse,
    Active,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum BarTransitionVariant {
    Pickup,
    PickupExtended,
    LateShift,
    MixedLength,
    Modulation,
    Reentry,
    CadentialElongation,
    ReentryHarmonicShift,
    ReentryDenseFill,
    ReentryAcceleratingHarmony,
    ReentryDeceleratingHarmony,
    ReentryAcceleratingHarmonyDenseFill,
    ReentryDeceleratingHarmonyDenseFill,
    ReentryAcceleratingHarmonyAccentShift,
    ReentryDeceleratingHarmonyAccentShift,
    ReentryAcceleratingHarmonyReset,
    ReentryDeceleratingHarmonyReset,
    ReentryAcceleratingHarmonySustainedReset,
    ReentryAcceleratingHarmonyLongSustainedReset,
    ReentryDeceleratingHarmonySustainedReset,
    ReentryAcceleratingHarmonyCadentialReanchor,
    ReentryDeceleratingHarmonyCadentialReanchor,
    ModulationDenseFill,
    ModulationDenseFillExtended,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum RhythmPreset {
    NeutralClick120,
    StructuredHarmony120(HarmonicRhythmVariant),
    AmbiguousSubdivision90,
    WeakBackbeat118,
    SectionTransition122,
    FillTransition124(FillDensityVariant),
    Dropout120(DropoutVariant),
    BarTransition120(BarTransitionVariant),
}

pub(super) fn build_structured_harmony_preset(
    sample_rate: u32,
    bpm: f32,
    harmonic_rhythm: HarmonicRhythmVariant,
) -> AudioBuffer {
    let (chord_every_bars, section_marker) = match harmonic_rhythm {
        HarmonicRhythmVariant::Sparse => (2, Some((12, CHORD_B, 0.68))),
        HarmonicRhythmVariant::Active => (1, Some((12, CHORD_C, 0.8))),
    };
    let mut fixture = FixtureBuilder::new();
    fixture.push_four_four_section(GrooveSection {
        bars: 6,
        beat_pattern: [0.5, 0.26, 0.38, 0.24],
        chord_cycle: CHORD_CYCLE_ABCD,
        chord_every_bars,
        section_marker,
        bar_patterns: None,
        bar_chords: None,
        dropout_bars: &[],
    });
    fixture.build(sample_rate, bpm)
}

pub(super) fn build_bar_transition_preset(
    sample_rate: u32,
    bpm: f32,
    variant: BarTransitionVariant,
) -> AudioBuffer {
    if let Some(audio) = build_bar_transition_basic(sample_rate, bpm, variant) {
        return audio;
    }
    if let Some(audio) = build_bar_transition_reentry(sample_rate, bpm, variant) {
        return audio;
    }
    if let Some(audio) = build_bar_transition_reentry_extended(sample_rate, bpm, variant) {
        return audio;
    }
    unreachable!("unhandled bar transition variant")
}

pub(super) fn render_preset(preset: RhythmPreset, sample_rate: u32) -> (f32, AudioBuffer) {
    match preset {
        RhythmPreset::NeutralClick120 => (120.0, click_track(sample_rate, 120.0, 8.0)),
        RhythmPreset::StructuredHarmony120(harmonic_rhythm) => (
            120.0,
            build_structured_harmony_preset(sample_rate, 120.0, harmonic_rhythm),
        ),
        RhythmPreset::AmbiguousSubdivision90 => (
            90.0,
            grid_click_track(sample_rate, 90.0, 2, 8.0, &[1.0, 0.3], None),
        ),
        RhythmPreset::WeakBackbeat118 => {
            let bpm = 118.0;
            let mut fixture = FixtureBuilder::new();
            fixture.push_four_four_section(GrooveSection {
                bars: 8,
                beat_pattern: [0.42, 0.24, 0.34, 0.22],
                chord_cycle: CHORD_CYCLE_ABCD,
                chord_every_bars: 2,
                section_marker: None,
                bar_patterns: None,
                bar_chords: None,
                dropout_bars: &[],
            });
            (bpm, fixture.build(sample_rate, bpm))
        }
        RhythmPreset::SectionTransition122 => {
            let bpm = 122.0;
            let mut fixture = FixtureBuilder::new();
            fixture.push_four_four_section(GrooveSection {
                bars: 4,
                beat_pattern: [0.48, 0.22, 0.36, 0.26],
                chord_cycle: CHORD_CYCLE_AB,
                chord_every_bars: 2,
                section_marker: Some((16, CHORD_C, 0.9)),
                bar_patterns: None,
                bar_chords: None,
                dropout_bars: &[],
            });
            fixture.push_four_four_section(GrooveSection {
                bars: 4,
                beat_pattern: [0.55, 0.26, 0.38, 0.28],
                chord_cycle: CHORD_CYCLE_CD,
                chord_every_bars: 2,
                section_marker: None,
                bar_patterns: None,
                bar_chords: None,
                dropout_bars: &[],
            });
            (bpm, fixture.build(sample_rate, bpm))
        }
        RhythmPreset::FillTransition124(density) => (
            124.0,
            build_fill_transition_preset(sample_rate, 124.0, density),
        ),
        RhythmPreset::Dropout120(variant) => {
            (120.0, build_dropout_preset(sample_rate, 120.0, variant))
        }
        RhythmPreset::BarTransition120(variant) => (
            120.0,
            build_bar_transition_preset(sample_rate, 120.0, variant),
        ),
    }
}
