use super::*;
use crate::{LoudnessChannelWeightSource, LoudnessSampleRateSupport};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};

#[test]
fn silence_reports_negative_infinity() {
    let audio = AudioBuffer::new(SampleRate(48_000), ChannelLayout::Mono, FrameCount(48_000));
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
        (loud_result.integrated_lufs - quiet_result.integrated_lufs - expected_delta).abs() < 0.3
    );
    assert!(
        (loud_result.true_peak_dbtp - quiet_result.true_peak_dbtp - expected_delta).abs() < 0.05
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
    assert!((supported_result.integrated_lufs - unsupported_result.integrated_lufs).abs() < 1.0);
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
    let stereo =
        AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Stereo, stereo_samples);
    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());

    let mono_result = meter.analyze(&mono);
    let stereo_result = meter.analyze(&stereo);

    assert_eq!(
        stereo_result.aggregation.channel_weight_source,
        LoudnessChannelWeightSource::StereoEqualWeight
    );
    assert_eq!(stereo_result.channels.len(), 2);
    assert!((stereo_result.integrated_lufs - mono_result.integrated_lufs - 3.0103).abs() < 0.25);
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
    let config = LoudnessMeterConfig {
        analysis_sample_rate: SampleRate(44_100),
        ..LoudnessMeterConfig::default()
    };
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
