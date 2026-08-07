//! Pitch shifting and offline short-window path selection.

use signal_dsp_resample::{resample_mono, ResampleConfig, ResampleQuality};
use signal_primitives::{Sample, SampleRate};

use crate::stretch_backend::{
    OfflineHighQualityPath, PhaseVocoderStretcher, TimeStretcher,
    COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
    COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES,
    COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP, DEFAULT_WINDOW_SIZE,
    EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES,
    EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
};
use crate::transient_smear;

pub(crate) fn pitch_shift_mono_to_nominal_rate(
    input: &[Sample],
    sample_rate: SampleRate,
    semitones: f64,
) -> Vec<Sample> {
    let Some(config) = pitch_shift_resample_config(sample_rate, semitones) else {
        return input.to_vec();
    };
    resample_mono(config, input)
}

pub(crate) fn pitch_shift_interleaved_stereo_to_nominal_rate(
    input: &[Sample],
    sample_rate: SampleRate,
    semitones: f64,
) -> Vec<Sample> {
    let frame_count = input.len() / 2;
    let Some(config) = pitch_shift_resample_config(sample_rate, semitones) else {
        return input[..frame_count * 2].to_vec();
    };

    let mut mid = Vec::with_capacity(frame_count);
    let mut side = Vec::with_capacity(frame_count);
    for frame in input[..frame_count * 2].chunks_exact(2) {
        let left = frame[0];
        let right = frame[1];
        mid.push((left + right) * 0.5);
        side.push((left - right) * 0.5);
    }
    let mid = resample_mono(config, &mid);
    let side = resample_mono(config, &side);
    let out_frames = mid.len().min(side.len());
    let mut output = Vec::with_capacity(out_frames * 2);
    for index in 0..out_frames {
        output.push(mid[index] + side[index]);
        output.push(mid[index] - side[index]);
    }
    output
}

pub(crate) fn pitch_shift_resample_config(
    sample_rate: SampleRate,
    semitones: f64,
) -> Option<ResampleConfig> {
    if sample_rate.0 == 0 || !semitones.is_finite() || semitones.abs() < 1.0e-9 {
        return None;
    }
    let factor = 2.0f64.powf(semitones / 12.0);
    if !factor.is_finite() || factor <= 0.0 {
        return None;
    }
    let virtual_input_rate =
        ((sample_rate.0 as f64 * factor).round()).clamp(1.0, u32::MAX as f64) as u32;
    Some(ResampleConfig::new(
        SampleRate(virtual_input_rate),
        sample_rate,
        ResampleQuality::BandLimited,
    ))
}

pub(crate) fn should_select_compression_short_window(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    if ratio >= 1.0 || input.is_empty() || current_output.is_empty() {
        return false;
    }

    let current_smear = transient_smear::measure_selector_transient_smear(
        input,
        current_output,
        ratio,
        COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    current_smear.missed_transients >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES
        || current_smear.max_smear_frames
            >= COMPRESSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_SMEAR_FRAMES
}

pub(crate) fn should_select_compression_short_window_interleaved(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    let input_mono = downmix_interleaved_stereo_to_mono(input);
    let output_mono = downmix_interleaved_stereo_to_mono(current_output);
    should_select_compression_short_window(&input_mono, &output_mono, ratio)
}

pub(crate) fn should_select_expansion_short_window(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    if ratio <= 1.0 || input.is_empty() || current_output.is_empty() {
        return false;
    }

    // Source transients are detected once and reused for both comparisons.
    // The current-output and draft-baseline measurements previously each
    // re-detected them from the same input with the same policy and geometry.
    let input_events = transient_smear::detect_stretch_transients_with_policy(
        input,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
        transient_smear::StretchTransientDetectorPolicy::production(),
    );
    let current_smear = transient_smear::measure_selector_transient_smear_with_input_events(
        input,
        &input_events,
        current_output,
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    if current_smear.missed_transients >= EXPANSION_SHORT_WINDOW_SELECTOR_MIN_CURRENT_MISSES {
        return true;
    }

    let mut draft = PhaseVocoderStretcher::new(ratio);
    let Ok(draft_output) = draft.stretch_mono(input) else {
        // The default render already succeeded at this size, so a draft render
        // of the same input cannot exceed the bound. Stay on the current path
        // rather than switching on missing evidence.
        return false;
    };
    let draft_smear = transient_smear::measure_selector_transient_smear_with_input_events(
        input,
        &input_events,
        &draft_output,
        ratio,
        EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE,
        EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP,
    );
    metric_worsened(current_smear.max_smear_frames, draft_smear.max_smear_frames)
}

pub(crate) fn should_select_expansion_short_window_interleaved(
    input: &[Sample],
    current_output: &[Sample],
    ratio: f64,
) -> bool {
    let input_mono = downmix_interleaved_stereo_to_mono(input);
    let output_mono = downmix_interleaved_stereo_to_mono(current_output);
    should_select_expansion_short_window(&input_mono, &output_mono, ratio)
}

pub(crate) fn metric_worsened(candidate: f64, production: f64) -> bool {
    if candidate.is_finite() && production.is_finite() {
        candidate > production
    } else {
        !candidate.is_finite() && production.is_finite()
    }
}

pub(crate) fn short_window_size_for_path(path: OfflineHighQualityPath) -> usize {
    match path {
        OfflineHighQualityPath::Default => DEFAULT_WINDOW_SIZE,
        OfflineHighQualityPath::CompressionShortWindowSelector => {
            COMPRESSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE
        }
        OfflineHighQualityPath::ExpansionShortWindowSelector => {
            EXPANSION_SHORT_WINDOW_SELECTOR_WINDOW_SIZE
        }
    }
}

pub(crate) fn short_window_analysis_hop_for_path(path: OfflineHighQualityPath) -> usize {
    match path {
        OfflineHighQualityPath::Default => DEFAULT_ANALYSIS_HOP,
        OfflineHighQualityPath::CompressionShortWindowSelector => {
            COMPRESSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP
        }
        OfflineHighQualityPath::ExpansionShortWindowSelector => {
            EXPANSION_SHORT_WINDOW_SELECTOR_ANALYSIS_HOP
        }
    }
}

pub(crate) fn downmix_interleaved_stereo_to_mono(samples: &[Sample]) -> Vec<Sample> {
    samples
        .chunks_exact(2)
        .map(|frame| (frame[0] + frame[1]) * 0.5)
        .collect()
}
