use std::path::Path;

use signal_dsp_stretch::OfflineHighQualityPath;

use super::ExternalBenchmarkQualityRender;

pub(super) const PEAK_CEILING: f64 = 0.95;

pub(super) struct LevelMatchedGroup {
    pub(super) source: Vec<f32>,
    pub(super) signal: Vec<f32>,
    pub(super) external: Vec<f32>,
    pub(super) target_rms: f64,
    pub(super) source_gain: f64,
    pub(super) signal_gain: f64,
    pub(super) external_gain: f64,
}

pub(super) fn level_match_group(
    source: &[f32],
    signal: &[f32],
    external: &[f32],
) -> Result<LevelMatchedGroup, String> {
    let source_stats = level_stats(source)?;
    let signal_stats = level_stats(signal)?;
    let external_stats = level_stats(external)?;
    let target_rms = source_stats
        .rms
        .min(source_stats.max_safe_rms)
        .min(signal_stats.max_safe_rms.min(external_stats.max_safe_rms));
    if !target_rms.is_finite() || target_rms <= 0.0 {
        return Err("blind listening level target is silent or invalid".to_string());
    }
    let source_gain = target_rms / source_stats.rms;
    let signal_gain = target_rms / signal_stats.rms;
    let external_gain = target_rms / external_stats.rms;
    Ok(LevelMatchedGroup {
        source: apply_gain(source, source_gain),
        signal: apply_gain(signal, signal_gain),
        external: apply_gain(external, external_gain),
        target_rms,
        source_gain,
        signal_gain,
        external_gain,
    })
}

pub(super) struct LevelStats {
    pub(super) rms: f64,
    max_safe_rms: f64,
}

pub(super) fn level_stats(samples: &[f32]) -> Result<LevelStats, String> {
    if samples.is_empty() {
        return Err("blind listening candidate is empty".to_string());
    }
    let rms = (samples
        .iter()
        .map(|sample| (*sample as f64) * (*sample as f64))
        .sum::<f64>()
        / samples.len() as f64)
        .sqrt();
    let peak = samples
        .iter()
        .map(|sample| sample.abs() as f64)
        .fold(0.0, f64::max);
    if !rms.is_finite() || rms <= 0.0 || !peak.is_finite() || peak <= 0.0 {
        return Err("blind listening candidate is silent or invalid".to_string());
    }
    Ok(LevelStats {
        rms,
        max_safe_rms: rms * PEAK_CEILING / peak,
    })
}

fn apply_gain(samples: &[f32], gain: f64) -> Vec<f32> {
    samples
        .iter()
        .map(|sample| (*sample as f64 * gain) as f32)
        .collect()
}

pub(super) fn gain_db(gain: f64) -> f64 {
    20.0 * gain.log10()
}

pub(super) fn stable_assignment_is_signal_a(
    render: &ExternalBenchmarkQualityRender,
    signal_path: OfflineHighQualityPath,
) -> bool {
    let assignment = format!(
        "{}|{:.9}|{}|{signal_path:?}",
        render.case_id, render.ratio, render.rendered_path
    );
    let hash = assignment
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    hash & 1 == 0
}

pub(super) fn write_float_wav(
    path: &Path,
    sample_rate: u32,
    channels: u16,
    samples: &[f32],
) -> Result<(), String> {
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .map_err(|error| format!("failed to create {}: {error}", path.display()))?;
    for sample in samples {
        writer
            .write_sample(*sample)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    }
    writer
        .finalize()
        .map_err(|error| format!("failed to finalize {}: {error}", path.display()))
}
