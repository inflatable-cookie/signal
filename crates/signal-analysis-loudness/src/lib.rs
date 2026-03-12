//! Loudness analysis surfaces for Signal.
//!
//! The crate currently exposes an offline loudness meter with integrated
//! loudness, trace, true-peak estimation, and confidence reporting.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_loudness::{LoudnessMeter, LoudnessMeterConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
//! let result = meter.analyze(&audio);
//!
//! assert_eq!(meter.mode(), signal_analysis::AnalysisMode::Offline);
//! assert_eq!(result.loudness_range_lu, 0.0);
//! ```

use signal_analysis::{
    prepare_mono_analysis, AnalysisInputConfig, AnalysisMode, AnalysisStage, Confidence,
};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, Sample, SampleRate, Seconds};

/// Configuration for the offline loudness meter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessMeterConfig {
    pub target_lufs: f32,
    pub block_seconds: f32,
    pub hop_seconds: f32,
    pub short_term_seconds: f32,
    /// Sample rate used by the loudness analysis path after input prep.
    ///
    /// Loudness weighting and confidence are currently calibrated for 48 kHz,
    /// so the default profiles freeze that rate and resample inputs when
    /// needed instead of silently degrading on non-48k material.
    pub analysis_sample_rate: SampleRate,
    /// Maximum duration to analyse, taken from the centre of the track.
    /// `None` means the entire track is processed (spec-compliant integrated
    /// LUFS).  Setting a value gives a faster estimate that may differ from
    /// the true whole-programme loudness.
    pub analysis_duration_seconds: Option<u32>,
}

impl LoudnessMeterConfig {
    /// Quick scanning profile — analyses a 30-second centre segment.
    pub fn low() -> Self {
        Self {
            analysis_duration_seconds: Some(30),
            ..Self::default()
        }
    }

    /// Balanced profile — analyses a 60-second centre segment.
    pub fn medium() -> Self {
        Self {
            analysis_duration_seconds: Some(60),
            ..Self::default()
        }
    }

    /// Full-accuracy profile — analyses the entire track.
    pub fn high() -> Self {
        Self::default()
    }
}

impl Default for LoudnessMeterConfig {
    fn default() -> Self {
        Self {
            target_lufs: -14.0,
            block_seconds: 0.400,
            hop_seconds: 0.100,
            short_term_seconds: 3.0,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: None,
        }
    }
}

/// Summary loudness metrics for one analyzed buffer.
///
/// Practical integration order:
/// 1. Read `integrated_lufs` as the program-level loudness figure.
/// 2. Read `loudness_range_lu` to gauge macro dynamics across the analyzed span.
/// 3. Read `true_peak_dbtp` before applying delivery or limiter decisions.
/// 4. Read `confidence` to determine whether the buffer was long and energetic
///    enough for the reported numbers to be treated as stable.
#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessAnalysisResult {
    pub integrated_lufs: f32,
    pub loudness_range_lu: f32,
    pub true_peak_dbtp: f32,
    pub confidence: Confidence,
    pub channels: Vec<LoudnessChannelSummary>,
    pub aggregation: LoudnessAggregationSummary,
    pub momentary_trace: LoudnessTrace,
    pub short_term_trace: LoudnessTrace,
    pub dynamics: LoudnessDynamicsSummary,
}

/// Channel-weighting contract applied during loudness aggregation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoudnessChannelWeightSource {
    MonoDirect,
    StereoEqualWeight,
    GenericCountFallback,
}

/// Loudness weighting support used at the configured analysis sample rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoudnessSampleRateSupport {
    Native48kKWeighted,
    ResampledTo48kKWeighted,
    UnweightedFallback,
}

/// Per-channel loudness evidence before cross-channel aggregation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessChannelSummary {
    pub index: usize,
    pub weight: f32,
    pub integrated_lufs: f32,
    pub true_peak_dbtp: f32,
}

