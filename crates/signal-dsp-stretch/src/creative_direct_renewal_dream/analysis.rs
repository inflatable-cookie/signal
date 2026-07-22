use rustfft::{num_complex::Complex32, Fft};

use super::{plan::RenderPlan, CandidateError, CandidateRequest};

pub(crate) fn periodic_hann(fft_size: usize) -> (Vec<f64>, f64) {
    let mut window = Vec::with_capacity(fft_size);
    let mut energy = 0.0_f64;
    for index in 0..fft_size {
        let phase = std::f64::consts::TAU * index as f64 / fft_size as f64;
        let value = 0.5 - 0.5 * phase.cos();
        window.push(value);
        energy += value * value;
    }
    let gain = (fft_size as f64 / energy).sqrt();
    (window, gain)
}

pub(crate) fn cubic_coefficients(u: f64) -> [f64; 4] {
    [
        -u * (u - 1.0) * (u - 2.0) / 6.0,
        (u + 1.0) * (u - 1.0) * (u - 2.0) / 2.0,
        -(u + 1.0) * u * (u - 2.0) / 2.0,
        (u + 1.0) * u * (u - 1.0) / 6.0,
    ]
}

pub(crate) fn interpolated_sample(
    input: &[f32],
    channels: usize,
    channel: usize,
    source_frames: usize,
    position: f64,
) -> f64 {
    let integer = position.floor();
    let fraction = position - integer;
    let coefficients = cubic_coefficients(fraction);
    let base = integer as i128;
    let offsets = [-1_i128, 0, 1, 2];
    let mut value = 0.0_f64;
    for (coefficient, offset) in coefficients.into_iter().zip(offsets) {
        let frame = base + offset;
        if frame >= 0 && frame < source_frames as i128 {
            value += coefficient * input[frame as usize * channels + channel] as f64;
        }
    }
    value
}

pub(super) struct Analyzer {
    window: Vec<f64>,
    forward_scratch: Vec<Complex32>,
}

impl Analyzer {
    pub(super) fn new(fft_size: usize, forward: &dyn Fft<f32>) -> Self {
        let (window, _) = periodic_hann(fft_size);
        Self {
            window,
            forward_scratch: vec![Complex32::new(0.0, 0.0); forward.get_inplace_scratch_len()],
        }
    }

    pub(super) fn analyze(
        &mut self,
        request: &CandidateRequest<'_>,
        plan: &RenderPlan,
        frame_index: usize,
        channel: usize,
        forward: &dyn Fft<f32>,
        spectrum: &mut [Complex32],
    ) -> Result<(), CandidateError> {
        let center = plan.source_center(frame_index)?;
        let half_offset = (plan.fft_size - 1) as f64 * 0.5;
        for (index, bin) in spectrum.iter_mut().enumerate() {
            let position = center + index as f64 - half_offset;
            let sample = interpolated_sample(
                request.input,
                plan.channels,
                channel,
                plan.source_frames,
                position,
            );
            *bin = Complex32::new((sample * self.window[index]) as f32, 0.0);
        }
        forward.process_with_scratch(spectrum, &mut self.forward_scratch);
        if spectrum
            .iter()
            .any(|coefficient| !coefficient.re.is_finite() || !coefficient.im.is_finite())
        {
            return Err(CandidateError::NonFiniteProcessing);
        }
        Ok(())
    }

    pub(super) fn planned_bytes(&self) -> usize {
        self.window.len() * std::mem::size_of::<f64>()
            + self.forward_scratch.len() * std::mem::size_of::<Complex32>()
    }
}
