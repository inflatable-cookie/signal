use signal_analysis::Confidence;
use signal_primitives::SampleRate;

use crate::refine_beat;

pub(crate) fn combined_confidence(
    onset_envelope: &[f32],
    tempo_confidence: Confidence,
    beat_positions_seconds: &[f32],
    bpm: f32,
) -> Confidence {
    if onset_envelope.is_empty() || bpm <= 0.0 {
        return Confidence::new(0.0);
    }

    let peak = onset_envelope
        .iter()
        .copied()
        .fold(0.0f32, |best, value| best.max(value));
    let mean = onset_envelope.iter().copied().sum::<f32>() / onset_envelope.len() as f32;
    let onset_strength = (peak - mean).max(0.0);
    let beat_density = (beat_positions_seconds.len() as f32 / 16.0).clamp(0.0, 1.0);
    Confidence::new(0.5 * onset_strength + 0.35 * tempo_confidence.0 + 0.15 * beat_density)
}

pub(crate) fn track_beats(
    onset_envelope: &[f32],
    lag_frames: usize,
    phase_offset_frames: usize,
    beat_tolerance: f32,
) -> Vec<usize> {
    if onset_envelope.is_empty() || lag_frames == 0 {
        return Vec::new();
    }

    let tolerance_frames = (lag_frames as f32 * beat_tolerance).round().max(1.0) as isize;
    let phase_offset_frames = phase_offset_frames.min(onset_envelope.len().saturating_sub(1));

    let mut beats = vec![refine_beat(
        onset_envelope,
        phase_offset_frames as isize,
        tolerance_frames,
    )];

    let mut next = phase_offset_frames as isize + lag_frames as isize;
    while next < onset_envelope.len() as isize {
        beats.push(refine_beat(onset_envelope, next, tolerance_frames));
        next += lag_frames as isize;
    }

    let mut previous = phase_offset_frames as isize - lag_frames as isize;
    while previous >= 0 {
        beats.push(refine_beat(onset_envelope, previous, tolerance_frames));
        previous -= lag_frames as isize;
    }

    beats.sort_unstable();
    beats.dedup();

    beats
        .into_iter()
        .filter(|frame| *frame >= 0)
        .map(|frame| frame as usize)
        .collect()
}

pub(crate) fn beat_frames_to_seconds(
    beat_frames: &[usize],
    sample_rate: SampleRate,
    hop_size: usize,
) -> Vec<f32> {
    if sample_rate.0 == 0 || hop_size == 0 {
        return Vec::new();
    }

    beat_frames
        .iter()
        .map(|frame| *frame as f32 * hop_size as f32 / sample_rate.0 as f32)
        .collect()
}

pub(crate) fn beat_frames_to_seconds_refined(
    beat_frames: &[f32],
    sample_rate: SampleRate,
    hop_size: usize,
) -> Vec<f32> {
    if sample_rate.0 == 0 || hop_size == 0 {
        return Vec::new();
    }

    beat_frames
        .iter()
        .map(|frame| *frame * hop_size as f32 / sample_rate.0 as f32)
        .collect()
}

fn refine_peak_frame(onset_envelope: &[f32], frame: usize) -> f32 {
    if onset_envelope.is_empty() {
        return 0.0;
    }
    if frame == 0 || frame + 1 >= onset_envelope.len() {
        return frame as f32;
    }

    let left = onset_envelope[frame - 1];
    let center = onset_envelope[frame];
    let right = onset_envelope[frame + 1];
    let denominator = left - 2.0 * center + right;
    if denominator.abs() <= f32::EPSILON {
        return frame as f32;
    }

    let delta = (0.5 * (left - right) / denominator).clamp(-0.5, 0.5);
    frame as f32 + delta
}

pub(crate) fn refine_beat_frames(onset_envelope: &[f32], beat_frames: &[usize]) -> Vec<f32> {
    beat_frames
        .iter()
        .map(|frame| refine_peak_frame(onset_envelope, *frame))
        .collect()
}