/// Summary of the loudness aggregation contract used for the current buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessAggregationSummary {
    pub channel_count: ChannelCount,
    pub channel_weight_source: LoudnessChannelWeightSource,
    pub sample_rate_support: LoudnessSampleRateSupport,
    pub analysis_sample_rate: SampleRate,
    pub true_peak_oversample_factor: usize,
}

/// One loudness trace point over a fixed analysis window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessTracePoint {
    pub index: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub loudness_lufs: f32,
}

/// Time-series loudness trace over one fixed window size.
#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessTrace {
    pub window_seconds: f32,
    pub hop_seconds: f32,
    pub points: Vec<LoudnessTracePoint>,
}

/// Compact delivery-facing dynamics summary built on the trace surfaces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoudnessDynamicsSummary {
    pub target_offset_lu: f32,
    pub peak_to_loudness_lu: f32,
    pub momentary_max_lufs: f32,
    pub short_term_max_lufs: f32,
    pub momentary_range_lu: f32,
    pub short_term_range_lu: f32,
}

/// Bounded loudness subset intended for runtime-diagnostics reuse.
#[derive(Clone, Debug, PartialEq)]
pub struct LoudnessRuntimeDiagnosticsSummary {
    pub integrated_lufs: f32,
    pub true_peak_dbtp: f32,
    pub target_offset_lu: f32,
    pub peak_to_loudness_lu: f32,
    pub current_momentary_lufs: f32,
    pub current_short_term_lufs: f32,
    pub momentary_max_lufs: f32,
    pub short_term_max_lufs: f32,
    pub recent_momentary: LoudnessTrace,
    pub recent_short_term: LoudnessTrace,
}

const RUNTIME_MOMENTARY_TAIL_POINTS: usize = 8;
const RUNTIME_SHORT_TERM_TAIL_POINTS: usize = 4;

/// Offline mono loudness meter.
#[derive(Debug, Default)]
pub struct LoudnessMeter {
    config: LoudnessMeterConfig,
}

impl LoudnessMeter {
    /// Create a meter with the provided config.
    pub fn new(config: LoudnessMeterConfig) -> Self {
        Self { config }
    }

    /// Return the current loudness config.
    pub fn config(&self) -> LoudnessMeterConfig {
        self.config
    }

    /// Analyze a mono sample slice directly.
    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> LoudnessAnalysisResult {
        self.analyze_channels(sample_rate, &[mono_samples.to_vec()], ChannelLayout::Mono)
    }

    fn analysis_input_config(&self) -> AnalysisInputConfig {
        AnalysisInputConfig {
            max_duration: self
                .config
                .analysis_duration_seconds
                .map(|seconds| Seconds(seconds as f32)),
            target_sample_rate: Some(self.config.analysis_sample_rate),
            ..AnalysisInputConfig::default()
        }
    }

