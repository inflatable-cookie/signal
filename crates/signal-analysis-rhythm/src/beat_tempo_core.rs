use rayon::prelude::*;
use signal_analysis::Confidence;
use signal_primitives::SampleRate;

use crate::tempo_policy::{filter_interval_outliers, median};
use crate::{beat_phase_score, select_beat_phase, TempoEstimate, TempoHypothesis};

#[path = "beat_tempo_core/refinement.rs"]
mod refinement;

pub(crate) use refinement::{
    beat_frames_to_seconds, beat_frames_to_seconds_refined, combined_confidence,
    refine_beat_frames, track_beats,
};

pub(crate) fn estimate_tempo(
    onset_envelope: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
    min_bpm: f32,
    max_bpm: f32,
) -> TempoEstimate {
    if onset_envelope.len() < 2 || sample_rate.0 == 0 || hop_size == 0 {
        return TempoEstimate {
            bpm: 0.0,
            confidence: Confidence::new(0.0),
            lag_frames: 0,
            phase_offset_frames: 0,
            candidates: [None, None, None],
            ambiguity: Confidence::new(0.0),
        };
    }

    let onset_rate = sample_rate.0 as f32 / hop_size as f32;
    let min_lag = ((60.0 * onset_rate) / max_bpm).round().max(1.0) as usize;
    let max_lag = ((60.0 * onset_rate) / min_bpm).round().max(min_lag as f32) as usize;
    let max_lag = max_lag.min(onset_envelope.len().saturating_sub(1));
    let mut lag_scores = vec![0.0; max_lag + 1];

    lag_scores[min_lag..=max_lag]
        .par_iter_mut()
        .enumerate()
        .for_each(|(offset, score)| *score = tempo_score(onset_envelope, min_lag + offset));

    let candidates = tempo_candidates(&lag_scores, min_lag, max_lag);
    let mut hypotheses = Vec::new();

    for lag in candidates.into_iter().take(6) {
        let raw_score = lag_scores[lag];
        if raw_score <= 0.0 {
            continue;
        }

        let refined_lag = refine_tempo_lag(&lag_scores, lag, min_lag, max_lag);
        let phase_offset = select_beat_phase(onset_envelope, lag);
        let phase_score = beat_phase_score(onset_envelope, lag, phase_offset);
        let hypothesis_score = raw_score * (0.7 + 0.3 * phase_score.clamp(0.0, 1.0));
        hypotheses.push(TempoHypothesis {
            bpm: 60.0 * onset_rate / refined_lag.max(1.0),
            lag_frames: lag,
            refined_lag_frames: refined_lag,
            phase_offset_frames: phase_offset,
            phase_score,
            score: hypothesis_score,
            confidence: Confidence::new(0.0),
        });
    }

    hypotheses.sort_by(|lhs, rhs| {
        rhs.score
            .partial_cmp(&lhs.score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    if hypotheses.is_empty() {
        return TempoEstimate {
            bpm: 0.0,
            confidence: Confidence::new(0.0),
            lag_frames: 0,
            phase_offset_frames: 0,
            candidates: [None, None, None],
            ambiguity: Confidence::new(0.0),
        };
    }

    let best_score = hypotheses[0].score;
    let runner_up = hypotheses
        .get(1)
        .map(|candidate| candidate.score)
        .unwrap_or(0.0);

    for hypothesis in &mut hypotheses {
        let score_ratio = if best_score > 0.0 {
            hypothesis.score / best_score
        } else {
            0.0
        };
        hypothesis.confidence = Confidence::new(0.7 * score_ratio + 0.3 * hypothesis.phase_score);
    }

    let best_candidate = hypotheses[0];
    let ambiguity = if best_score > 0.0 {
        let runner_ratio = runner_up / best_score;
        let relation_bonus = hypotheses
            .get(1)
            .map(|candidate| {
                let ratio = best_candidate.bpm / candidate.bpm.max(1.0);
                if (ratio - 2.0).abs() < 0.18
                    || (ratio - 0.5).abs() < 0.09
                    || (ratio - 1.5).abs() < 0.12
                {
                    0.2
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        Confidence::new((runner_ratio + relation_bonus).min(1.0))
    } else {
        Confidence::new(0.0)
    };

    TempoEstimate {
        bpm: best_candidate.bpm,
        confidence: if best_score > 0.0 {
            let margin = (best_score - runner_up).max(0.0) / best_score;
            Confidence::new(0.65 * margin + 0.35 * best_candidate.phase_score)
        } else {
            Confidence::new(0.0)
        },
        lag_frames: best_candidate.lag_frames,
        phase_offset_frames: best_candidate.phase_offset_frames,
        candidates: [
            hypotheses.first().copied(),
            hypotheses.get(1).copied(),
            hypotheses.get(2).copied(),
        ],
        ambiguity,
    }
}

fn tempo_candidates(lag_scores: &[f32], min_lag: usize, max_lag: usize) -> Vec<usize> {
    let mut candidates = Vec::new();

    for lag in min_lag..=max_lag {
        let score = lag_scores[lag];
        if score <= 0.0 {
            continue;
        }

        let previous = if lag > min_lag {
            lag_scores[lag - 1]
        } else {
            0.0
        };
        let next = if lag < max_lag {
            lag_scores[lag + 1]
        } else {
            0.0
        };
        if score >= previous && score >= next {
            candidates.push(lag);
        }
    }

    candidates.sort_by(|lhs, rhs| {
        lag_scores[*rhs]
            .partial_cmp(&lag_scores[*lhs])
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    let mut filtered = Vec::new();
    for lag in candidates {
        if filtered
            .iter()
            .all(|existing: &usize| existing.abs_diff(lag) > 2)
        {
            filtered.push(lag);
        }
    }

    filtered
}

fn refine_tempo_lag(lag_scores: &[f32], lag: usize, min_lag: usize, max_lag: usize) -> f32 {
    if lag <= min_lag || lag >= max_lag {
        return lag as f32;
    }

    let left = lag_scores[lag - 1];
    let center = lag_scores[lag];
    let right = lag_scores[lag + 1];
    let denominator = left - 2.0 * center + right;

    if denominator.abs() <= f32::EPSILON {
        return lag as f32;
    }

    let delta = (0.5 * (left - right) / denominator).clamp(-0.5, 0.5);
    lag as f32 + delta
}

fn tempo_score(onset_envelope: &[f32], lag: usize) -> f32 {
    if lag == 0 || lag >= onset_envelope.len() {
        return 0.0;
    }

    let base = autocorrelation(onset_envelope, lag);
    let second = autocorrelation(onset_envelope, lag * 2) * 0.5;
    let third = autocorrelation(onset_envelope, lag * 3) * 0.25;
    base + second + third
}

fn autocorrelation(onset_envelope: &[f32], lag: usize) -> f32 {
    if lag == 0 || lag >= onset_envelope.len() {
        return 0.0;
    }

    let mut score = 0.0;
    for index in lag..onset_envelope.len() {
        score += onset_envelope[index] * onset_envelope[index - lag];
    }
    score
}

pub(crate) fn refine_bpm_from_beats(
    coarse_bpm: f32,
    beat_frames: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
) -> f32 {
    if coarse_bpm <= 0.0 || sample_rate.0 == 0 || hop_size == 0 || beat_frames.len() < 2 {
        return coarse_bpm.max(0.0);
    }

    let intervals: Vec<f32> = beat_frames
        .windows(2)
        .filter_map(|pair| {
            let interval = pair[1] - pair[0];
            (interval > 0.0).then_some(interval)
        })
        .collect();
    if intervals.is_empty() {
        return coarse_bpm;
    }
    let (filtered, diagnostics) = filter_interval_outliers(&intervals);
    let intervals = if filtered.len() >= 4 {
        filtered
    } else if diagnostics.median_interval > 0.0 {
        vec![diagnostics.median_interval]
    } else {
        intervals
    };
    let average_interval = intervals.iter().copied().sum::<f32>() / intervals.len() as f32;
    if average_interval <= 0.0 {
        return coarse_bpm;
    }

    let onset_rate = sample_rate.0 as f32 / hop_size as f32;
    let beat_grid_bpm = 60.0 * onset_rate / average_interval;
    let interval_median = if intervals.is_empty() {
        0.0
    } else {
        let mut median_intervals = intervals.clone();
        median(&mut median_intervals)
    };
    let mean_abs_deviation = intervals
        .iter()
        .map(|interval| (*interval - interval_median).abs())
        .sum::<f32>()
        / intervals.len() as f32;
    let consistency =
        (1.0 - (mean_abs_deviation / interval_median.max(1.0)) / 0.02).clamp(0.0, 1.0);
    let mismatch = (beat_grid_bpm - coarse_bpm).abs();
    let agreement = (1.0 - mismatch / 0.6).clamp(0.0, 1.0);
    let correction_strength = (consistency * agreement).clamp(0.0, 1.0);
    coarse_bpm + (beat_grid_bpm - coarse_bpm) * correction_strength
}
