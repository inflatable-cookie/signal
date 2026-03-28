use signal_analysis::Confidence;

use crate::{
    Key, KeyMode, KeyProfile, LocalTonalTrackingSummary, Tonic, TonalAnalysisResult,
    TonalProfileCandidate, TonalScoringSummary, TuningEstimate, TuningReferenceSource,
};

const STANDARD_TUNING_HZ: f32 = 440.0;
const KRUMHANSL_MAJOR: [f32; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];
const KRUMHANSL_MINOR: [f32; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];
const TEMPERLEY_MAJOR: [f32; 12] = [5.0, 2.0, 3.5, 2.0, 4.5, 4.0, 2.0, 4.5, 2.0, 3.5, 1.5, 4.0];
const TEMPERLEY_MINOR: [f32; 12] = [5.0, 2.0, 3.5, 4.5, 2.0, 4.0, 2.0, 4.5, 3.5, 2.0, 1.5, 4.0];

/// Score a chroma vector against key profiles, returning the best-matching
/// key with its confidence and full correlation set.
pub(crate) fn score_chroma(chroma: [f32; 12], profile: KeyProfile) -> TonalAnalysisResult {
    let correlations = correlate_profiles(chroma, profile);

    let (best_index, best_score) = correlations
        .iter()
        .copied()
        .enumerate()
        .max_by(|(_, lhs), (_, rhs)| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap_or((0, 0.0));

    let second_best = correlations
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| *index != best_index)
        .map(|(_, score)| score)
        .fold(f32::NEG_INFINITY, |best, score| best.max(score));

    let key = if best_score.is_finite() && best_score > second_best && best_score > 0.0 {
        Some(key_from_index(best_index))
    } else {
        None
    };

    let confidence = if best_score > second_best && best_score != 0.0 {
        Confidence::new(((best_score - second_best) / best_score.abs()).max(0.0))
    } else {
        Confidence::new(0.0)
    };

    let best = best_score.is_finite().then_some(TonalProfileCandidate {
        key: key_from_index(best_index),
        correlation: best_score,
    });
    let runner_up = correlations
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| *index != best_index)
        .max_by(|(_, lhs), (_, rhs)| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal))
        .map(|(index, correlation)| TonalProfileCandidate {
            key: key_from_index(index),
            correlation,
        });

    TonalAnalysisResult {
        key,
        confidence,
        tuning: TuningEstimate {
            source: TuningReferenceSource::StandardA440,
            reference_hz: STANDARD_TUNING_HZ,
            cents_offset: 0.0,
            confidence: Confidence::new(1.0),
            score: 0.0,
            runner_up: None,
        },
        chroma,
        correlations,
        scoring: TonalScoringSummary {
            profile,
            best,
            runner_up,
            ambiguity: confidence,
        },
        local_tracking: LocalTonalTrackingSummary {
            window_seconds: 0.0,
            hop_seconds: 0.0,
            segments: Vec::new(),
            changes: Vec::new(),
            ambiguities: Vec::new(),
        },
    }
}

impl TonalAnalysisResult {
    pub(crate) fn with_tuning(mut self, tuning: TuningEstimate) -> Self {
        self.tuning = tuning;
        self
    }

    pub(crate) fn with_local_tracking(mut self, local_tracking: LocalTonalTrackingSummary) -> Self {
        self.local_tracking = local_tracking;
        self
    }
}

/// Compute Pearson correlation coefficients between the observed chroma and
/// all 24 rotated key profiles.
///
/// Pearson correlation centres both vectors around their means before
/// computing the cosine of the angle between them. This is the standard
/// approach in the Krumhansl-Schmuckler key-finding algorithm.
fn correlate_profiles(chroma: [f32; 12], profile: KeyProfile) -> [f32; 24] {
    let (major_profile, minor_profile) = match profile {
        KeyProfile::Krumhansl => (KRUMHANSL_MAJOR, KRUMHANSL_MINOR),
        KeyProfile::Temperley => (TEMPERLEY_MAJOR, TEMPERLEY_MINOR),
    };

    let mut correlations = [0.0; 24];
    for tonic in 0..12 {
        correlations[tonic] = pearson(chroma, rotate_profile(&major_profile, tonic));
        correlations[12 + tonic] = pearson(chroma, rotate_profile(&minor_profile, tonic));
    }
    correlations
}

fn rotate_profile(profile: &[f32; 12], tonic: usize) -> [f32; 12] {
    let mut rotated = [0.0; 12];
    for (index, value) in profile.iter().copied().enumerate() {
        rotated[(index + tonic) % 12] = value;
    }
    rotated
}

fn pearson(x: [f32; 12], y: [f32; 12]) -> f32 {
    let n = 12.0f32;
    let x_mean = x.iter().copied().sum::<f32>() / n;
    let y_mean = y.iter().copied().sum::<f32>() / n;

    let mut numerator = 0.0f32;
    let mut x_var = 0.0f32;
    let mut y_var = 0.0f32;

    for i in 0..12 {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        numerator += dx * dy;
        x_var += dx * dx;
        y_var += dy * dy;
    }

    let denominator = (x_var * y_var).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

fn key_from_index(index: usize) -> Key {
    let tonic = tonic_from_index(index % 12);
    let mode = if index < 12 {
        KeyMode::Major
    } else {
        KeyMode::Minor
    };
    Key { tonic, mode }
}

fn tonic_from_index(index: usize) -> Tonic {
    match index % 12 {
        0 => Tonic::C,
        1 => Tonic::Cs,
        2 => Tonic::D,
        3 => Tonic::Ds,
        4 => Tonic::E,
        5 => Tonic::F,
        6 => Tonic::Fs,
        7 => Tonic::G,
        8 => Tonic::Gs,
        9 => Tonic::A,
        10 => Tonic::As,
        _ => Tonic::B,
    }
}
