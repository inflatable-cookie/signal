use rustfft::num_complex::Complex64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) struct SynthesisRelationTrace {
    pub(in super::super) inverse_ipd_error: [f64; 2],
    pub(in super::super) accumulated_ipd_error: [f64; 2],
    pub(in super::super) normalized_ipd_error: [f64; 2],
}

#[derive(Clone, Copy)]
pub(super) struct SynthesisTraceSpec {
    pub(super) frequency: f64,
    pub(super) inverse_expected_ipd: f64,
    pub(super) output_expected_ipd: [f64; 2],
    pub(super) sample_rate: usize,
    pub(super) interior_trim: usize,
}

pub(super) struct SynthesisTraceState {
    spec: SynthesisTraceSpec,
    frames: [Vec<f64>; 2],
    inverse_ipd_error: [f64; 2],
    accumulated_ipd_error: [f64; 2],
    normalized_ipd_error: [f64; 2],
}

impl SynthesisTraceState {
    pub(super) fn new(spec: SynthesisTraceSpec, frame_length: usize) -> Self {
        Self {
            spec,
            frames: std::array::from_fn(|_| vec![0.0; frame_length]),
            inverse_ipd_error: [0.0; 2],
            accumulated_ipd_error: [0.0; 2],
            normalized_ipd_error: [0.0; 2],
        }
    }

    pub(super) fn record_frame_channel(&mut self, channel: usize, frame: &[f64]) {
        self.frames[channel].copy_from_slice(frame);
    }

    pub(super) fn complete_frame(
        &mut self,
        output_center: isize,
        target_length: usize,
        frame_length: usize,
    ) {
        let error = expected_ipd_error(
            [&self.frames[0], &self.frames[1]],
            self.spec.frequency,
            self.spec.inverse_expected_ipd,
            self.spec.sample_rate,
        );
        self.inverse_ipd_error[0] = self.inverse_ipd_error[0].max(error);
        let start = output_center - frame_length as isize / 2;
        let end = start + frame_length as isize;
        let interior_end = target_length.saturating_sub(self.spec.interior_trim) as isize;
        if start >= self.spec.interior_trim as isize && end <= interior_end {
            self.inverse_ipd_error[1] = self.inverse_ipd_error[1].max(error);
        }
    }

    pub(super) fn record_accumulated(&mut self, output: &[Vec<f64>; 2]) {
        self.accumulated_ipd_error = self.output_errors(output);
    }

    pub(super) fn record_normalized(&mut self, output: &[Vec<f64>; 2]) {
        self.normalized_ipd_error = self.output_errors(output);
    }

    pub(super) fn finish(self) -> SynthesisRelationTrace {
        SynthesisRelationTrace {
            inverse_ipd_error: self.inverse_ipd_error,
            accumulated_ipd_error: self.accumulated_ipd_error,
            normalized_ipd_error: self.normalized_ipd_error,
        }
    }

    fn output_errors(&self, output: &[Vec<f64>; 2]) -> [f64; 2] {
        let whole = expected_ipd_error(
            [&output[0], &output[1]],
            self.spec.frequency,
            self.spec.output_expected_ipd[0],
            self.spec.sample_rate,
        );
        let end = output[0].len().saturating_sub(self.spec.interior_trim);
        let start = self.spec.interior_trim.min(end);
        let interior = expected_ipd_error(
            [&output[0][start..end], &output[1][start..end]],
            self.spec.frequency,
            self.spec.output_expected_ipd[1],
            self.spec.sample_rate,
        );
        [whole, interior]
    }
}

fn expected_ipd_error(
    channels: [&[f64]; 2],
    frequency: f64,
    expected_ipd: f64,
    sample_rate: usize,
) -> f64 {
    let phases = channels.map(|samples| projection(samples, frequency, sample_rate).arg());
    wrap(wrap(phases[1] - phases[0]) - expected_ipd).abs()
}

fn projection(samples: &[f64], frequency: f64, sample_rate: usize) -> Complex64 {
    samples
        .iter()
        .enumerate()
        .fold(Complex64::new(0.0, 0.0), |sum, (index, sample)| {
            let phase = -std::f64::consts::TAU * frequency * index as f64 / sample_rate as f64;
            sum + Complex64::from_polar(*sample, phase)
        })
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
