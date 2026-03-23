use signal_analysis::Confidence;

use crate::MeterConfidenceBreakdown;

#[derive(Clone, Copy)]
pub(crate) struct MeterHypothesis {
    pub beats_per_bar: usize,
    pub phase_offset_beats: usize,
    pub score: f32,
    pub support_ratio: f32,
    pub meter_support_ratio: f32,
    pub meter_contrast_mean: f32,
    pub regularity: f32,
    pub recent_strength: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct MeterWindowCandidate {
    pub start_beat: usize,
    pub end_beat: usize,
    pub hypothesis: MeterHypothesis,
    pub confidence: Confidence,
    pub confidence_breakdown: MeterConfidenceBreakdown,
    pub supporting_windows: usize,
}

pub(crate) fn meter_hypotheses(
    beat_strengths: &[f32],
    meter_strengths: &[f32],
) -> Vec<MeterHypothesis> {
    let mut hypotheses = Vec::new();

    for beats_per_bar in [3usize, 4usize] {
        for phase_offset_beats in 0..beats_per_bar {
            let mut bars = 0usize;
            let total_bars =
                (beat_strengths.len().saturating_sub(phase_offset_beats)) / beats_per_bar;
            let mut supported_weight = 0.0f32;
            let mut meter_supported_weight = 0.0f32;
            let mut downbeat_sum = 0.0f32;
            let mut weakbeat_sum = 0.0f32;
            let mut contrast_sum = 0.0f32;
            let mut meter_contrast_sum = 0.0f32;
            let mut weight_sum = 0.0f32;
            let mut bar_strengths = Vec::new();

            let mut index = phase_offset_beats;
            while index + beats_per_bar <= beat_strengths.len() {
                let bar = &beat_strengths[index..index + beats_per_bar];
                let meter_bar = &meter_strengths[index..index + beats_per_bar];
                let downbeat = 0.55 * bar[0] + 0.45 * meter_bar[0];
                let weakbeat_mean = if beats_per_bar > 1 {
                    let onset_mean =
                        bar[1..].iter().copied().sum::<f32>() / (beats_per_bar - 1) as f32;
                    let meter_mean =
                        meter_bar[1..].iter().copied().sum::<f32>() / (beats_per_bar - 1) as f32;
                    0.6 * onset_mean + 0.4 * meter_mean
                } else {
                    0.0
                };
                let contrast = (downbeat - weakbeat_mean).max(0.0);
                let meter_contrast = (meter_bar[0]
                    - if beats_per_bar > 1 {
                        meter_bar[1..].iter().copied().sum::<f32>() / (beats_per_bar - 1) as f32
                    } else {
                        0.0
                    })
                .max(0.0);
                let progress = if total_bars > 1 {
                    bars as f32 / (total_bars - 1) as f32
                } else {
                    1.0
                };
                let weight = 0.65 + 0.35 * progress;

                bars += 1;
                if contrast > 0.06 || meter_contrast > 0.08 {
                    supported_weight += weight;
                }
                if meter_contrast > 0.08 {
                    meter_supported_weight += weight;
                }
                downbeat_sum += downbeat * weight;
                weakbeat_sum += weakbeat_mean * weight;
                contrast_sum += contrast * weight;
                meter_contrast_sum += meter_contrast * weight;
                weight_sum += weight;
                bar_strengths.push(0.7 * contrast + 0.3 * meter_contrast);
                index += beats_per_bar;
            }

            if bars < 2 || weight_sum <= 0.0 {
                continue;
            }

            let downbeat_mean = downbeat_sum / weight_sum;
            let weakbeat_mean = weakbeat_sum / weight_sum;
            let contrast_mean = contrast_sum / weight_sum;
            let meter_contrast_mean = meter_contrast_sum / weight_sum;
            let support_ratio = supported_weight / weight_sum;
            let meter_support_ratio = meter_supported_weight / weight_sum;
            let coverage = (bars as f32 / 4.0).clamp(0.0, 1.0);
            let mean_bar_strength =
                bar_strengths.iter().copied().sum::<f32>() / bar_strengths.len() as f32;
            let regularity = if mean_bar_strength > 0.0 {
                let deviation = bar_strengths
                    .iter()
                    .copied()
                    .map(|strength| (strength - mean_bar_strength).abs())
                    .sum::<f32>()
                    / bar_strengths.len() as f32;
                (1.0 - deviation / mean_bar_strength).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let recent_strength = if bar_strengths.is_empty() {
                0.0
            } else {
                let tail = bar_strengths.len().min(2);
                bar_strengths[bar_strengths.len() - tail..]
                    .iter()
                    .copied()
                    .sum::<f32>()
                    / tail as f32
            };
            let score = (0.38 * contrast_mean
                + 0.22 * meter_contrast_mean
                + 0.15 * (downbeat_mean - weakbeat_mean).max(0.0)
                + 0.06 * support_ratio
                + 0.04 * meter_support_ratio
                + 0.07 * regularity
                + 0.08 * recent_strength)
                * coverage;

            hypotheses.push(MeterHypothesis {
                beats_per_bar,
                phase_offset_beats,
                score,
                support_ratio,
                meter_support_ratio,
                meter_contrast_mean,
                regularity,
                recent_strength,
            });
        }
    }

    hypotheses.sort_by(|lhs, rhs| {
        rhs.score
            .partial_cmp(&lhs.score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    hypotheses
}

pub(crate) fn meter_confidence_breakdown(
    best: MeterHypothesis,
    runner_up_score: f32,
) -> MeterConfidenceBreakdown {
    let margin = if best.score > 0.0 {
        (best.score - runner_up_score).max(0.0) / best.score
    } else {
        0.0
    };
    let salience = (best.score / 0.35).clamp(0.0, 1.0);
    MeterConfidenceBreakdown {
        phase_margin: margin,
        support: best.support_ratio,
        meter_support: best.meter_support_ratio,
        regularity: best.regularity,
        recent_stability: best.recent_strength,
        salience,
    }
}

pub(crate) fn meter_hypothesis_confidence(
    best: MeterHypothesis,
    runner_up_score: f32,
) -> Confidence {
    if best.score <= 0.0 {
        return Confidence::new(0.0);
    }

    let breakdown = meter_confidence_breakdown(best, runner_up_score);
    Confidence::new(
        (0.38 * breakdown.phase_margin
            + 0.18 * breakdown.support
            + 0.09 * breakdown.meter_support
            + 0.07 * best.meter_contrast_mean.clamp(0.0, 1.0)
            + 0.15 * breakdown.regularity
            + 0.20 * breakdown.recent_stability)
            * breakdown.salience,
    )
}

pub(crate) fn meter_window_candidate(
    beat_strengths: &[f32],
    meter_strengths: &[f32],
    start_beat: usize,
    end_beat: usize,
) -> Option<MeterWindowCandidate> {
    if end_beat <= start_beat || end_beat > beat_strengths.len() || end_beat > meter_strengths.len()
    {
        return None;
    }

    let hypotheses = meter_hypotheses(
        &beat_strengths[start_beat..end_beat],
        &meter_strengths[start_beat..end_beat],
    );
    let hypothesis = hypotheses.first().copied()?;
    let runner_up = hypotheses
        .get(1)
        .map(|candidate| candidate.score)
        .unwrap_or(0.0);
    let confidence_breakdown = meter_confidence_breakdown(hypothesis, runner_up);

    Some(MeterWindowCandidate {
        start_beat,
        end_beat,
        hypothesis,
        confidence: meter_hypothesis_confidence(hypothesis, runner_up),
        confidence_breakdown,
        supporting_windows: 1,
    })
}
