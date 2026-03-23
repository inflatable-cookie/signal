use signal_analysis::Confidence;
use signal_primitives::SampleRate;

use crate::rhythm_policy::{meter_window_candidate, MeterHypothesis, MeterWindowCandidate};
use crate::{MeterConfidenceBreakdown, MeterRecoveryContext};

pub(crate) fn window_absolute_phase(window: MeterWindowCandidate) -> usize {
    (window.start_beat + window.hypothesis.phase_offset_beats) % window.hypothesis.beats_per_bar
}

pub(crate) fn window_phase_offset(
    beats_per_bar: usize,
    absolute_phase: usize,
    start_beat: usize,
) -> usize {
    (absolute_phase + beats_per_bar - (start_beat % beats_per_bar)) % beats_per_bar
}

pub(crate) fn window_is_recoverable(window: MeterWindowCandidate) -> bool {
    window.hypothesis.score >= 0.17
        && window.confidence.0 >= 0.24
        && window.hypothesis.support_ratio >= 0.72
        && window.hypothesis.regularity >= 0.56
        && window.hypothesis.recent_strength >= 0.12
}

pub(crate) fn select_segment_meter_candidate(
    beat_strengths: &[f32],
    meter_strengths: &[f32],
) -> Option<MeterWindowCandidate> {
    if beat_strengths.len() < 12 || meter_strengths.len() < 12 {
        return None;
    }

    let total_beats = beat_strengths.len().min(meter_strengths.len());
    let mut windows = Vec::new();

    for beat_count in [8usize, 12, 16] {
        if beat_count > total_beats {
            continue;
        }

        for trailing_offset in [0usize, 4, 8] {
            if total_beats < beat_count + trailing_offset {
                continue;
            }

            let end_beat = total_beats - trailing_offset;
            let start_beat = end_beat.saturating_sub(beat_count);
            if let Some(window) =
                meter_window_candidate(beat_strengths, meter_strengths, start_beat, end_beat)
            {
                if window_is_recoverable(window) {
                    windows.push(window);
                }
            }
        }
    }

    if windows.is_empty() {
        return None;
    }

    let mut best_cluster = None;

    for candidate in windows.iter().copied() {
        let absolute_phase = window_absolute_phase(candidate);
        let mut supporters = Vec::new();

        for window in windows.iter().copied() {
            if window.hypothesis.beats_per_bar == candidate.hypothesis.beats_per_bar
                && window_absolute_phase(window) == absolute_phase
            {
                supporters.push(window);
            }
        }

        let cluster_start = supporters
            .iter()
            .map(|window| window.start_beat)
            .min()
            .unwrap_or(candidate.start_beat);
        let min_end = supporters
            .iter()
            .map(|window| window.end_beat)
            .min()
            .unwrap_or(candidate.end_beat);
        let max_end = supporters
            .iter()
            .map(|window| window.end_beat)
            .max()
            .unwrap_or(candidate.end_beat);
        let mean_confidence = supporters
            .iter()
            .map(|window| window.confidence.0)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_support = supporters
            .iter()
            .map(|window| window.hypothesis.support_ratio)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_regularity = supporters
            .iter()
            .map(|window| window.hypothesis.regularity)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_recent = supporters
            .iter()
            .map(|window| window.hypothesis.recent_strength)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_margin = supporters
            .iter()
            .map(|window| window.confidence_breakdown.phase_margin)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_meter_support = supporters
            .iter()
            .map(|window| window.confidence_breakdown.meter_support)
            .sum::<f32>()
            / supporters.len() as f32;
        let mean_salience = supporters
            .iter()
            .map(|window| window.confidence_breakdown.salience)
            .sum::<f32>()
            / supporters.len() as f32;

        if max_end != total_beats
            || max_end.saturating_sub(min_end) < 8
            || mean_confidence < 0.28
            || mean_support < 0.78
            || mean_regularity < 0.62
            || mean_recent < 0.14
        {
            continue;
        }

        let lead_end = cluster_start.min(total_beats);
        let lead_window = if lead_end >= 8 {
            meter_window_candidate(
                beat_strengths,
                meter_strengths,
                lead_end.saturating_sub(lead_end.min(16)),
                lead_end,
            )
        } else {
            None
        };
        let lead_improvement = if let Some(lead_window) = lead_window {
            let lead_unstable = lead_window.hypothesis.support_ratio < 0.58
                || lead_window.hypothesis.regularity < 0.48
                || lead_window.hypothesis.meter_support_ratio < 0.46
                || lead_window.hypothesis.meter_contrast_mean < 0.045;
            lead_unstable
                && (mean_confidence >= lead_window.confidence.0 + 0.05
                    || candidate.hypothesis.score >= lead_window.hypothesis.score + 0.03)
        } else {
            false
        };

        if !lead_improvement {
            continue;
        }

        let cluster_score = 0.45 * mean_confidence
            + 0.20 * mean_regularity
            + 0.15 * mean_support
            + 0.10 * mean_recent
            + 0.10 * ((max_end.saturating_sub(min_end)) as f32 / 8.0).clamp(0.0, 1.0);
        let adjusted_phase_offset = window_phase_offset(
            candidate.hypothesis.beats_per_bar,
            absolute_phase,
            cluster_start,
        );
        let adjusted_candidate = MeterWindowCandidate {
            start_beat: cluster_start,
            end_beat: total_beats,
            hypothesis: MeterHypothesis {
                phase_offset_beats: adjusted_phase_offset,
                ..candidate.hypothesis
            },
            confidence: Confidence::new(cluster_score.clamp(0.0, 1.0)),
            confidence_breakdown: MeterConfidenceBreakdown {
                phase_margin: mean_margin,
                support: mean_support,
                meter_support: mean_meter_support,
                regularity: mean_regularity,
                recent_stability: mean_recent,
                salience: mean_salience,
            },
            supporting_windows: supporters.len(),
        };

        match best_cluster {
            Some((best_score, _)) if best_score >= cluster_score => {}
            _ => best_cluster = Some((cluster_score, adjusted_candidate)),
        }
    }

    best_cluster.map(|(_, candidate)| candidate)
}

