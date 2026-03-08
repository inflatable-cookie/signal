//! Loudness analysis surfaces for Signal.

use signal_analysis::{AnalysisMode, AnalysisStage, Confidence};
use signal_primitives::{AudioBuffer, Sample, SampleRate};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessMeterConfig {
    pub target_lufs: f32,
    pub block_seconds: f32,
    pub hop_seconds: f32,
    pub short_term_seconds: f32,
}

impl Default for LoudnessMeterConfig {
    fn default() -> Self {
        Self {
            target_lufs: -14.0,
            block_seconds: 0.400,
            hop_seconds: 0.100,
            short_term_seconds: 3.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessAnalysisResult {
    pub integrated_lufs: f32,
    pub loudness_range_lu: f32,
    pub true_peak_dbtp: f32,
    pub confidence: Confidence,
}

#[derive(Debug, Default)]
pub struct LoudnessMeter {
    config: LoudnessMeterConfig,
}

impl LoudnessMeter {
    pub fn new(config: LoudnessMeterConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> LoudnessMeterConfig {
        self.config
    }

    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> LoudnessAnalysisResult {
        if mono_samples.is_empty() || sample_rate.0 == 0 {
            return LoudnessAnalysisResult {
                integrated_lufs: f32::NEG_INFINITY,
                loudness_range_lu: 0.0,
                true_peak_dbtp: f32::NEG_INFINITY,
                confidence: Confidence::new(0.0),
            };
        }

        let weighted = k_weight(sample_rate, mono_samples);
        let has_signal = weighted.iter().any(|sample| sample.abs() > 0.0);
        let block_size = seconds_to_frames(sample_rate, self.config.block_seconds).max(1);
        let hop_size = seconds_to_frames(sample_rate, self.config.hop_seconds).max(1);
        let short_term_size =
            seconds_to_frames(sample_rate, self.config.short_term_seconds).max(block_size);

        let block_energies = window_mean_square(&weighted, block_size, hop_size);
        let integrated_lufs = gated_integrated_loudness(&block_energies);
        let loudness_range_lu = loudness_range(&weighted, short_term_size, hop_size);
        let true_peak_dbtp = true_peak_dbtp(mono_samples, 4);
        let confidence = if has_signal {
            loudness_confidence(sample_rate, block_energies.len())
        } else {
            Confidence::new(0.0)
        };

        LoudnessAnalysisResult {
            integrated_lufs,
            loudness_range_lu,
            true_peak_dbtp,
            confidence,
        }
    }
}

impl AnalysisStage<LoudnessAnalysisResult> for LoudnessMeter {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> LoudnessAnalysisResult {
        self.analyze_mono(audio.sample_rate(), &audio.to_mono())
    }
}

#[derive(Clone, Copy, Debug)]
struct BiquadCoefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadState {
    fn process(&mut self, coeffs: BiquadCoefficients, input: f32) -> f32 {
        let output = coeffs.b0 * input + coeffs.b1 * self.x1 + coeffs.b2 * self.x2
            - coeffs.a1 * self.y1
            - coeffs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

const PRE_FILTER_48K: BiquadCoefficients = BiquadCoefficients {
    b0: 1.5351249,
    b1: -2.6916962,
    b2: 1.1983929,
    a1: -1.6906593,
    a2: 0.73248076,
};

const HIGH_SHELF_48K: BiquadCoefficients = BiquadCoefficients {
    b0: 1.0049943,
    b1: -1.9899137,
    b2: 0.9849193,
    a1: -1.9970102,
    a2: 0.9970102,
};

fn k_weight(sample_rate: SampleRate, samples: &[f32]) -> Vec<f32> {
    if sample_rate.0 != 48_000 {
        return samples.to_vec();
    }

    let mut stage_one = BiquadState::default();
    let mut stage_two = BiquadState::default();
    let mut weighted = Vec::with_capacity(samples.len());

    for sample in samples {
        let first = stage_one.process(PRE_FILTER_48K, *sample);
        let second = stage_two.process(HIGH_SHELF_48K, first);
        weighted.push(second);
    }

    weighted
}

fn seconds_to_frames(sample_rate: SampleRate, seconds: f32) -> usize {
    (sample_rate.0 as f32 * seconds).round().max(0.0) as usize
}

fn window_mean_square(samples: &[f32], window_size: usize, hop_size: usize) -> Vec<f32> {
    if samples.is_empty() || window_size == 0 || hop_size == 0 {
        return Vec::new();
    }

    let mut energies = Vec::new();
    let mut start = 0usize;
    while start < samples.len() {
        let end = (start + window_size).min(samples.len());
        let window = &samples[start..end];
        if window.is_empty() {
            break;
        }
        let mean_square =
            window.iter().map(|sample| sample * sample).sum::<f32>() / window.len() as f32;
        energies.push(mean_square);

        if end == samples.len() {
            break;
        }
        start = start.saturating_add(hop_size);
    }
    energies
}

fn lufs_from_mean_square(mean_square: f32) -> f32 {
    if mean_square <= 0.0 {
        f32::NEG_INFINITY
    } else {
        -0.691 + 10.0 * mean_square.log10()
    }
}

fn gated_integrated_loudness(block_energies: &[f32]) -> f32 {
    if block_energies.is_empty() {
        return f32::NEG_INFINITY;
    }

    let absolute_gated: Vec<f32> = block_energies
        .iter()
        .copied()
        .filter(|energy| lufs_from_mean_square(*energy) >= -70.0)
        .collect();

    if absolute_gated.is_empty() {
        return f32::NEG_INFINITY;
    }

    let absolute_mean = absolute_gated.iter().copied().sum::<f32>() / absolute_gated.len() as f32;
    let relative_threshold = lufs_from_mean_square(absolute_mean) - 10.0;

    let relative_gated: Vec<f32> = absolute_gated
        .into_iter()
        .filter(|energy| lufs_from_mean_square(*energy) >= relative_threshold)
        .collect();

    if relative_gated.is_empty() {
        return f32::NEG_INFINITY;
    }

    let integrated_mean = relative_gated.iter().copied().sum::<f32>() / relative_gated.len() as f32;
    lufs_from_mean_square(integrated_mean)
}

fn loudness_range(samples: &[f32], short_term_size: usize, hop_size: usize) -> f32 {
    let mut loudness_values: Vec<f32> = window_mean_square(samples, short_term_size, hop_size)
        .into_iter()
        .map(lufs_from_mean_square)
        .filter(|value| value.is_finite() && *value >= -70.0)
        .collect();

    if loudness_values.len() < 2 {
        return 0.0;
    }

    loudness_values.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal));
    let lower = percentile(&loudness_values, 0.10);
    let upper = percentile(&loudness_values, 0.95);
    (upper - lower).max(0.0)
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    let index = ((sorted.len() - 1) as f32 * fraction).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn true_peak_dbtp(samples: &[f32], oversample_factor: usize) -> f32 {
    if samples.is_empty() || oversample_factor == 0 {
        return f32::NEG_INFINITY;
    }

    let mut peak = 0.0f32;
    for window in samples.windows(2) {
        let start = window[0];
        let end = window[1];
        peak = peak.max(start.abs()).max(end.abs());
        for step in 1..oversample_factor {
            let t = step as f32 / oversample_factor as f32;
            let interpolated = start + (end - start) * t;
            peak = peak.max(interpolated.abs());
        }
    }

    if peak == 0.0 {
        f32::NEG_INFINITY
    } else {
        20.0 * peak.log10()
    }
}

fn loudness_confidence(sample_rate: SampleRate, block_count: usize) -> Confidence {
    let rate_factor = if sample_rate.0 == 48_000 { 1.0 } else { 0.4 };
    let coverage_factor = (block_count as f32 / 10.0).clamp(0.0, 1.0);
    Confidence::new(rate_factor * coverage_factor)
}

#[cfg(test)]
mod tests {
    use super::{LoudnessMeter, LoudnessMeterConfig};
    use signal_analysis::AnalysisStage;
    use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

    fn sine(sample_rate: u32, frequency: f32, amplitude: f32, seconds: f32) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let mut samples = Vec::with_capacity(frames);
        for index in 0..frames {
            let t = index as f32 / sample_rate as f32;
            samples.push(amplitude * (core::f32::consts::TAU * frequency * t).sin());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    #[test]
    fn silence_reports_negative_infinity() {
        let audio = AudioBuffer::new(
            SampleRate(48_000),
            ChannelLayout::Mono,
            signal_primitives::FrameCount(48_000),
        );
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
        let result = meter.analyze(&audio);

        assert!(!result.integrated_lufs.is_finite());
        assert!(!result.true_peak_dbtp.is_finite());
        assert_eq!(result.loudness_range_lu, 0.0);
        assert_eq!(result.confidence.0, 0.0);
    }

    #[test]
    fn louder_signal_has_higher_loudness_and_peak() {
        let quiet = sine(48_000, 1_000.0, 0.1, 4.0);
        let loud = sine(48_000, 1_000.0, 0.5, 4.0);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let quiet_result = meter.analyze(&quiet);
        let loud_result = meter.analyze(&loud);

        assert!(loud_result.integrated_lufs > quiet_result.integrated_lufs);
        assert!(loud_result.true_peak_dbtp > quiet_result.true_peak_dbtp);
        assert!(loud_result.confidence.0 > 0.9);
    }

    #[test]
    fn unsupported_sample_rate_reduces_confidence() {
        let supported = sine(48_000, 440.0, 0.2, 4.0);
        let unsupported = sine(44_100, 440.0, 0.2, 4.0);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let supported_result = meter.analyze(&supported);
        let unsupported_result = meter.analyze(&unsupported);

        assert!(supported_result.confidence.0 > unsupported_result.confidence.0);
    }
}
