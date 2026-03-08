//! Spectral DSP helpers for the Signal workspace.

use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::{FrameCount, Sample, SampleRate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StftConfig {
    pub window_size: FrameCount,
    pub hop_size: FrameCount,
}

impl StftConfig {
    pub fn new(window_size: usize, hop_size: usize) -> Self {
        Self {
            window_size: FrameCount(window_size),
            hop_size: FrameCount(hop_size),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumFrame {
    pub magnitudes: Vec<f32>,
    pub phases: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spectrogram {
    pub sample_rate: SampleRate,
    pub config: StftConfig,
    pub frames: Vec<SpectrumFrame>,
}

impl Spectrogram {
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn bins(&self) -> usize {
        self.frames
            .first()
            .map(|frame| frame.magnitudes.len())
            .unwrap_or(0)
    }

    pub fn chroma(&self) -> [f32; 12] {
        let mut chroma = [0.0; 12];
        let window_size = self.config.window_size.0;
        if self.frames.is_empty() || window_size == 0 || self.sample_rate.0 == 0 {
            return chroma;
        }

        for frame in &self.frames {
            for (bin_index, magnitude) in frame.magnitudes.iter().enumerate().skip(1) {
                let frequency = bin_frequency(bin_index, self.sample_rate, window_size);
                if let Some(pitch_class) = frequency_to_pitch_class(frequency) {
                    chroma[pitch_class] += *magnitude;
                }
            }
        }

        normalize_array(&mut chroma);
        chroma
    }
}

#[derive(Clone, Debug)]
pub struct Stft {
    config: StftConfig,
    window: Vec<f32>,
}

impl Stft {
    pub fn new(config: StftConfig) -> Self {
        let window = hann_window(config.window_size.0);
        Self { config, window }
    }

    pub fn config(&self) -> StftConfig {
        self.config
    }

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

            let mut magnitudes = Vec::with_capacity(window_size / 2 + 1);
            let mut phases = Vec::with_capacity(window_size / 2 + 1);
            for bin in buffer.iter().take(window_size / 2 + 1) {
                magnitudes.push(bin.norm());
                phases.push(bin.arg());
            }

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

pub fn hann_window(size: usize) -> Vec<f32> {
    if size == 0 {
        return Vec::new();
    }

    let scale = core::f32::consts::TAU / size as f32;
    (0..size)
        .map(|index| 0.5 - 0.5 * (scale * index as f32).cos())
        .collect()
}

fn bin_frequency(bin_index: usize, sample_rate: SampleRate, window_size: usize) -> f32 {
    bin_index as f32 * sample_rate.0 as f32 / window_size as f32
}

fn frequency_to_pitch_class(frequency: f32) -> Option<usize> {
    if !frequency.is_finite() || frequency <= 0.0 {
        return None;
    }

    let midi = 69.0 + 12.0 * (frequency / 440.0).log2();
    let rounded = midi.round() as i32;
    let pitch_class = rounded.rem_euclid(12) as usize;
    Some(pitch_class)
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
    use super::{hann_window, Stft, StftConfig};
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
}
