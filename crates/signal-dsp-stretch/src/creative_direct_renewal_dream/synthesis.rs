use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};

use super::{
    analysis::{periodic_hann, Analyzer},
    plan::RenderPlan,
    stereo::{renew_mono, renew_stereo},
    CandidateError, CandidateRequest,
};

struct Renderer {
    forward: Arc<dyn Fft<f32>>,
    inverse: Arc<dyn Fft<f32>>,
    analyzer: Analyzer,
    inverse_scratch: Vec<Complex32>,
    left_spectrum: Vec<Complex32>,
    right_spectrum: Vec<Complex32>,
    first_frame: Vec<f32>,
    second_frame: Vec<f32>,
    gain: f64,
}

impl Renderer {
    fn new(plan: &RenderPlan) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(plan.fft_size);
        let inverse = planner.plan_fft_inverse(plan.fft_size);
        let analyzer = Analyzer::new(plan.fft_size, forward.as_ref());
        let inverse_scratch = vec![Complex32::new(0.0, 0.0); inverse.get_inplace_scratch_len()];
        let left_spectrum = vec![Complex32::new(0.0, 0.0); plan.fft_size];
        let right_spectrum = vec![Complex32::new(0.0, 0.0); plan.fft_size];
        let frame_samples = plan.fft_size * plan.channels;
        let first_frame = vec![0.0; frame_samples];
        let second_frame = vec![0.0; frame_samples];
        let (_, gain) = periodic_hann(plan.fft_size);
        Self {
            forward,
            inverse,
            analyzer,
            inverse_scratch,
            left_spectrum,
            right_spectrum,
            first_frame,
            second_frame,
            gain,
        }
    }

    fn compute_frame(
        &mut self,
        request: &CandidateRequest<'_>,
        plan: &RenderPlan,
        frame_index: usize,
        first: bool,
    ) -> Result<(), CandidateError> {
        self.analyzer.analyze(
            request,
            plan,
            frame_index,
            0,
            self.forward.as_ref(),
            &mut self.left_spectrum,
        )?;
        if plan.channels == 2 {
            self.analyzer.analyze(
                request,
                plan,
                frame_index,
                1,
                self.forward.as_ref(),
                &mut self.right_spectrum,
            )?;
            renew_stereo(
                &mut self.left_spectrum,
                &mut self.right_spectrum,
                frame_index,
                request.seed,
                request.space,
                plan.sample_rate,
            );
        } else {
            renew_mono(&mut self.left_spectrum, frame_index, request.seed);
        }

        self.inverse
            .process_with_scratch(&mut self.left_spectrum, &mut self.inverse_scratch);
        let destination = if first {
            &mut self.first_frame
        } else {
            &mut self.second_frame
        };
        let inverse_scale = 1.0 / plan.fft_size as f32;
        for index in 0..plan.fft_size {
            destination[index * plan.channels] = self.left_spectrum[index].re * inverse_scale;
        }

        if plan.channels == 2 {
            self.inverse
                .process_with_scratch(&mut self.right_spectrum, &mut self.inverse_scratch);
            for index in 0..plan.fft_size {
                destination[index * 2 + 1] = self.right_spectrum[index].re * inverse_scale;
            }
        }
        Ok(())
    }

    fn planned_bytes(&self) -> usize {
        self.analyzer.planned_bytes()
            + self.inverse_scratch.len() * std::mem::size_of::<Complex32>()
            + self.left_spectrum.len() * std::mem::size_of::<Complex32>()
            + self.right_spectrum.len() * std::mem::size_of::<Complex32>()
            + self.first_frame.len() * std::mem::size_of::<f32>()
            + self.second_frame.len() * std::mem::size_of::<f32>()
    }
}

#[cfg(test)]
pub(crate) fn planned_working_bytes(plan: &RenderPlan) -> usize {
    Renderer::new(plan).planned_bytes()
}

pub(super) fn render(
    request: &CandidateRequest<'_>,
    plan: &RenderPlan,
) -> Result<Vec<f32>, CandidateError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(plan.output_samples)
        .map_err(|_| CandidateError::AllocationFailed)?;
    output.resize(plan.output_samples, 0.0);

    let mut renderer = Renderer::new(plan);
    if renderer.planned_bytes() > 32 * 1024 * 1024 {
        return Err(CandidateError::SizeOverflow);
    }
    #[cfg(test)]
    crate::direct_renewal_dream_processing_started();
    renderer.compute_frame(request, plan, 0, true)?;
    renderer.compute_frame(request, plan, 1, false)?;

    for block in 0..plan.blocks {
        let block_start = block * plan.hop;
        let remaining = plan.target_frames - block_start;
        let block_frames = remaining.min(plan.hop);
        for offset in 0..block_frames {
            let u = (offset as f64 + 0.5) / plan.hop as f64;
            let first_weight = 0.5 + 0.5 * (std::f64::consts::PI * u).cos();
            let second_weight = 1.0 - first_weight;
            let compensation =
                1.0 / (first_weight * first_weight + second_weight * second_weight).sqrt();
            let output_frame = block_start + offset;
            let envelope = boundary_envelope(output_frame, plan);
            for channel in 0..plan.channels {
                let first = renderer.first_frame[(plan.hop + offset) * plan.channels + channel];
                let second = renderer.second_frame[offset * plan.channels + channel];
                let sample = renderer.gain
                    * compensation
                    * (first_weight * first as f64 + second_weight * second as f64)
                    * envelope;
                if !sample.is_finite() {
                    return Err(CandidateError::NonFiniteProcessing);
                }
                output[output_frame * plan.channels + channel] = sample as f32;
            }
        }
        if block + 1 < plan.blocks {
            std::mem::swap(&mut renderer.first_frame, &mut renderer.second_frame);
            renderer.compute_frame(request, plan, block + 2, false)?;
        }
    }

    for channel in 0..plan.channels {
        output[channel] = 0.0;
        output[(plan.target_frames - 1) * plan.channels + channel] = 0.0;
    }
    Ok(output)
}

pub(crate) fn boundary_envelope(output_frame: usize, plan: &RenderPlan) -> f64 {
    envelope_factor(output_frame, plan.head_extent)
        * envelope_factor(plan.target_frames - 1 - output_frame, plan.tail_extent)
}

fn envelope_factor(distance: usize, extent: usize) -> f64 {
    match extent {
        0 => 1.0,
        1 => {
            if distance == 0 {
                0.0
            } else {
                1.0
            }
        }
        _ if distance < extent => {
            (std::f64::consts::FRAC_PI_2 * distance as f64 / (extent - 1) as f64).sin()
        }
        _ => 1.0,
    }
}
