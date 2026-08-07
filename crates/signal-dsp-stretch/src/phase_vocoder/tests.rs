use super::*;

fn bin_centered_sine(bin: usize, window_size: usize) -> Vec<Sample> {
    (0..window_size)
        .map(|index| (std::f32::consts::TAU * bin as f32 * index as f32 / window_size as f32).sin())
        .collect()
}

fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
    let margin = samples.len() / 8;
    let interior = &samples[margin..samples.len() - margin];
    let crossings = interior
        .windows(2)
        .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
        .count();
    crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
}

fn rms(samples: &[Sample]) -> f32 {
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len().max(1) as f32).sqrt()
}

fn boundary_content_probe(len: usize, edge_frames: usize) -> Vec<Sample> {
    let mut input = vec![0.0; len];
    input[..edge_frames].fill(0.5);
    input[len - edge_frames..].fill(-0.5);
    input
}

fn sample_bit_hash(samples: &[Sample]) -> u64 {
    samples.iter().fold(0xcbf2_9ce4_8422_2325, |hash, sample| {
        sample
            .to_bits()
            .to_le_bytes()
            .into_iter()
            .fold(hash, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
            })
    })
}

/// `A1` byte-exactness control. The overlap law must be a no-op wherever
/// the configured geometry already satisfies it, so every ratio through
/// `3.0` at the retained `2048/512` geometry keeps the pre-correction
/// analysis hop and therefore the pre-correction output.
///
/// This is asserted structurally rather than by output hash because f32
/// render output differs between optimization profiles, so an absolute
/// hash is only valid in the profile that captured it.
#[test]
fn overlap_safe_analysis_hop_is_a_no_op_through_ratio_three() {
    for ratio in [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0] {
        assert_eq!(
            overlap_safe_analysis_hop(512, ratio, 2_048),
            512,
            "ratio {ratio} must keep the configured hop"
        );
    }
}

/// `A1`. Above ratio `3.0` the law reduces the hop so the synthesis hop
/// stays inside `0.75 * window_size`.
#[test]
fn overlap_safe_analysis_hop_bounds_the_synthesis_hop() {
    let bound = MAX_SYNTHESIS_HOP_WINDOW_FRACTION * 2_048.0;
    for (ratio, expected) in [(3.5, 438), (4.0, 384), (6.0, 256), (8.0, 192), (16.0, 96)] {
        let hop = overlap_safe_analysis_hop(512, ratio, 2_048);
        assert_eq!(hop, expected, "ratio {ratio}");
        assert!(
            hop as f64 * ratio <= bound,
            "ratio {ratio}: synthesis hop {} passes the bound {bound}",
            hop as f64 * ratio
        );
    }
}

/// The law never enlarges a caller's hop and never returns zero.
#[test]
fn overlap_safe_analysis_hop_stays_within_caller_bounds() {
    assert_eq!(overlap_safe_analysis_hop(128, 64.0, 512), 6);
    assert_eq!(overlap_safe_analysis_hop(1, 1_000.0, 64), 1);
    assert_eq!(overlap_safe_analysis_hop(512, f64::NAN, 2_048), 512);
    assert_eq!(overlap_safe_analysis_hop(512, 0.0, 2_048), 512);
    assert_eq!(overlap_safe_analysis_hop(64, 0.5, 2_048), 64);
}

#[test]
fn phase_vocoder_bit_exact_baseline() {
    let input = (0..8_192)
        .map(|index| {
            let fundamental = (std::f32::consts::TAU * 11.0 * index as f32 / 2_048.0).sin() * 0.6;
            let partial = (std::f32::consts::TAU * 37.0 * index as f32 / 2_048.0).sin() * 0.2;
            let impulse = if index % 997 == 0 { 0.75 } else { 0.0 };
            fundamental + partial + impulse
        })
        .collect::<Vec<_>>();
    let output = transient_reset_phase_vocoder(&input, 12_288, 1.5, 2_048, 512);

    assert_eq!(sample_bit_hash(&output), 0x8255_b183_11f7_78f9);
}

#[test]
fn phase_vocoder_boundary_expansion_preserves_head_and_tail_content() {
    let input = boundary_content_probe(48_000, 384);
    let output = transient_reset_phase_vocoder(&input, 96_000, 2.0, 2_048, 512);
    let edge_span = 2_048;

    assert!(rms(&output[..edge_span]) > 0.01);
    assert!(rms(&output[output.len() - edge_span..]) > 0.01);
}

#[test]
fn phase_vocoder_boundary_compression_preserves_head_and_tail_content() {
    let input = boundary_content_probe(48_000, 384);
    let output = transient_reset_phase_vocoder(&input, 24_000, 0.5, 2_048, 512);
    let edge_span = 1_024;

    assert!(rms(&output[..edge_span]) > 0.01);
    assert!(rms(&output[output.len() - edge_span..]) > 0.01);
}

#[test]
fn tracks_local_spectral_peaks_for_current_frame() {
    let window_size = 256;
    let target_bin = 17;
    let input = bin_centered_sine(target_bin, window_size);
    let config = PhaseVocoderConfig::new(&input, window_size, 1.0, window_size, window_size / 4);
    let mut engine = DraftPhaseVocoder::new(config, PhasePropagationMode::IndependentBins);

    engine.analyze_frame(&input, 0);
    engine.track_spectral_peaks(0);

    assert!(
        engine
            .analysis
            .current_peaks
            .iter()
            .any(|peak| peak.bin.abs_diff(target_bin) <= 1),
        "expected a peak near bin {target_bin}, got {:?}",
        engine.analysis.current_peaks
    );
}

