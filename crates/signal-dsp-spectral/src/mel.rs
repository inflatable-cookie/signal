//! Mel-spectrogram projection from linear-frequency spectrograms.
//!
//! This module provides mel-scale filterbank construction and log-compression
//! for converting STFT magnitude spectra into mel-frequency representations
//! commonly used in audio ML and analysis pipelines.

use crate::Spectrogram;
use signal_primitives::SampleRate;

/// Mel frequency scale variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MelScale {
    /// Standard O'Shaughnessy formula: `2595 * log10(1 + f/700)`.
    Htk,
    /// Slaney / librosa formula: linear spacing below 1000 Hz, logarithmic
    /// above.  Matches librosa and Essentia conventions.
    Slaney,
}

/// Log-compression applied after mel-filterbank projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogCompression {
    /// Essentia-style: `log10(1 + 10000 * x)`.
    EssentiaLog10,
    /// Natural log: `ln(1 + x)`.
    NaturalLog,
    /// No compression — raw mel-frequency energy.
    None,
}

/// Normalization mode for mel filterbank triangles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MelFilterNorm {
    /// No normalization — each triangle peaks at 1.0 regardless of width.
    None,
    /// Slaney / Essentia `unit_tri` normalization: each triangle is divided
    /// by its bandwidth in Hz (`2 / (high_hz - low_hz)`), giving narrower
    /// low-frequency filters proportionally higher weights.  This matches
    /// Essentia's `MelBands(normalize='unit_tri')` and librosa's
    /// `norm='slaney'`.
    UnitTri,
}

/// Configuration for a mel filterbank.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MelFilterbankConfig {
    pub mel_bin_count: usize,
    pub low_frequency_hz: f32,
    pub high_frequency_hz: f32,
    pub mel_scale: MelScale,
    /// Triangle normalization mode.  Use [`MelFilterNorm::UnitTri`] to match
    /// Essentia's `MelBands(normalize='unit_tri')`.
    pub normalize: MelFilterNorm,
}

/// Configuration for mel-spectrogram projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MelSpectrogramConfig {
    pub filterbank: MelFilterbankConfig,
    pub log_compression: LogCompression,
}

/// A mel-frequency spectrogram projected from a linear-frequency [`Spectrogram`].
#[derive(Clone, Debug, PartialEq)]
pub struct MelSpectrogram {
    pub config: MelSpectrogramConfig,
    /// Mel-frequency energy per frame: `frames[frame_index][mel_bin]`.
    pub frames: Vec<Vec<f32>>,
}

/// Convert Hz to mel scale.
pub fn hz_to_mel(hz: f32, scale: MelScale) -> f32 {
    match scale {
        MelScale::Htk => 2595.0 * (1.0 + hz / 700.0).log10(),
        MelScale::Slaney => {
            const BREAK_HZ: f32 = 1000.0;
            const LINEAR_SLOPE: f32 = 3.0 / 200.0; // 1 / (200/3)
            if hz < BREAK_HZ {
                hz * LINEAR_SLOPE
            } else {
                let log_step = (6.4f32).ln() / 27.0; // ln(6400/1000) / 27
                let min_log_mel = BREAK_HZ * LINEAR_SLOPE;
                min_log_mel + ((hz / BREAK_HZ).ln() / log_step)
            }
        }
    }
}

/// Convert mel scale to Hz.
pub fn mel_to_hz(mel: f32, scale: MelScale) -> f32 {
    match scale {
        MelScale::Htk => 700.0 * (10.0f32.powf(mel / 2595.0) - 1.0),
        MelScale::Slaney => {
            const BREAK_HZ: f32 = 1000.0;
            const LINEAR_SLOPE: f32 = 3.0 / 200.0;
            let min_log_mel = BREAK_HZ * LINEAR_SLOPE;
            if mel < min_log_mel {
                mel / LINEAR_SLOPE
            } else {
                let log_step = (6.4f32).ln() / 27.0;
                BREAK_HZ * ((mel - min_log_mel) * log_step).exp()
            }
        }
    }
}

