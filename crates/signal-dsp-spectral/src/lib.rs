//! Spectral DSP helpers for the Signal workspace.
//!
//! The crate currently provides a forward mono STFT, Hann window generation,
//! spectrogram helpers for chroma and spectral centroid extraction,
//! mel-spectrogram projection, and a uniform-partitioned frequency-domain
//! convolver ([`PartitionedConvolver`]) for reverb-length impulse responses.
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

#![warn(missing_docs)]

pub mod analysis;
mod convolution;
pub mod mel;

pub use convolution::PartitionedConvolver;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::{FrameCount, Sample, SampleRate};
use std::sync::Arc;

/// Configuration for a forward short-time Fourier transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StftConfig {
    /// Number of samples per FFT window.
    pub window_size: FrameCount,
    /// Number of samples to advance between consecutive windows.
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
    /// FFT magnitude per positive-frequency bin (`window_size / 2 + 1` bins).
    pub magnitudes: Vec<f32>,
    /// FFT phase per positive-frequency bin in radians. Empty when
    /// [`StftConfig::compute_phases`] is `false`.
    pub phases: Vec<f32>,
}

/// A mono spectrogram produced by [`Stft::analyze_mono`].
#[derive(Clone, Debug, PartialEq)]
pub struct Spectrogram {
    /// Sample rate of the source audio.
    pub sample_rate: SampleRate,
    /// STFT configuration used to produce this spectrogram.
    pub config: StftConfig,
    /// Ordered sequence of spectrum frames, one per hop position.
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
        analysis::compute_chroma(
            &self.frames,
            self.sample_rate,
            self.config.window_size.0,
            reference_hz,
        )
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
        analysis::compute_spectral_centroid(
            &self.frames,
            self.sample_rate,
            self.config.window_size.0,
        )
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

    /// Create a stateful chunked STFT analyzer with the same configuration.
    pub fn streaming(&self) -> StreamingStft {
        StreamingStft::new(self.config)
    }

    /// Analyze a mono sample slice and produce a spectrogram.
    pub fn analyze_mono(&self, sample_rate: SampleRate, samples: &[Sample]) -> Spectrogram {
        if self.config.window_size.0 == 0 || samples.is_empty() {
            return Spectrogram {
                sample_rate,
                config: self.config,
                frames: Vec::new(),
            };
        }

        let mut streaming = StreamingStft::with_window(self.config, self.window.clone());
        let mut frames = streaming.process_chunk(samples);
        frames.extend(streaming.finish());

        Spectrogram {
            sample_rate,
            config: self.config,
            frames,
        }
    }
}

/// Stateful chunked STFT analyzer that matches [`Stft::analyze_mono`] framing.
pub struct StreamingStft {
    config: StftConfig,
    window: Vec<f32>,
    fft: Arc<dyn Fft<f32>>,
    pending: Vec<Sample>,
    frames_emitted: usize,
}

impl StreamingStft {
    /// Create a chunked STFT analyzer with a precomputed Hann window.
    pub fn new(config: StftConfig) -> Self {
        Self::with_window(config, hann_window(config.window_size.0))
    }

    fn with_window(config: StftConfig, window: Vec<f32>) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(config.window_size.0.max(1));

        Self {
            config,
            window,
            fft,
            pending: Vec::new(),
            frames_emitted: 0,
        }
    }

    /// Return the current STFT configuration.
    pub fn config(&self) -> StftConfig {
        self.config
    }

    /// Reset internal state, discarding any buffered samples and frame count.
    pub fn reset(&mut self) {
        self.pending.clear();
        self.frames_emitted = 0;
    }

    /// Feed one chunk of mono audio and return every newly available full frame.
    pub fn process_chunk(&mut self, samples: &[Sample]) -> Vec<SpectrumFrame> {
        let window_size = self.config.window_size.0;
        let hop_size = self.config.hop_size.0.max(1);
        if window_size == 0 || samples.is_empty() {
            return Vec::new();
        }

        self.pending.extend_from_slice(samples);

        let mut frames = Vec::new();
        while self.pending.len() >= window_size {
            frames.push(compute_spectrum_frame(
                self.config,
                &self.window,
                &self.fft,
                &self.pending[..window_size],
            ));
            self.frames_emitted += 1;

            let drain_count = hop_size.min(self.pending.len());
            self.pending.drain(..drain_count);
        }

        frames
    }

    /// Flush a final zero-padded frame when the remaining samples still imply
    /// another hop position that has not been emitted yet.
    pub fn finish(&mut self) -> Vec<SpectrumFrame> {
        let window_size = self.config.window_size.0;
        let hop_size = self.config.hop_size.0.max(1);
        if window_size == 0 || self.pending.is_empty() {
            self.reset();
            return Vec::new();
        }

        let overlap_tail = window_size.saturating_sub(hop_size);
        let should_emit_partial = if self.frames_emitted == 0 {
            true
        } else {
            self.pending.len() > overlap_tail
        };

        let frames = if should_emit_partial {
            vec![compute_spectrum_frame(
                self.config,
                &self.window,
                &self.fft,
                &self.pending,
            )]
        } else {
            Vec::new()
        };

        self.reset();
        frames
    }
}

fn compute_spectrum_frame(
    config: StftConfig,
    window: &[f32],
    fft: &Arc<dyn Fft<f32>>,
    samples: &[Sample],
) -> SpectrumFrame {
    let window_size = config.window_size.0;
    let mut buffer = vec![Complex32::new(0.0, 0.0); window_size];
    for (index, slot) in buffer.iter_mut().enumerate() {
        let sample = samples.get(index).copied().unwrap_or(0.0);
        *slot = Complex32::new(sample * window[index], 0.0);
    }

    fft.process(&mut buffer);

    let positive_bins = window_size / 2 + 1;
    let mut magnitudes = Vec::with_capacity(positive_bins);
    let phases = if config.compute_phases {
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

    SpectrumFrame { magnitudes, phases }
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

// Re-export key mel types for convenience.
pub use mel::{
    apply_compression, build_filterbank, hz_to_mel, mel_to_hz, LogCompression, MelFilterNorm,
    MelFilterbankConfig, MelScale, MelSpectrogram, MelSpectrogramConfig,
};

#[cfg(test)]
mod stft_tests;
