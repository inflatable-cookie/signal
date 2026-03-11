//! Spectral DSP helpers for the Signal workspace.
//!
//! The crate currently provides a forward mono STFT, Hann window generation,
//! spectrogram helpers for chroma and spectral centroid extraction, and
//! mel-spectrogram projection.
//!
//! ```no_run
//! use signal_dsp_spectral::{Stft, StftConfig};
//! use signal_primitives::SampleRate;
//!
//! let stft = Stft::new(StftConfig::new(1024, 256));
//! let samples = vec![0.0f32; 4_096];
//! let spectrogram = stft.analyze_mono(SampleRate(48_000), &samples);
//!
//! assert_eq!(spectrogram.config.window_size.0, 1024);
//! assert_eq!(spectrogram.bins(), 513);
//! ```

use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::{FrameCount, Sample, SampleRate};

/// Configuration for a forward short-time Fourier transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StftConfig {
    pub window_size: FrameCount,
    pub hop_size: FrameCount,
    /// When `false`, phase values are omitted from each [`SpectrumFrame`],
    /// saving the per-bin `atan2` cost.  Defaults to `true`.
    pub compute_phases: bool,
}

impl StftConfig {
    /// Build an STFT config from raw frame counts.
    pub fn new(window_size: usize, hop_size: usize) -> Self {
        Self {
            window_size: FrameCount(window_size),
            hop_size: FrameCount(hop_size),
            compute_phases: true,
        }
    }
}

/// A single positive-frequency spectrum frame.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumFrame {
    pub magnitudes: Vec<f32>,
    pub phases: Vec<f32>,
}

/// A mono spectrogram produced by [`Stft::analyze_mono`].
#[derive(Clone, Debug, PartialEq)]
pub struct Spectrogram {
    pub sample_rate: SampleRate,
    pub config: StftConfig,
    pub frames: Vec<SpectrumFrame>,
}

impl Spectrogram {
    /// Return `true` when no spectrum frames were produced.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Return the number of retained positive-frequency bins per frame.
    pub fn bins(&self) -> usize {
        self.frames
            .first()
            .map(|frame| frame.magnitudes.len())
            .unwrap_or(0)
    }

    /// Accumulate a normalized 12-bin pitch-class profile across all frames,
    /// using A=440 Hz as the tuning reference.
    ///
    /// Each bin's magnitude is weighted by `1 / frequency` before
    /// accumulation.  In a linear FFT, higher octaves contain proportionally
    /// more bins per pitch class than lower ones, so unweighted sums let
    /// high-frequency content (and harmonic overtones) dominate the profile.
    /// The `1/f` weight corrects for this by giving each octave roughly equal
    /// influence and naturally attenuating the harmonic series bleed that
    /// otherwise inflates the perfect-fifth pitch class above every strong
    /// fundamental.
    pub fn chroma(&self) -> [f32; 12] {
        self.chroma_with_reference(440.0)
    }

    /// Like [`chroma`](Self::chroma), but with an explicit tuning reference
    /// in Hz.  Passing a value other than 440.0 shifts the pitch-class
    /// boundaries so that tracks produced at non-standard tunings (e.g.
    /// A=432 Hz) map to the correct semitones.
    pub fn chroma_with_reference(&self, reference_hz: f32) -> [f32; 12] {
        let mut chroma = [0.0; 12];
        let window_size = self.config.window_size.0;
        if self.frames.is_empty() || window_size == 0 || self.sample_rate.0 == 0 {
            return chroma;
        }

        // Compute the minimum frequency at which FFT bins are narrow enough
        // for reliable pitch-class assignment.  Below this threshold the bin
        // spacing exceeds one semitone, so a single note's energy can land
        // in the wrong pitch class entirely (e.g. B1 at 61.7 Hz splits
        // between A# and C bins with no bin mapping to B).  Combined with
        // 1/f weighting these mis-mapped sub-bass bins would dominate the
        // chroma and shift the detected key by a semitone.
        //
        // The factor 0.05777 ≈ 2^(1/24) − 2^(−1/24) is the fractional width
        // of one semitone.
        let bin_spacing = self.sample_rate.0 as f32 / window_size as f32;
        let min_frequency = bin_spacing / SEMITONE_WIDTH_RATIO;

        for frame in &self.frames {
            for (bin_index, magnitude) in frame.magnitudes.iter().enumerate().skip(1) {
                let frequency = bin_frequency(bin_index, self.sample_rate, window_size);
                if frequency < min_frequency || frequency > 5_000.0 {
                    continue;
                }
                let pitch_class = frequency_to_pitch_class_with_reference(frequency, reference_hz);
                chroma[pitch_class] += *magnitude / frequency;
            }
        }

        normalize_array(&mut chroma);
        chroma
    }