/// Build the mel filterbank matrix: `mel_bin_count` triangular filters
/// spanning `low_frequency_hz` to `high_frequency_hz`, projected onto
/// `fft_bin_count` linear-frequency FFT bins.
///
/// Returns a `Vec` of length `mel_bin_count`, each containing `fft_bin_count`
/// weights (a sparse triangle, mostly zeros).
pub fn build_filterbank(
    config: &MelFilterbankConfig,
    sample_rate: SampleRate,
    fft_bin_count: usize,
) -> Vec<Vec<f32>> {
    let scale = config.mel_scale;
    let mel_low = hz_to_mel(config.low_frequency_hz, scale);
    let mel_high = hz_to_mel(config.high_frequency_hz, scale);
    let n_filters = config.mel_bin_count;

    // n_filters + 2 center frequencies (includes the two boundary edges).
    let mel_points: Vec<f32> = (0..n_filters + 2)
        .map(|i| mel_low + (mel_high - mel_low) * (i as f32 / (n_filters + 1) as f32))
        .collect();

    let hz_points: Vec<f32> = mel_points.iter().map(|&m| mel_to_hz(m, scale)).collect();

    // Map Hz center frequencies to fractional FFT bin indices.
    let fft_size = (fft_bin_count - 1) * 2; // window_size
    let bin_indices: Vec<f32> = hz_points
        .iter()
        .map(|&hz| hz * fft_size as f32 / sample_rate.0 as f32)
        .collect();

    let mut filterbank = vec![vec![0.0f32; fft_bin_count]; n_filters];

    for i in 0..n_filters {
        let left = bin_indices[i];
        let center = bin_indices[i + 1];
        let right = bin_indices[i + 2];

        // Optional unit_tri normalization: scale = 2 / (right_hz - left_hz).
        // In bin-index space this becomes: 2 / ((right - left) * hz_per_bin).
        let norm_scale = match config.normalize {
            MelFilterNorm::UnitTri => {
                let hz_per_bin = sample_rate.0 as f32 / fft_size as f32;
                let bandwidth_hz = (right - left) * hz_per_bin;
                if bandwidth_hz > 0.0 {
                    2.0 / bandwidth_hz
                } else {
                    1.0
                }
            }
            MelFilterNorm::None => 1.0,
        };

        for k in 0..fft_bin_count {
            let kf = k as f32;
            if kf > left && kf <= center && center > left {
                filterbank[i][k] = norm_scale * (kf - left) / (center - left);
            } else if kf > center && kf < right && right > center {
                filterbank[i][k] = norm_scale * (right - kf) / (right - center);
            }
        }
    }

    filterbank
}

/// Apply log compression to a mel energy value.
pub fn apply_compression(value: f32, compression: LogCompression) -> f32 {
    match compression {
        LogCompression::EssentiaLog10 => (1.0 + 10000.0 * value).log10(),
        LogCompression::NaturalLog => (1.0 + value).ln(),
        LogCompression::None => value,
    }
}

