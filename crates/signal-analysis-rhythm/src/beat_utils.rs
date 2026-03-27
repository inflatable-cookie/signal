pub(crate) fn select_beat_phase(onset_envelope: &[f32], lag_frames: usize) -> usize {
    if onset_envelope.is_empty() || lag_frames == 0 {
        return 0;
    }

    let search_len = lag_frames.min(onset_envelope.len());
    let mut best_phase = 0usize;
    let mut best_score = 0.0f32;

    for phase in 0..search_len {
        let score = beat_phase_score(onset_envelope, lag_frames, phase);
        if score > best_score {
            best_score = score;
            best_phase = phase;
        }
    }

    best_phase
}

pub(crate) fn beat_phase_score(
    onset_envelope: &[f32],
    lag_frames: usize,
    phase_offset_frames: usize,
) -> f32 {
    if onset_envelope.is_empty() || lag_frames == 0 {
        return 0.0;
    }

    let radius = ((lag_frames as f32) * 0.15).round().max(1.0) as usize;
    let half_lag = lag_frames / 2;
    let mut beat_sum = 0.0;
    let mut beat_count = 0usize;
    let mut supported_beats = 0usize;
    let mut offbeat_sum = 0.0;
    let mut offbeat_count = 0usize;

    let mut index = phase_offset_frames.min(onset_envelope.len().saturating_sub(1));
    while index < onset_envelope.len() {
        let beat_peak = neighborhood_peak(onset_envelope, index, radius);
        beat_sum += beat_peak;
        beat_count += 1;
        if beat_peak > 0.35 {
            supported_beats += 1;
        }

        if half_lag > 1 {
            let midpoint = index + half_lag;
            if midpoint < onset_envelope.len() {
                offbeat_sum += neighborhood_peak(onset_envelope, midpoint, radius);
                offbeat_count += 1;
            }
        }

        index += lag_frames;
    }

    if beat_count == 0 {
        return 0.0;
    }

    let beat_average = beat_sum / beat_count as f32;
    let support_ratio = supported_beats as f32 / beat_count as f32;
    let offbeat_average = if offbeat_count > 0 {
        offbeat_sum / offbeat_count as f32
    } else {
        0.0
    };

    (0.55 * beat_average + 0.45 * support_ratio - 0.35 * offbeat_average)
        .max(0.0)
        .clamp(0.0, 1.0)
}

pub(crate) fn neighborhood_peak(onset_envelope: &[f32], center: usize, radius: usize) -> f32 {
    if onset_envelope.is_empty() {
        return 0.0;
    }

    let start = center.saturating_sub(radius);
    let end = (center + radius + 1).min(onset_envelope.len());
    onset_envelope[start..end]
        .iter()
        .copied()
        .fold(0.0f32, |best, value| best.max(value))
}

pub(crate) fn refine_beat(onset_envelope: &[f32], center: isize, tolerance_frames: isize) -> isize {
    let start = (center - tolerance_frames).max(0) as usize;
    let end =
        (center + tolerance_frames).min(onset_envelope.len().saturating_sub(1) as isize) as usize;

    let mut best_index = center.clamp(0, onset_envelope.len().saturating_sub(1) as isize) as usize;
    let mut best_value = onset_envelope[best_index];

    for index in start..=end {
        let value = onset_envelope[index];
        if value > best_value {
            best_value = value;
            best_index = index;
        }
    }

    best_index as isize
}

pub fn normalize(values: &mut [f32]) {
    let max_value = values
        .iter()
        .copied()
        .fold(0.0f32, |best, value| best.max(value));

    if max_value > 0.0 {
        for value in values {
            *value /= max_value;
        }
    }
}