    /// Compute the median spectral centroid across all frames, in Hz.
    ///
    /// The spectral centroid is the magnitude-weighted mean of FFT bin
    /// frequencies — a single number capturing the "centre of mass" of the
    /// spectrum.  It is a standard timbral brightness indicator: low values
    /// (~500 Hz) correspond to dark/warm timbres, high values (~3000+ Hz) to
    /// bright/crisp ones.
    ///
    /// The per-frame centroids are reduced to a single value via the median,
    /// which is robust against transient outliers (e.g. a brief cymbal crash
    /// in an otherwise warm track).
    ///
    /// Returns `0.0` for an empty spectrogram or when all magnitudes are zero.
    pub fn spectral_centroid(&self) -> f32 {
        if self.frames.is_empty() || self.config.window_size.0 == 0 || self.sample_rate.0 == 0 {
            return 0.0;
        }

        let window_size = self.config.window_size.0;
        let mut frame_centroids = Vec::with_capacity(self.frames.len());

        for frame in &self.frames {
            let mut weighted_sum = 0.0f32;
            let mut magnitude_sum = 0.0f32;

            for (bin_index, &magnitude) in frame.magnitudes.iter().enumerate() {
                let frequency = bin_frequency(bin_index, self.sample_rate, window_size);
                weighted_sum += frequency * magnitude;
                magnitude_sum += magnitude;
            }

            if magnitude_sum > 0.0 {
                frame_centroids.push(weighted_sum / magnitude_sum);
            }
        }

        if frame_centroids.is_empty() {
            return 0.0;
        }

        // Median for robustness against transient outliers.
        frame_centroids.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        frame_centroids[frame_centroids.len() / 2]
    }
}

/// Forward STFT analyzer for mono audio.
#[derive(Clone, Debug)]
pub struct Stft {
    config: StftConfig,
    window: Vec<f32>,
}

impl Stft {
    /// Create an STFT analyzer with a precomputed Hann window.
    pub fn new(config: StftConfig) -> Self {
        let window = hann_window(config.window_size.0);
        Self { config, window }
    }

    /// Return the current transform configuration.
    pub fn config(&self) -> StftConfig {
        self.config
    }

    /// Analyze a mono sample slice and produce a spectrogram.
    pub fn analyze_mono(&self, sample_rate: SampleRate, samples: &[Sample]) -> Spectrogram {
        let window_size = self.config.window_size.0;
        let hop_size = self.config.hop_size.0.max(1);

        if window_size == 0 || samples.is_empty() {
            return Spectrogram {
                sample_rate,
                config: self.config,
                frames: Vec::new(),
            };
        }

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(window_size);
        let mut frames = Vec::new();
        let mut start = 0usize;

        loop {
            let mut buffer = vec![Complex32::new(0.0, 0.0); window_size];
            for (index, slot) in buffer.iter_mut().enumerate() {
                let sample = samples.get(start + index).copied().unwrap_or(0.0);
                *slot = Complex32::new(sample * self.window[index], 0.0);
            }

            fft.process(&mut buffer);

            let positive_bins = window_size / 2 + 1;
            let mut magnitudes = Vec::with_capacity(positive_bins);
            let phases = if self.config.compute_phases {
                let mut phases = Vec::with_capacity(positive_bins);
                for bin in buffer.iter().take(positive_bins) {
                    magnitudes.push(bin.norm());
                    phases.push(bin.arg());
                }
                phases
            } else {
                for bin in buffer.iter().take(positive_bins) {
                    magnitudes.push(bin.norm());
                }
                Vec::new()
            };

            frames.push(SpectrumFrame { magnitudes, phases });

            if start + window_size >= samples.len() {
                break;
            }

            start = start.saturating_add(hop_size);
        }

        Spectrogram {
            sample_rate,
            config: self.config,
            frames,
        }
    }
}

/// Generate a Hann window of `size` samples.
pub fn hann_window(size: usize) -> Vec<f32> {
    if size == 0 {
        return Vec::new();
    }

    let scale = core::f32::consts::TAU / size as f32;
    (0..size)
        .map(|index| 0.5 - 0.5 * (scale * index as f32).cos())
        .collect()
}

/// Compute the centre frequency of an FFT bin.
pub fn bin_frequency(bin_index: usize, sample_rate: SampleRate, window_size: usize) -> f32 {
    bin_index as f32 * sample_rate.0 as f32 / window_size as f32
}

// ---------------------------------------------------------------------------
// Mel-spectrogram support
// ---------------------------------------------------------------------------

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

// ---- Mel scale conversions ----

fn hz_to_mel(hz: f32, scale: MelScale) -> f32 {
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

fn mel_to_hz(mel: f32, scale: MelScale) -> f32 {
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
fn build_filterbank(
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

fn apply_compression(value: f32, compression: LogCompression) -> f32 {
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

/// Fractional width of one semitone: 2^(1/24) − 2^(−1/24).
///
/// Used to compute the minimum frequency at which one FFT bin fits within
/// a single semitone: `min_freq = bin_spacing / SEMITONE_WIDTH_RATIO`.
const SEMITONE_WIDTH_RATIO: f32 = 0.057_77;

/// Map a frequency (already range-checked by the caller) to its nearest
/// pitch class (0 = C … 11 = B).
fn frequency_to_pitch_class_with_reference(frequency: f32, reference_hz: f32) -> usize {
    let midi = 69.0 + 12.0 * (frequency / reference_hz).log2();
    let rounded = midi.round() as i32;
    rounded.rem_euclid(12) as usize
}

fn normalize_array(values: &mut [f32; 12]) {
    let sum = values.iter().copied().sum::<f32>();
    if sum > 0.0 {
        for value in values {
            *value /= sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let spectrogram = super::Spectrogram {
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

    // --- Mel-spectrogram tests ---

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