impl Spectrogram {
    /// Project this linear-frequency spectrogram onto a mel-frequency scale
    /// using a triangular filterbank.
    pub fn to_mel_spectrogram(&self, config: &MelSpectrogramConfig) -> MelSpectrogram {
        if self.frames.is_empty() {
            return MelSpectrogram {
                config: *config,
                frames: Vec::new(),
            };
        }

        let fft_bins = self.bins();
        let filterbank = build_filterbank(&config.filterbank, self.sample_rate, fft_bins);
        let n_mel = config.filterbank.mel_bin_count;

        let mel_frames: Vec<Vec<f32>> = self
            .frames
            .iter()
            .map(|frame| {
                let mut mel_energies = vec![0.0f32; n_mel];
                for (mel_idx, filter) in filterbank.iter().enumerate() {
                    let mut energy = 0.0f32;
                    for (k, &weight) in filter.iter().enumerate() {
                        if weight > 0.0 {
                            // Use squared magnitude (power spectrum) for mel energy.
                            let mag = frame.magnitudes.get(k).copied().unwrap_or(0.0);
                            energy += weight * mag * mag;
                        }
                    }
                    mel_energies[mel_idx] = apply_compression(energy, config.log_compression);
                }
                mel_energies
            })
            .collect();

        MelSpectrogram {
            config: *config,
            frames: mel_frames,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_primitives::SampleRate;

    #[test]
    fn slaney_mel_roundtrip_below_break() {
        // Below 1000 Hz, Slaney mel scale should be linear.
        let hz = 500.0f32;
        let mel = hz_to_mel(hz, MelScale::Slaney);
        let roundtrip = mel_to_hz(mel, MelScale::Slaney);
        assert!(
            (roundtrip - hz).abs() < 0.1,
            "roundtrip was {roundtrip}, expected {hz}"
        );
    }

    #[test]
    fn slaney_mel_roundtrip_above_break() {
        // Above 1000 Hz, Slaney mel scale should be logarithmic.
        let hz = 4000.0f32;
        let mel = hz_to_mel(hz, MelScale::Slaney);
        let roundtrip = mel_to_hz(mel, MelScale::Slaney);
        assert!(
            (roundtrip - hz).abs() < 1.0,
            "roundtrip was {roundtrip}, expected {hz}"
        );
    }

    #[test]
    fn htk_mel_roundtrip() {
        let hz = 4000.0f32;
        let mel = hz_to_mel(hz, MelScale::Htk);
        let roundtrip = mel_to_hz(mel, MelScale::Htk);
        assert!(
            (roundtrip - hz).abs() < 0.1,
            "roundtrip was {roundtrip}, expected {hz}"
        );
    }

    #[test]
    fn essentia_log10_compression_increases_values() {
        // EssentiaLog10: log10(1 + 10000 * x)
        // For x = 0.001: log10(1 + 10) = log10(11) ≈ 1.041
        let compressed = apply_compression(0.001, LogCompression::EssentiaLog10);
        assert!(
            compressed > 1.0 && compressed < 1.1,
            "compressed was {compressed}"
        );

        // For x = 0.0: log10(1 + 0) = 0
        let zero = apply_compression(0.0, LogCompression::EssentiaLog10);
        assert!((zero - 0.0).abs() < 1e-6);
    }

    #[test]
    fn filterbank_triangles_cover_frequency_range() {
        let config = MelFilterbankConfig {
            mel_bin_count: 10,
            low_frequency_hz: 0.0,
            high_frequency_hz: 8000.0,
            mel_scale: MelScale::Slaney,
            normalize: MelFilterNorm::None,
        };

        // 512 FFT → 257 bins at 16kHz
        let filterbank = build_filterbank(&config, SampleRate(16_000), 257);
        assert_eq!(filterbank.len(), 10);

        // Each filter should have exactly 257 weights.
        for filter in &filterbank {
            assert_eq!(filter.len(), 257);
        }

        // Each filter should have some non-zero weights (triangles).
        for (i, filter) in filterbank.iter().enumerate() {
            let nonzero_count = filter.iter().filter(|&&w| w > 0.0).count();
            assert!(nonzero_count > 0, "filter {i} has no non-zero weights");
        }

        // Filters should be ordered: later filters have their peak at higher frequencies.
        let peak_bins: Vec<usize> = filterbank
            .iter()
            .map(|filter| {
                filter
                    .iter()
                    .copied()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            })
            .collect();
        for i in 1..peak_bins.len() {
            assert!(
                peak_bins[i] >= peak_bins[i - 1],
                "filter peaks should be non-decreasing: {:?}",
                peak_bins
            );
        }
    }
}
