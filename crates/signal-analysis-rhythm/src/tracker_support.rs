use signal_analysis::Confidence;
use signal_primitives::SampleRate;

use crate::beat_utils::normalize;
use crate::rhythm_policy::MeterHypothesis;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TempoHypothesis {
    pub(crate) bpm: f32,
    pub(crate) lag_frames: usize,
    pub(crate) refined_lag_frames: f32,
    pub(crate) phase_offset_frames: usize,
    pub(crate) phase_score: f32,
    pub(crate) score: f32,
    pub(crate) confidence: Confidence,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TempoEstimate {
    pub(crate) bpm: f32,
    pub(crate) confidence: Confidence,
    pub(crate) lag_frames: usize,
    pub(crate) phase_offset_frames: usize,
    pub(crate) candidates: [Option<TempoHypothesis>; 3],
    pub(crate) ambiguity: Confidence,
}

pub(crate) fn combine_meter_cues(low_band_cue: &[f32], profile_change_cue: &[f32]) -> Vec<f32> {
    let len = low_band_cue.len().max(profile_change_cue.len());
    let mut combined = vec![0.0; len];

    for (index, value) in combined.iter_mut().enumerate().take(len) {
        let low = low_band_cue.get(index).copied().unwrap_or(0.0);
        let profile = profile_change_cue.get(index).copied().unwrap_or(0.0);
        *value = 0.55 * low + 0.45 * profile;
    }

    normalize(&mut combined);
    combined
}

pub(crate) fn downbeat_frames_for_hypothesis(
    beat_frames: &[usize],
    beat_offset: usize,
    hypothesis: MeterHypothesis,
) -> Vec<usize> {
    beat_frames
        .iter()
        .skip(beat_offset + hypothesis.phase_offset_beats)
        .step_by(hypothesis.beats_per_bar)
        .copied()
        .collect()
}

pub(crate) fn beat_index_to_seconds(
    beat_frames: &[usize],
    beat_index: usize,
    sample_rate: SampleRate,
    hop_size: usize,
) -> f32 {
    if sample_rate.0 == 0 || hop_size == 0 {
        return 0.0;
    }

    beat_frames
        .get(beat_index)
        .copied()
        .unwrap_or_else(|| *beat_frames.last().unwrap_or(&0)) as f32
        * hop_size as f32
        / sample_rate.0 as f32
}
