const TARGET_RMS: f64 = 0.15;
pub(super) const PEAK_CEILING: f64 = 0.95;

pub(super) fn tail_excerpt(samples: &[f32], frame_limit: usize) -> &[f32] {
    &samples[samples.len().saturating_sub(frame_limit)..]
}

pub(super) fn shared_tail_gain(current: &[f32], alternatives: [&[f32]; 2]) -> Result<f64, String> {
    if current.is_empty() {
        return Err("tail listening current candidate is empty".to_string());
    }
    let rms = (current
        .iter()
        .map(|sample| (*sample as f64) * (*sample as f64))
        .sum::<f64>()
        / current.len() as f64)
        .sqrt();
    let peak = current
        .iter()
        .chain(alternatives[0])
        .chain(alternatives[1])
        .map(|sample| sample.abs() as f64)
        .fold(0.0, f64::max);
    if !rms.is_finite() || rms <= 0.0 || !peak.is_finite() || peak <= 0.0 {
        return Err("tail listening level reference is silent or invalid".to_string());
    }
    Ok((TARGET_RMS / rms).min(PEAK_CEILING / peak))
}

pub(super) fn append_silence(samples: &[f32], gain: f64, silence_frames: usize) -> Vec<f32> {
    let mut output = Vec::with_capacity(samples.len() + silence_frames);
    output.extend(samples.iter().map(|sample| (*sample as f64 * gain) as f32));
    output.resize(output.len() + silence_frames, 0.0);
    output
}

pub(super) fn amplitude_dbfs(amplitude: f64) -> f64 {
    if amplitude > 0.0 {
        20.0 * amplitude.log10()
    } else {
        f64::NEG_INFINITY
    }
}
