//! Tests for STFT, spectrogram analysis, and mel-spectrogram functionality.

use crate::{
    hann_window, FrameCount, LogCompression, MelFilterNorm, MelFilterbankConfig, MelScale,
    MelSpectrogramConfig, Spectrogram, Stft, StftConfig,
};
use signal_primitives::SampleRate;

#[test]
fn hann_window_has_expected_shape() {
    let window = hann_window(8);
    assert_eq!(window.len(), 8);
    assert_eq!(window[0], 0.0);
    assert!(window[4] > window[1]);
}

#[test]
fn stft_produces_frames_and_bins() {
    let stft = Stft::new(StftConfig::new(8, 4));
    let spectrogram = stft.analyze_mono(SampleRate(8_000), &[1.0; 16]);

    assert!(!spectrogram.is_empty());
    assert_eq!(spectrogram.bins(), 5);
    assert_eq!(spectrogram.frames.len(), 3);
    assert_eq!(spectrogram.frames[0].phases.len(), 5);
}

#[test]
fn streaming_stft_matches_offline_across_irregular_chunks() {
    let config = StftConfig::new(32, 12);
    let stft = Stft::new(config);
    let samples: Vec<f32> = (0..137).map(|index| (index as f32 * 0.071).sin()).collect();

    let offline = stft.analyze_mono(SampleRate(48_000), &samples);

    let mut streaming = stft.streaming();
    let mut frames = Vec::new();
    frames.extend(streaming.process_chunk(&samples[..7]));
    frames.extend(streaming.process_chunk(&samples[7..29]));
    frames.extend(streaming.process_chunk(&samples[29..74]));
    frames.extend(streaming.process_chunk(&samples[74..]));
    frames.extend(streaming.finish());

    assert_eq!(frames, offline.frames);
}

#[test]
fn streaming_stft_flushes_final_zero_padded_frame() {
    let config = StftConfig::new(8, 4);
    let stft = Stft::new(config);
    let samples = vec![1.0f32; 10];

    let offline = stft.analyze_mono(SampleRate(8_000), &samples);

    let mut streaming = stft.streaming();
    let mut frames = streaming.process_chunk(&samples);
    frames.extend(streaming.finish());

    assert_eq!(frames.len(), 2);
    assert_eq!(frames, offline.frames);
}

#[test]
fn spectral_centroid_near_1khz_for_sine() {
    let sample_rate = 48_000;
    let stft = Stft::new(StftConfig::new(4096, 2048));
    let mut samples = vec![0.0f32; sample_rate];
    for (index, sample) in samples.iter_mut().enumerate() {
        let t = index as f32 / sample_rate as f32;
        *sample = (core::f32::consts::TAU * 1000.0 * t).sin();
    }
    let spectrogram = stft.analyze_mono(SampleRate(sample_rate as u32), &samples);
    let centroid = spectrogram.spectral_centroid();
    assert!(
        centroid > 800.0 && centroid < 1200.0,
        "centroid was {centroid}"
    );
}

#[test]
fn spectral_centroid_is_zero_for_empty_spectrogram() {
    let spectrogram = Spectrogram {
        sample_rate: SampleRate(48_000),
        config: StftConfig::new(1024, 512),
        frames: Vec::new(),
    };
    assert_eq!(spectrogram.spectral_centroid(), 0.0);
}