    fn analyze_channels(
        &self,
        source_sample_rate: SampleRate,
        channels: &[Vec<Sample>],
        layout: ChannelLayout,
    ) -> LoudnessAnalysisResult {
        let channel_count = ChannelCount(channels.len());
        let (weights, channel_weight_source) = loudness_channel_weights(layout, channel_count);
        let analysis_sample_rate = self.config.analysis_sample_rate;
        let sample_rate_support =
            loudness_sample_rate_support(source_sample_rate, analysis_sample_rate);
        let true_peak_oversample_factor = true_peak_oversample_factor(analysis_sample_rate);
        let aggregation = LoudnessAggregationSummary {
            channel_count,
            channel_weight_source,
            sample_rate_support,
            analysis_sample_rate,
            true_peak_oversample_factor,
        };

        if channels.is_empty() || source_sample_rate.0 == 0 {
            return empty_loudness_result(aggregation);
        }

        let prepared_channels: Vec<Vec<Sample>> = channels
            .iter()
            .map(|channel| {
                prepare_mono_analysis(source_sample_rate, channel, self.analysis_input_config())
                    .samples
            })
            .collect();
        if prepared_channels.iter().all(Vec::is_empty) {
            return empty_loudness_result(aggregation);
        }

        let analysis_sample_rate = self.config.analysis_sample_rate;
        let block_size = seconds_to_frames(analysis_sample_rate, self.config.block_seconds).max(1);
        let hop_size = seconds_to_frames(analysis_sample_rate, self.config.hop_seconds).max(1);
        let short_term_size =
            seconds_to_frames(analysis_sample_rate, self.config.short_term_seconds).max(block_size);

        let weighted_channels: Vec<Vec<f32>> = prepared_channels
            .iter()
            .map(|channel| apply_loudness_weighting(analysis_sample_rate, channel))
            .collect();
        let block_energies_by_channel: Vec<Vec<f32>> = weighted_channels
            .iter()
            .map(|channel| window_mean_square(channel, block_size, hop_size))
            .collect();
        let short_term_energies_by_channel: Vec<Vec<f32>> = weighted_channels
            .iter()
            .map(|channel| window_mean_square(channel, short_term_size, hop_size))
            .collect();

        let aggregated_block_energies =
            aggregate_weighted_energies(&block_energies_by_channel, &weights);
        let aggregated_short_term_energies =
            aggregate_weighted_energies(&short_term_energies_by_channel, &weights);
        let momentary_trace = loudness_trace_from_energies(
            &aggregated_block_energies,
            self.config.block_seconds,
            self.config.hop_seconds,
        );
        let short_term_trace = loudness_trace_from_energies(
            &aggregated_short_term_energies,
            self.config.short_term_seconds,
            self.config.hop_seconds,
        );
        let channels: Vec<LoudnessChannelSummary> = prepared_channels
            .iter()
            .enumerate()
            .map(|(index, channel)| LoudnessChannelSummary {
                index,
                weight: *weights.get(index).unwrap_or(&1.0),
                integrated_lufs: gated_integrated_loudness(&block_energies_by_channel[index]),
                true_peak_dbtp: true_peak_dbtp(channel, true_peak_oversample_factor),
            })
            .collect();

        let integrated_lufs = gated_integrated_loudness(&aggregated_block_energies);
        let loudness_range_lu = loudness_range_from_energies(&aggregated_short_term_energies);
        let true_peak_dbtp = channels
            .iter()
            .map(|channel| channel.true_peak_dbtp)
            .fold(f32::NEG_INFINITY, f32::max);
        let dynamics = dynamics_summary(
            self.config.target_lufs,
            integrated_lufs,
            true_peak_dbtp,
            &momentary_trace,
            &short_term_trace,
        );
        let has_signal = weighted_channels
            .iter()
            .flatten()
            .any(|sample| sample.abs() > 0.0);
        let confidence = if has_signal {
            loudness_confidence(
                sample_rate_support,
                channel_weight_source,
                aggregated_block_energies.len(),
            )
        } else {
            Confidence::new(0.0)
        };

        LoudnessAnalysisResult {
            integrated_lufs,
            loudness_range_lu,
            true_peak_dbtp,
            confidence,
            channels,
            aggregation,
            momentary_trace,
            short_term_trace,
            dynamics,
        }
    }
}

impl AnalysisStage<LoudnessAnalysisResult> for LoudnessMeter {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> LoudnessAnalysisResult {
        let channels = deinterleave_channels(audio);
        self.analyze_channels(audio.sample_rate(), &channels, audio.channels())
    }
}

