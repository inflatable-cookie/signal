//! Absolute known-answer tests against BS.1770-4 / EBU R 128 reference
//! values, so a calibration bug cannot hide behind relative-delta checks.

use super::*;
use signal_analysis::AnalysisStage;
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

/// Stereo buffer with a sine in the left channel and silence in the right.
fn stereo_left_sine(sample_rate: u32, frequency: f32, amplitude: f32, seconds: f32) -> AudioBuffer {
    let frames = (sample_rate as f32 * seconds).round() as usize;
    let mut samples = Vec::with_capacity(frames * 2);
    for index in 0..frames {
        let t = index as f32 / sample_rate as f32;
        samples.push(amplitude * (core::f32::consts::TAU * frequency * t).sin());
        samples.push(0.0);
    }
    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Stereo, samples)
}

/// Mono sine with an explicit initial phase, in radians.
fn mono_sine_with_phase(
    sample_rate: u32,
    frequency: f32,
    amplitude: f32,
    phase: f32,
    seconds: f32,
) -> AudioBuffer {
    let frames = (sample_rate as f32 * seconds).round() as usize;
    let mut samples = Vec::with_capacity(frames);
    // f64 phase math: at fs/4 the f32 phase argument grows large enough that
    // rounding drifts samples off the exact +/- amplitude/sqrt(2) lattice.
    for index in 0..frames {
        let t = index as f64 / sample_rate as f64;
        let angle = core::f64::consts::TAU * frequency as f64 * t + phase as f64;
        samples.push(amplitude * angle.sin() as f32);
    }
    AudioBuffer::from_interleaved(SampleRate(sample_rate), ChannelLayout::Mono, samples)
}

// BS.1770-4 reference point: a 0 dBFS 997 Hz sine applied to one channel of
// a stereo pair indicates -3.01 LKFS. Math: mean square of a full-scale sine
// is 0.5 -> 10*log10(0.5) = -3.0103; the K-weighting gain at 997 Hz
// (~ +0.691 dB) cancels the -0.691 offset in the loudness formula; the
// silent right channel contributes zero energy under equal stereo weights.
#[test]
fn full_scale_997hz_left_only_stereo_sine_reads_minus_3_01_lufs() {
    let audio = stereo_left_sine(48_000, 997.0, 1.0, 10.0);
    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
    let result = meter.analyze(&audio);

    assert!(
        (result.integrated_lufs + 3.0103).abs() < 0.1,
        "integrated_lufs = {}, expected -3.01 +/- 0.1",
        result.integrated_lufs
    );
}

// Same tone scaled to -23 dBFS amplitude: integrated loudness shifts by
// exactly the amplitude change, -3.0103 - 23 = -26.0103 LUFS.
#[test]
fn minus_23dbfs_997hz_left_only_stereo_sine_reads_minus_26_01_lufs() {
    let amplitude = 10.0f32.powf(-23.0 / 20.0);
    let audio = stereo_left_sine(48_000, 997.0, amplitude, 10.0);
    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
    let result = meter.analyze(&audio);

    assert!(
        (result.integrated_lufs + 26.0103).abs() < 0.1,
        "integrated_lufs = {}, expected -26.01 +/- 0.1",
        result.integrated_lufs
    );
}

// Inter-sample peak case: a 12 kHz sine (fs/4) with initial phase pi/4
// only ever samples the continuous waveform at +/- amplitude/sqrt(2), so
// the raw sample peak under-reads the true peak by 3.01 dB. With amplitude
// 0.5 the sample peak is 20*log10(0.5/sqrt(2)) = -9.03 dBFS while the
// continuous (true) peak is 20*log10(0.5) = -6.02 dBTP. The 4x oversampled
// FIR must recover most of that gap; linear interpolation cannot exceed the
// sample peak at all.
#[test]
fn true_peak_recovers_inter_sample_peak_of_quarter_rate_sine() {
    let amplitude = 0.5f32;
    let audio = mono_sine_with_phase(
        48_000,
        12_000.0,
        amplitude,
        core::f32::consts::FRAC_PI_4,
        1.0,
    );
    let sample_peak_db = 20.0
        * audio
            .samples()
            .iter()
            .fold(0.0f32, |acc, sample| acc.max(sample.abs()))
            .log10();
    let analytic_true_peak_db = 20.0 * amplitude.log10();

    let mut meter = LoudnessMeter::new(LoudnessMeterConfig::default());
    let result = meter.analyze(&audio);

    // Sanity: the construction really does hide the peak between samples.
    assert!(
        (sample_peak_db - (analytic_true_peak_db - 3.0103)).abs() < 0.05,
        "sample peak {} should sit 3.01 dB below the true peak {}",
        sample_peak_db,
        analytic_true_peak_db
    );
    assert!(
        result.true_peak_dbtp > sample_peak_db + 2.5,
        "true_peak_dbtp = {} should exceed sample peak {} by a clear margin",
        result.true_peak_dbtp,
        sample_peak_db
    );
    assert!(
        (result.true_peak_dbtp - analytic_true_peak_db).abs() < 0.3,
        "true_peak_dbtp = {}, expected {} +/- 0.3",
        result.true_peak_dbtp,
        analytic_true_peak_db
    );
}

// LRA relative-gate case (EBU Tech 3342). Non-overlapping 3 s short-term
// windows (hop = 3 s) over 997 Hz segments:
//   quiet 6 s at amp 0.005 -> 2 windows at ~ -49.0 LUFS
//   mid   9 s at amp 0.25  -> 3 windows at ~ -15.05 LUFS
//   loud  9 s at amp 0.5   -> 3 windows at ~  -9.03 LUFS
// Absolute gate (-70) keeps all 8. Power mean of the 8 energies is
// (2*1.25e-5 + 3*0.03125 + 3*0.125)/8 = 0.0586 -> -12.3 LUFS, so the
// relative gate at -32.3 drops both quiet windows. Remaining distribution
// [-15.05 x3, -9.03 x3] gives 10th pct -15.05, 95th pct -9.03,
// LRA = 6.02 LU. Without the relative gate the 10th percentile lands on a
// quiet window and LRA balloons to ~40 LU.
#[test]
fn loudness_range_applies_relative_gate_to_quiet_segments() {
    let config = LoudnessMeterConfig {
        hop_seconds: 3.0,
        ..LoudnessMeterConfig::default()
    };
    let audio = sine_sequence(
        48_000,
        &[(997.0, 0.005, 6.0), (997.0, 0.25, 9.0), (997.0, 0.5, 9.0)],
    );
    let mut meter = LoudnessMeter::new(config);
    let result = meter.analyze(&audio);

    assert!(
        (result.loudness_range_lu - 6.0206).abs() < 0.7,
        "loudness_range_lu = {}, expected 6.02 +/- 0.7",
        result.loudness_range_lu
    );
    assert!(
        result.loudness_range_lu < 15.0,
        "loudness_range_lu = {}: relative gate failed to drop quiet windows",
        result.loudness_range_lu
    );
}