#[test]
fn chroma_peaks_near_expected_pitch_class() {
    let sample_rate = 48_000;
    let stft = Stft::new(StftConfig::new(4096, 2048));
    let mut samples = vec![0.0f32; sample_rate];

    for (index, sample) in samples.iter_mut().enumerate() {
        let t = index as f32 / sample_rate as f32;
        *sample = (core::f32::consts::TAU * 440.0 * t).sin();
    }

    let spectrogram = stft.analyze_mono(SampleRate(sample_rate as u32), &samples);
    let chroma = spectrogram.chroma();
    let (best_index, _) = chroma
        .iter()
        .copied()
        .enumerate()
        .max_by(|(_, lhs), (_, rhs)| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap();

    assert_eq!(best_index, 9);
}

// --- Mel-spectrogram integration tests ---

#[test]
fn mel_spectrogram_has_correct_dimensions() {
    // 16kHz mono, 512 FFT, 256 hop — matches the Monkey AudioFeatureContract.
    let sample_rate = 16_000u32;
    let samples: Vec<f32> = (0..sample_rate as usize)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (core::f32::consts::TAU * 1000.0 * t).sin()
        })
        .collect();

    let stft = Stft::new(StftConfig {
        window_size: FrameCount(512),
        hop_size: FrameCount(256),
        compute_phases: false,
    });
    let spectrogram = stft.analyze_mono(SampleRate(sample_rate), &samples);

    let mel_config = MelSpectrogramConfig {
        filterbank: MelFilterbankConfig {
            mel_bin_count: 96,
            low_frequency_hz: 0.0,
            high_frequency_hz: 8000.0,
            mel_scale: MelScale::Slaney,
            normalize: MelFilterNorm::None,
        },
        log_compression: LogCompression::EssentiaLog10,
    };

    let mel = spectrogram.to_mel_spectrogram(&mel_config);

    // Each frame should have exactly 96 mel bins.
    assert!(
        !mel.frames.is_empty(),
        "mel spectrogram should not be empty"
    );
    for frame in &mel.frames {
        assert_eq!(frame.len(), 96, "each mel frame should have 96 bins");
    }
}

#[test]
fn mel_spectrogram_1khz_energy_in_expected_bins() {
    // A 1kHz sine at 16kHz sample rate should concentrate energy around
    // the mel bin corresponding to 1000 Hz.
    let sample_rate = 16_000u32;
    let samples: Vec<f32> = (0..sample_rate as usize)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (core::f32::consts::TAU * 1000.0 * t).sin()
        })
        .collect();

    let stft = Stft::new(StftConfig {
        window_size: FrameCount(512),
        hop_size: FrameCount(256),
        compute_phases: false,
    });
    let spectrogram = stft.analyze_mono(SampleRate(sample_rate), &samples);

    let mel_config = MelSpectrogramConfig {
        filterbank: MelFilterbankConfig {
            mel_bin_count: 96,
            low_frequency_hz: 0.0,
            high_frequency_hz: 8000.0,
            mel_scale: MelScale::Slaney,
            normalize: MelFilterNorm::None,
        },
        log_compression: LogCompression::None,
    };

    let mel = spectrogram.to_mel_spectrogram(&mel_config);

    // Find the mel bin with the most energy in the first frame.
    let frame = &mel.frames[0];
    let (peak_bin, peak_energy) = frame
        .iter()
        .copied()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap();

    // 1000 Hz is the break frequency in Slaney scale.  With 96 bins from
    // 0 to 8000 Hz, the 1 kHz energy should land roughly in the first
    // third of the bins (Slaney linear range is 0–1000 Hz = ~15 mel bins).
    assert!(
        peak_energy > 0.0,
        "peak energy should be positive for 1kHz sine"
    );
    assert!(
        peak_bin < 48,
        "1kHz energy should be in the lower half; got bin {peak_bin}"
    );
}

#[test]
fn empty_spectrogram_produces_empty_mel() {
    let spectrogram = Spectrogram {
        sample_rate: SampleRate(16_000),
        config: StftConfig::new(512, 256),
        frames: Vec::new(),
    };

    let mel_config = MelSpectrogramConfig {
        filterbank: MelFilterbankConfig {
            mel_bin_count: 96,
            low_frequency_hz: 0.0,
            high_frequency_hz: 8000.0,
            mel_scale: MelScale::Slaney,
            normalize: MelFilterNorm::None,
        },
        log_compression: LogCompression::EssentiaLog10,
    };

    let mel = spectrogram.to_mel_spectrogram(&mel_config);
    assert!(mel.frames.is_empty());
}