impl LoudnessAnalysisResult {
    /// Return the bounded loudness subset intended for runtime diagnostics.
    ///
    /// This keeps the reusable contract compact: top-line delivery metrics,
    /// current windowed state, and bounded recent trace tails rather than the
    /// full offline timeline.
    pub fn runtime_diagnostics_summary(&self) -> LoudnessRuntimeDiagnosticsSummary {
        let recent_momentary = trace_tail(&self.momentary_trace, RUNTIME_MOMENTARY_TAIL_POINTS);
        let recent_short_term = trace_tail(&self.short_term_trace, RUNTIME_SHORT_TERM_TAIL_POINTS);

        LoudnessRuntimeDiagnosticsSummary {
            integrated_lufs: self.integrated_lufs,
            true_peak_dbtp: self.true_peak_dbtp,
            target_offset_lu: self.dynamics.target_offset_lu,
            peak_to_loudness_lu: self.dynamics.peak_to_loudness_lu,
            current_momentary_lufs: trace_latest_loudness(&recent_momentary),
            current_short_term_lufs: trace_latest_loudness(&recent_short_term),
            momentary_max_lufs: self.dynamics.momentary_max_lufs,
            short_term_max_lufs: self.dynamics.short_term_max_lufs,
            recent_momentary,
            recent_short_term,
        }
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

fn apply_loudness_weighting(sample_rate: SampleRate, samples: &[f32]) -> Vec<f32> {
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

fn loudness_channel_weights(
    layout: ChannelLayout,
    channel_count: ChannelCount,
) -> (Vec<f32>, LoudnessChannelWeightSource) {
    match layout {
        ChannelLayout::Mono => (vec![1.0], LoudnessChannelWeightSource::MonoDirect),
        ChannelLayout::Stereo => (
            vec![1.0, 1.0],
            LoudnessChannelWeightSource::StereoEqualWeight,
        ),
        ChannelLayout::Count(count) => (
            vec![1.0; count.0.min(channel_count.0)],
            LoudnessChannelWeightSource::GenericCountFallback,
        ),
    }
}

fn loudness_sample_rate_support(
    source_sample_rate: SampleRate,
    analysis_sample_rate: SampleRate,
) -> LoudnessSampleRateSupport {
    if analysis_sample_rate.0 == 48_000 {
        if source_sample_rate.0 == 48_000 {
            LoudnessSampleRateSupport::Native48kKWeighted
        } else {
            LoudnessSampleRateSupport::ResampledTo48kKWeighted
        }
    } else {
        LoudnessSampleRateSupport::UnweightedFallback
    }
}

fn true_peak_oversample_factor(sample_rate: SampleRate) -> usize {
    match sample_rate.0 {
        0..=48_000 => 4,
        48_001..=96_000 => 2,
        _ => 1,
    }
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

fn aggregate_weighted_energies(channel_energies: &[Vec<f32>], weights: &[f32]) -> Vec<f32> {
    let max_len = channel_energies.iter().map(Vec::len).max().unwrap_or(0);
    let mut aggregated = vec![0.0; max_len];

    for (channel_index, energies) in channel_energies.iter().enumerate() {
        let gain = *weights.get(channel_index).unwrap_or(&1.0);
        let energy_scale = gain * gain;
        for (index, energy) in energies.iter().copied().enumerate() {
            aggregated[index] += energy * energy_scale;
        }
    }

    aggregated
}

fn loudness_range_from_energies(short_term_energies: &[f32]) -> f32 {
    let mut loudness_values: Vec<f32> = short_term_energies
        .iter()
        .copied()
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

fn loudness_trace_from_energies(
    energies: &[f32],
    window_seconds: f32,
    hop_seconds: f32,
) -> LoudnessTrace {
    let points = energies
        .iter()
        .copied()
        .enumerate()
        .map(|(index, energy)| {
            let start_seconds = index as f32 * hop_seconds;
            LoudnessTracePoint {
                index,
                start_seconds,
                end_seconds: start_seconds + window_seconds,
                loudness_lufs: lufs_from_mean_square(energy),
            }
        })
        .collect();

    LoudnessTrace {
        window_seconds,
        hop_seconds,
        points,
    }
}

fn trace_tail(trace: &LoudnessTrace, max_points: usize) -> LoudnessTrace {
    let keep_from = trace.points.len().saturating_sub(max_points);
    LoudnessTrace {
        window_seconds: trace.window_seconds,
        hop_seconds: trace.hop_seconds,
        points: trace.points[keep_from..].to_vec(),
    }
}

fn trace_latest_loudness(trace: &LoudnessTrace) -> f32 {
    trace
        .points
        .last()
        .map(|point| point.loudness_lufs)
        .unwrap_or(f32::NEG_INFINITY)
}

fn dynamics_summary(
    target_lufs: f32,
    integrated_lufs: f32,
    true_peak_dbtp: f32,
    momentary_trace: &LoudnessTrace,
    short_term_trace: &LoudnessTrace,
) -> LoudnessDynamicsSummary {
    let momentary_values: Vec<f32> = momentary_trace
        .points
        .iter()
        .map(|point| point.loudness_lufs)
        .filter(|value| value.is_finite())
        .collect();
    let short_term_values: Vec<f32> = short_term_trace
        .points
        .iter()
        .map(|point| point.loudness_lufs)
        .filter(|value| value.is_finite())
        .collect();
    let momentary_max_lufs = finite_max_or_neg_infinity(&momentary_values);
    let short_term_max_lufs = finite_max_or_neg_infinity(&short_term_values);
    let peak_to_loudness_lu = if true_peak_dbtp.is_finite() && integrated_lufs.is_finite() {
        (true_peak_dbtp - integrated_lufs).max(0.0)
    } else {
        0.0
    };

    LoudnessDynamicsSummary {
        target_offset_lu: if integrated_lufs.is_finite() {
            integrated_lufs - target_lufs
        } else {
            0.0
        },
        peak_to_loudness_lu,
        momentary_max_lufs,
        short_term_max_lufs,
        momentary_range_lu: loudness_range_from_values(&momentary_values),
        short_term_range_lu: loudness_range_from_values(&short_term_values),
    }
}

fn finite_max_or_neg_infinity(values: &[f32]) -> f32 {
    values.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

fn loudness_range_from_values(values: &[f32]) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal));
    let lower = percentile(&sorted, 0.10);
    let upper = percentile(&sorted, 0.95);
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

fn loudness_confidence(
    sample_rate_support: LoudnessSampleRateSupport,
    channel_weight_source: LoudnessChannelWeightSource,
    block_count: usize,
) -> Confidence {
    let rate_factor = match sample_rate_support {
        LoudnessSampleRateSupport::Native48kKWeighted => 1.0,
        LoudnessSampleRateSupport::ResampledTo48kKWeighted => 0.95,
        LoudnessSampleRateSupport::UnweightedFallback => 0.75,
    };
    let channel_factor = match channel_weight_source {
        LoudnessChannelWeightSource::MonoDirect
        | LoudnessChannelWeightSource::StereoEqualWeight => 1.0,
        LoudnessChannelWeightSource::GenericCountFallback => 0.9,
    };
    let coverage_factor = (block_count as f32 / 10.0).clamp(0.0, 1.0);
    Confidence::new(rate_factor * channel_factor * coverage_factor)
}

fn deinterleave_channels(audio: &AudioBuffer) -> Vec<Vec<Sample>> {
    let channel_count = audio.channel_count().0;
    if channel_count == 0 || audio.is_empty() {
        return Vec::new();
    }

    if channel_count == 1 {
        return vec![audio.samples().to_vec()];
    }

    let mut channels = vec![Vec::with_capacity(audio.frames().0); channel_count];
    for frame in audio.samples().chunks_exact(channel_count) {
        for (channel, sample) in channels.iter_mut().zip(frame.iter().copied()) {
            channel.push(sample);
        }
    }
    channels
}

fn empty_loudness_result(aggregation: LoudnessAggregationSummary) -> LoudnessAnalysisResult {
    LoudnessAnalysisResult {
        integrated_lufs: f32::NEG_INFINITY,
        loudness_range_lu: 0.0,
        true_peak_dbtp: f32::NEG_INFINITY,
        confidence: Confidence::new(0.0),
        channels: Vec::new(),
        aggregation,
        momentary_trace: LoudnessTrace {
            window_seconds: 0.0,
            hop_seconds: 0.0,
            points: Vec::new(),
        },
        short_term_trace: LoudnessTrace {
            window_seconds: 0.0,
            hop_seconds: 0.0,
            points: Vec::new(),
        },
        dynamics: LoudnessDynamicsSummary {
            target_offset_lu: 0.0,
            peak_to_loudness_lu: 0.0,
            momentary_max_lufs: f32::NEG_INFINITY,
            short_term_max_lufs: f32::NEG_INFINITY,
            momentary_range_lu: 0.0,
            short_term_range_lu: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LoudnessAnalysisResult, LoudnessChannelWeightSource, LoudnessMeter, LoudnessMeterConfig,
        LoudnessSampleRateSupport, RUNTIME_MOMENTARY_TAIL_POINTS, RUNTIME_SHORT_TERM_TAIL_POINTS,
    };
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue, AnalysisStage,
    };
    use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, SampleRate};

    fn sine(sample_rate: u32, frequency: f32, amplitude: f32, seconds: f32) -> AudioBuffer {
        let frames = (sample_rate as f32 * seconds).round() as usize;
        let mut samples = Vec::with_capacity(frames);
        for index in 0..frames {
            let t = index as f32 / sample_rate as f32;
            samples.push(amplitude * (core::f32::consts::TAU * frequency * t).sin());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn sine_sequence(sample_rate: u32, sections: &[(f32, f32, f32)]) -> AudioBuffer {
        let mut samples = Vec::new();
        for (frequency, amplitude, seconds) in sections {
            samples
                .extend_from_slice(sine(sample_rate, *frequency, *amplitude, *seconds).samples());
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
    }

    fn loudness_metrics(result: &LoudnessAnalysisResult) -> Vec<AnalysisMetricValue> {
        vec![
            AnalysisMetricValue::new("integrated_lufs", result.integrated_lufs),
            AnalysisMetricValue::new("true_peak_dbtp", result.true_peak_dbtp),
            AnalysisMetricValue::new("loudness_range_lu", result.loudness_range_lu),
            AnalysisMetricValue::new("confidence", result.confidence.0),
            AnalysisMetricValue::new("momentary_range_lu", result.dynamics.momentary_range_lu),
        ]
    }

    fn loudness_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:quiet-sine",
                    AnalysisCorpusFamily::Loudness,
                    "Quiet tonal loudness reference",
                ),
                sine(48_000, 1_000.0, 0.1, 4.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "true_peak_dbtp",
                    Some(-20.5),
                    Some(-19.5),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:loud-sine",
                    AnalysisCorpusFamily::Loudness,
                    "Loud tonal loudness reference",
                ),
                sine(48_000, 1_000.0, 0.5, 4.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "true_peak_dbtp",
                    Some(-6.5),
                    Some(-5.5),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "loudness:level-step",
                    AnalysisCorpusFamily::Loudness,
                    "Two-section level-step range reference",
                ),
                sine_sequence(48_000, &[(440.0, 0.08, 4.0), (440.0, 0.35, 4.0)]),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "loudness_range_lu",
                    Some(5.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "momentary_range_lu",
                    Some(5.0),
                    None,
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "confidence",
                    Some(0.9),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
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
        assert_eq!(result.aggregation.channel_count, ChannelCount(1));
        assert!(result
            .momentary_trace
            .points
            .iter()
            .all(|point| !point.loudness_lufs.is_finite()));
        assert!(result
            .short_term_trace
            .points
            .iter()
            .all(|point| !point.loudness_lufs.is_finite()));
    }

    #[test]
    fn louder_signal_matches_expected_decibel_scaling() {
        let quiet = sine(48_000, 1_000.0, 0.1, 4.0);
        let loud = sine(48_000, 1_000.0, 0.5, 4.0);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let quiet_result = meter.analyze(&quiet);
        let loud_result = meter.analyze(&loud);
        let expected_delta = 20.0 * (0.5f32 / 0.1f32).log10();

        assert!(loud_result.integrated_lufs > quiet_result.integrated_lufs);
        assert!(loud_result.true_peak_dbtp > quiet_result.true_peak_dbtp);
        assert!(loud_result.confidence.0 > 0.9);
        assert!(loud_result.dynamics.momentary_max_lufs > quiet_result.dynamics.momentary_max_lufs);
        assert!(
            (loud_result.integrated_lufs - quiet_result.integrated_lufs - expected_delta).abs()
                < 0.3
        );
        assert!(
            (loud_result.true_peak_dbtp - quiet_result.true_peak_dbtp - expected_delta).abs()
                < 0.05
        );
    }

    #[test]
    fn non_native_input_rate_is_resampled_without_material_drift() {
        let supported = sine(48_000, 440.0, 0.2, 4.0);
        let unsupported = sine(44_100, 440.0, 0.2, 4.0);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let supported_result = meter.analyze(&supported);
        let unsupported_result = meter.analyze(&unsupported);

        assert_eq!(
            supported_result.aggregation.sample_rate_support,
            LoudnessSampleRateSupport::Native48kKWeighted
        );
        assert_eq!(
            unsupported_result.aggregation.sample_rate_support,
            LoudnessSampleRateSupport::ResampledTo48kKWeighted
        );
        assert!(supported_result.confidence.0 > unsupported_result.confidence.0);
        assert!(unsupported_result.confidence.0 >= 0.85);
        assert!(
            (supported_result.integrated_lufs - unsupported_result.integrated_lufs).abs() < 1.0
        );
    }

    #[test]
    fn low_profile_produces_finite_results() {
        let audio = sine(48_000, 440.0, 0.3, 4.0);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::low());
        let result = meter.analyze(&audio);

        assert!(result.integrated_lufs.is_finite());
        assert!(result.true_peak_dbtp.is_finite());
        assert!(result.confidence.0 > 0.0);
        assert!(!result.momentary_trace.points.is_empty());
    }

    #[test]
    fn stereo_inputs_use_explicit_equal_weight_aggregation() {
        let mono = sine(48_000, 440.0, 0.25, 4.0);
        let stereo_samples: Vec<f32> = mono
            .samples()
            .iter()
            .flat_map(|sample| [*sample, *sample])
            .collect();
        let stereo = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            stereo_samples,
        );
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let mono_result = meter.analyze(&mono);
        let stereo_result = meter.analyze(&stereo);

        assert_eq!(
            stereo_result.aggregation.channel_weight_source,
            LoudnessChannelWeightSource::StereoEqualWeight
        );
        assert_eq!(stereo_result.channels.len(), 2);
        assert!(
            (stereo_result.integrated_lufs - mono_result.integrated_lufs - 3.0103).abs() < 0.25
        );
        assert!((mono_result.true_peak_dbtp - stereo_result.true_peak_dbtp).abs() < 0.1);
        assert_eq!(
            stereo_result.short_term_trace.window_seconds,
            LoudnessMeterConfig::default().short_term_seconds
        );
    }

    #[test]
    fn generic_multichannel_layout_uses_deterministic_fallback_weights() {
        let mono = sine(48_000, 440.0, 0.2, 4.0);
        let quad_samples: Vec<f32> = mono
            .samples()
            .iter()
            .flat_map(|sample| [*sample, *sample, *sample, *sample])
            .collect();
        let quad = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Count(ChannelCount(4)),
            quad_samples,
        );
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
        let mono_result = meter.analyze(&mono);
        let result = meter.analyze(&quad);

        assert_eq!(
            result.aggregation.channel_weight_source,
            LoudnessChannelWeightSource::GenericCountFallback
        );
        assert_eq!(result.channels.len(), 4);
        assert!(result.channels.iter().all(|channel| channel.weight == 1.0));
        assert!(result.confidence.0 < 1.0);
        assert!((result.integrated_lufs - mono_result.integrated_lufs - 6.0206).abs() < 0.35);
    }

    #[test]
    fn non_48k_analysis_rate_reports_unweighted_fallback() {
        let audio = sine(44_100, 1_000.0, 0.2, 4.0);
        let mut config = LoudnessMeterConfig::default();
        config.analysis_sample_rate = SampleRate(44_100);
        let mut fallback_meter = LoudnessMeter::new(config);
        let mut default_meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let fallback_result = fallback_meter.analyze(&audio);
        let default_result = default_meter.analyze(&audio);

        assert_eq!(
            fallback_result.aggregation.sample_rate_support,
            LoudnessSampleRateSupport::UnweightedFallback
        );
        assert_eq!(fallback_result.aggregation.true_peak_oversample_factor, 4);
        assert!(fallback_result.confidence.0 < default_result.confidence.0);
        assert!(fallback_result.integrated_lufs.is_finite());
    }

    #[test]
    fn harness_loudness_cases_meet_frozen_acceptance_thresholds() {
        let cases = loudness_acceptance_cases();
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let report =
            run_audio_acceptance_harness(&cases, |audio| meter.analyze(audio), loudness_metrics);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert!(report
            .cases
            .iter()
            .all(|case| case.status == AcceptanceStatus::Pass));
    }

    #[test]
    fn frozen_loudness_acceptance_report_remains_interpretable_for_closeout() {
        let cases = loudness_acceptance_cases();
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

        let report =
            run_audio_acceptance_harness(&cases, |audio| meter.analyze(audio), loudness_metrics);

        println!("loudness_acceptance_report={:#?}", report);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert_eq!(report.cases.len(), 3);
    }

    #[test]
    fn loudness_traces_capture_level_step_and_dynamics_summary() {
        let audio = sine_sequence(48_000, &[(440.0, 0.08, 4.0), (440.0, 0.35, 4.0)]);
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
        let result = meter.analyze(&audio);

        assert!(result.momentary_trace.points.len() > result.short_term_trace.points.len());
        assert!(result.momentary_trace.points.len() > 10);
        assert!(result.short_term_trace.points.len() >= 2);
        assert!(result.dynamics.momentary_max_lufs >= result.integrated_lufs);
        assert!(result.dynamics.short_term_max_lufs >= result.integrated_lufs);
        assert!(result.dynamics.momentary_range_lu > 0.0);
        assert!(result.dynamics.short_term_range_lu > 0.0);
        assert!(result.dynamics.target_offset_lu.is_finite());

        let loudest_momentary = result
            .momentary_trace
            .points
            .iter()
            .max_by(|lhs, rhs| {
                lhs.loudness_lufs
                    .partial_cmp(&rhs.loudness_lufs)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .expect("loudest momentary point");
        assert!(loudest_momentary.start_seconds >= 3.0);
    }

    #[test]
    fn runtime_diagnostics_summary_uses_bounded_recent_trace_tails() {
        let audio = sine_sequence(
            48_000,
            &[(440.0, 0.05, 3.0), (440.0, 0.2, 3.0), (440.0, 0.35, 3.0)],
        );
        let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
        let result = meter.analyze(&audio);
        let diagnostics = result.runtime_diagnostics_summary();

        assert!(diagnostics.recent_momentary.points.len() <= RUNTIME_MOMENTARY_TAIL_POINTS);
        assert!(diagnostics.recent_short_term.points.len() <= RUNTIME_SHORT_TERM_TAIL_POINTS);
        assert_eq!(
            diagnostics.current_momentary_lufs,
            diagnostics
                .recent_momentary
                .points
                .last()
                .expect("recent momentary point")
                .loudness_lufs
        );
        assert_eq!(
            diagnostics.current_short_term_lufs,
            diagnostics
                .recent_short_term
                .points
                .last()
                .expect("recent short-term point")
                .loudness_lufs
        );
        assert_eq!(diagnostics.integrated_lufs, result.integrated_lufs);
        assert_eq!(diagnostics.true_peak_dbtp, result.true_peak_dbtp);
        assert_eq!(
            diagnostics.target_offset_lu,
            result.dynamics.target_offset_lu
        );
        assert_eq!(
            diagnostics.momentary_max_lufs,
            result.dynamics.momentary_max_lufs
        );
        assert_eq!(
            diagnostics.short_term_max_lufs,
            result.dynamics.short_term_max_lufs
        );
    }
}