pub(crate) fn meter_recovery_context(
    beat_frames: &[usize],
    sample_rate: SampleRate,
    hop_size: usize,
    candidate: MeterWindowCandidate,
) -> MeterRecoveryContext {
    let recovered_beats = candidate.end_beat.saturating_sub(candidate.start_beat);
    MeterRecoveryContext {
        start_beat_index: candidate.start_beat,
        end_beat_index: candidate.end_beat,
        recovered_beats,
        recovered_bars: recovered_beats / candidate.hypothesis.beats_per_bar,
        start_seconds: crate::beat_index_to_seconds(
            beat_frames,
            candidate.start_beat,
            sample_rate,
            hop_size,
        ),
        end_seconds: crate::beat_index_to_seconds(
            beat_frames,
            candidate.end_beat.saturating_sub(1),
            sample_rate,
            hop_size,
        ),
        supporting_windows: candidate.supporting_windows,
    }
}

pub(crate) fn trailing_meter_window_candidate(
    beat_strengths: &[f32],
    meter_strengths: &[f32],
) -> Option<MeterWindowCandidate> {
    let total_beats = beat_strengths.len().min(meter_strengths.len());
    let mut best_candidate: Option<MeterWindowCandidate> = None;

    for beat_count in [8usize, 12, 16] {
        if beat_count > total_beats {
            continue;
        }

        for trailing_offset in [0usize, 4, 8] {
            if total_beats < beat_count + trailing_offset {
                continue;
            }

            let end_beat = total_beats - trailing_offset;
            let start_beat = end_beat.saturating_sub(beat_count);
            let Some(candidate) =
                meter_window_candidate(beat_strengths, meter_strengths, start_beat, end_beat)
            else {
                continue;
            };

            match best_candidate {
                Some(best) if best.confidence.0 >= candidate.confidence.0 => {}
                _ => best_candidate = Some(candidate),
            }
        }
    }

    best_candidate
}
