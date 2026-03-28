use signal_analysis::Confidence;
use signal_dsp_spectral::{bin_frequency, Spectrogram};

use crate::{
    KeyDetectorConfig, TuningCandidate, TuningEstimate, TuningReferenceMode, TuningReferenceSource,
};

const STANDARD_TUNING_HZ: f32 = 440.0;
const MAX_TUNING_DEVIATION_CENTS: f32 = 50.0;
const SEMITONE_WIDTH_RATIO: f32 = 0.057_762_265;

pub(crate) fn estimate_tuning(spectrogram: &Spectrogram, config: KeyDetectorConfig) -> TuningEstimate {
    match config.tuning_reference {
        TuningReferenceMode::StandardA440 => TuningEstimate {
            source: TuningReferenceSource::StandardA440,
            reference_hz: STANDARD_TUNING_HZ,
            cents_offset: 0.0,
            confidence: Confidence::new(1.0),
            score: 0.0,
            runner_up: None,
        },
        TuningReferenceMode::Fixed(reference_hz) => TuningEstimate {
            source: TuningReferenceSource::FixedReference,
            reference_hz,
            cents_offset: cents_offset_from_standard(reference_hz),
            confidence: Confidence::new(1.0),
            score: 0.0,
            runner_up: None,
        },
        TuningReferenceMode::Estimate => estimate_tuning_reference(spectrogram, config),
    }
}

fn estimate_tuning_reference(
    spectrogram: &Spectrogram,
    config: KeyDetectorConfig,
) -> TuningEstimate {
    let search_cents = config.tuning_search_cents.max(1) as i32;
    let step_cents = config.tuning_step_cents.max(1) as i32;
    let mut best: Option<TuningCandidate> = None;
    let mut runner_up: Option<TuningCandidate> = None;

    for cents in (-search_cents..=search_cents).step_by(step_cents as usize) {
        let cents_offset = cents as f32;
        let reference_hz = reference_hz_from_cents(cents_offset);
        let score = tuning_alignment_score(spectrogram, reference_hz);
        let candidate = TuningCandidate {
            reference_hz,
            cents_offset,
            score,
        };

        let replace_best = match best {
            None => true,
            Some(current) => {
                candidate.score > current.score
                    || ((candidate.score - current.score).abs() < 1.0e-6
                        && candidate.cents_offset.abs() < current.cents_offset.abs())
            }
        };

        if replace_best {
            runner_up = best;
            best = Some(candidate);
        } else if runner_up.is_none_or(|current| candidate.score > current.score) {
            runner_up = Some(candidate);
        }
    }

    let best = best.unwrap_or(TuningCandidate {
        reference_hz: STANDARD_TUNING_HZ,
        cents_offset: 0.0,
        score: 0.0,
    });
    let confidence = if let Some(runner_up) = runner_up {
        if best.score > runner_up.score && best.score > 0.0 {
            Confidence::new(((best.score - runner_up.score) / best.score.abs()).max(0.0))
        } else {
            Confidence::new(0.0)
        }
    } else {
        Confidence::new(0.0)
    };

    TuningEstimate {
        source: TuningReferenceSource::Estimated,
        reference_hz: best.reference_hz,
        cents_offset: best.cents_offset,
        confidence,
        score: best.score,
        runner_up,
    }
}

fn tuning_alignment_score(spectrogram: &Spectrogram, reference_hz: f32) -> f32 {
    let window_size = spectrogram.config.window_size.0;
    if spectrogram.frames.is_empty() || window_size == 0 || spectrogram.sample_rate.0 == 0 {
        return 0.0;
    }

    let bin_spacing = spectrogram.sample_rate.0 as f32 / window_size as f32;
    let min_frequency = bin_spacing / SEMITONE_WIDTH_RATIO;
    let mut score = 0.0f32;

    for frame in &spectrogram.frames {
        for (bin_index, magnitude) in frame.magnitudes.iter().enumerate().skip(1) {
            let frequency = bin_frequency(bin_index, spectrogram.sample_rate, window_size);
            if frequency < min_frequency || frequency > 5_000.0 {
                continue;
            }

            let midi = 69.0 + 12.0 * (frequency / reference_hz.max(1.0)).log2();
            let cents = 100.0 * (midi - midi.round()).abs();
            let closeness = (1.0 - cents / MAX_TUNING_DEVIATION_CENTS).max(0.0);
            score += (magnitude / frequency) * closeness * closeness;
        }
    }

    score
}

fn reference_hz_from_cents(cents_offset: f32) -> f32 {
    STANDARD_TUNING_HZ * 2.0_f32.powf(cents_offset / 1200.0)
}

fn cents_offset_from_standard(reference_hz: f32) -> f32 {
    1200.0 * (reference_hz / STANDARD_TUNING_HZ).log2()
}