#[test]
fn transient_reset_detector_flags_energy_and_flux_jump() {
    let window_size = 256;
    let mut input = vec![0.0; window_size * 2];
    for sample in &mut input[window_size..] {
        *sample = 1.0;
    }
    let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, window_size, window_size);
    let mut engine =
        DraftPhaseVocoder::new(config, PhasePropagationMode::IdentityLockedTransientReset);

    engine.analyze_frame(&input, 0);
    engine.track_spectral_peaks(0);
    assert!(!engine.analysis.transient_reset_current_frame);

    engine.analyze_frame(&input, 1);
    engine.track_spectral_peaks(1);
    assert!(engine.analysis.transient_reset_current_frame);
}

#[test]
fn identity_locking_preserves_peak_neighborhood_phase_offsets() {
    let input = vec![0.0; 512];
    let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, 512, 128);
    let mut engine = DraftPhaseVocoder::new(config, PhasePropagationMode::IdentityLocked);
    engine.analysis.current_peaks.push(SpectralPeak {
        bin: 10,
        magnitude: 1.0,
    });
    engine.analysis.current_phases[9] = 0.20;
    engine.analysis.current_phases[10] = 0.50;
    engine.analysis.current_phases[11] = 0.90;
    engine.propagation.synthesis_phase[10] = 1.25;

    engine.lock_phase_to_peaks();

    assert!((wrap_phase(engine.propagation.synthesis_phase[9] - 0.95)).abs() < 1.0e-6);
    assert!((wrap_phase(engine.propagation.synthesis_phase[10] - 1.25)).abs() < 1.0e-6);
    assert!((wrap_phase(engine.propagation.synthesis_phase[11] - 1.65)).abs() < 1.0e-6);
}

#[test]
fn draft_phase_vocoder_keeps_independent_bin_baseline() {
    let input = bin_centered_sine(9, 4096);
    let draft = phase_vocoder(&input, 6144, 1.5, 512, 128);
    let baseline = run_phase_vocoder(
        &input,
        6144,
        1.5,
        512,
        128,
        PhasePropagationMode::IndependentBins,
    );

    assert_eq!(draft, baseline);
}

#[test]
fn phase_locked_prototype_honors_output_length_contract() {
    let input = bin_centered_sine(11, 8192);
    for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
        let target_len = (input.len() as f64 * ratio).round() as usize;
        let output = phase_locked_phase_vocoder(&input, target_len, ratio, 1024, 256);
        assert_eq!(output.len(), target_len, "ratio {ratio}");
    }
}

#[test]
fn transient_reset_prototype_honors_output_length_contract() {
    let input = bin_centered_sine(11, 8192);
    for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
        let target_len = (input.len() as f64 * ratio).round() as usize;
        let output = transient_reset_phase_vocoder(&input, target_len, ratio, 1024, 256);
        assert_eq!(output.len(), target_len, "ratio {ratio}");
    }
}

#[test]
fn transient_reset_uses_phase_locking_for_time_compression() {
    let ratio = 0.75;
    let input = bin_centered_sine(11, 8192);
    let target_len = (input.len() as f64 * ratio).round() as usize;

    assert_eq!(
        transient_reset_phase_vocoder(&input, target_len, ratio, 1024, 256),
        phase_locked_phase_vocoder(&input, target_len, ratio, 1024, 256)
    );
}

#[test]
fn phase_locked_prototype_preserves_tonal_pitch_near_draft_baseline() {
    let sample_rate = 48_000.0;
    let frequency_hz = 468.75;
    let input = (0..48_000)
        .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate).sin())
        .collect::<Vec<_>>();

    for ratio in [0.75, 1.5, 2.0] {
        let target_len = (input.len() as f64 * ratio).round() as usize;
        let draft = phase_vocoder(&input, target_len, ratio, 2048, 512);
        let locked = phase_locked_phase_vocoder(&input, target_len, ratio, 2048, 512);
        let draft_frequency = dominant_frequency_hz(&draft, sample_rate);
        let locked_frequency = dominant_frequency_hz(&locked, sample_rate);

        assert!(
            (locked_frequency - frequency_hz).abs() <= (draft_frequency - frequency_hz).abs() + 3.0,
            "ratio {ratio}: locked frequency {locked_frequency} Hz regressed from draft {draft_frequency} Hz"
        );
    }
}

#[test]
fn transient_reset_prototype_preserves_tonal_pitch_near_draft_baseline() {
    let sample_rate = 48_000.0;
    let frequency_hz = 468.75;
    let input = (0..48_000)
        .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate).sin())
        .collect::<Vec<_>>();

    for ratio in [0.75, 1.5, 2.0] {
        let target_len = (input.len() as f64 * ratio).round() as usize;
        let draft = phase_vocoder(&input, target_len, ratio, 2048, 512);
        let reset = transient_reset_phase_vocoder(&input, target_len, ratio, 2048, 512);
        let draft_frequency = dominant_frequency_hz(&draft, sample_rate);
        let reset_frequency = dominant_frequency_hz(&reset, sample_rate);

        assert!(
            (reset_frequency - frequency_hz).abs()
                <= (draft_frequency - frequency_hz).abs() + 3.0,
            "ratio {ratio}: reset frequency {reset_frequency} Hz regressed from draft {draft_frequency} Hz"
        );
    }
}

#[test]
fn phase_vocoder_output_is_deterministic_with_peak_tracking() {
    let input = bin_centered_sine(9, 4096);
    let first = phase_vocoder(&input, 6144, 1.5, 512, 128);
    let repeated = phase_vocoder(&input, 6144, 1.5, 512, 128);

    assert_eq!(first, repeated);
}
