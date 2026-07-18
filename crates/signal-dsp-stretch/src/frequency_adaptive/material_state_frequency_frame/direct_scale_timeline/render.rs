use super::{geometry::*, *};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RenderMetrics {
    pub imaginary_residue: f64,
    pub conjugate_error: f64,
    pub non_finite_values: usize,
    pub work: WorkCounts,
}

impl Prepared {
    pub(super) fn render_scale(
        &mut self,
        source: &[f64],
        destination: &mut [f64],
        channel: usize,
        scale: Scale,
        masked: bool,
    ) -> RenderMetrics {
        assert_eq!(source.len(), destination.len());
        assert!(channel < self.channels);
        destination.fill(0.0);
        let index = scale.index();
        let prefix = self.lengths[..index].iter().sum::<usize>();
        let frame_offset = channel * self.lengths.iter().sum::<usize>() + prefix;
        let ring_length = self.lengths[0];
        let ring_offset = channel * ring_length;
        let plan = &self.plans[index];
        let buffer = &mut self.transform[frame_offset..frame_offset + plan.length];
        let ring = &mut self.output_ring[ring_offset..ring_offset + ring_length];
        let scratch = &mut self.scratch[..self.planner_scratch];
        ring.fill(0.0);

        let mut metrics = RenderMetrics::default();
        let Some((first, last)) = scale_tick_range(source.len(), plan.length, self.hop) else {
            return metrics;
        };
        let mut emitted = 0_usize;
        let half = plan.length as isize / 2;
        for tick in first..=last {
            let center = tick * self.hop as isize;
            for (offset, (slot, window)) in buffer.iter_mut().zip(&plan.window).enumerate() {
                let logical = center - half + offset as isize;
                *slot = Complex64::new(reflected(source, logical) * window, 0.0);
            }
            plan.forward.process_with_scratch(buffer, scratch);
            metrics.work.forward_transforms += 1;
            metrics.work.window_visits += plan.length;
            if masked {
                apply_mask(buffer, self.sample_rate, plan.scale);
            }
            metrics.conjugate_error = metrics.conjugate_error.max(conjugate_error(buffer));
            metrics.work.conjugate_visits += plan.length / 2 + 1;
            plan.inverse.process_with_scratch(buffer, scratch);
            metrics.work.inverse_transforms += 1;
            metrics.work.coefficient_visits += plan.length;
            let inverse = 1.0 / plan.length as f64;
            for (offset, (value, window)) in buffer.iter().zip(&plan.window).enumerate() {
                let logical = center - half + offset as isize;
                metrics.imaginary_residue = metrics
                    .imaginary_residue
                    .max((value.im * inverse * window).abs());
                metrics.non_finite_values +=
                    usize::from(!value.re.is_finite()) + usize::from(!value.im.is_finite());
                if (0..source.len() as isize).contains(&logical) {
                    ring[logical as usize % ring_length] += value.re * inverse * window;
                }
            }
            metrics.work.window_visits += plan.length;

            let safe_end =
                (center + self.hop as isize - half).clamp(0, source.len() as isize) as usize;
            while emitted < safe_end {
                destination[emitted] = ring[emitted % ring_length];
                ring[emitted % ring_length] = 0.0;
                emitted += 1;
            }
        }
        while emitted < source.len() {
            destination[emitted] = ring[emitted % ring_length];
            ring[emitted % ring_length] = 0.0;
            emitted += 1;
        }
        metrics.non_finite_values += destination
            .iter()
            .filter(|sample| !sample.is_finite())
            .count();
        metrics
    }
}

pub(super) fn unity_bypass(source: &[f64], destination: &mut [f64]) {
    assert_eq!(source.len(), destination.len());
    destination.copy_from_slice(source);
}

pub(super) fn partition_error(window: &[f64], hop: usize) -> f64 {
    let overlaps = window.len() / hop;
    (0..hop)
        .map(|index| {
            let sum = (0..overlaps)
                .map(|overlap| window[index + overlap * hop].powi(2))
                .sum::<f64>();
            (sum - 1.0).abs()
        })
        .fold(0.0_f64, f64::max)
}

fn scale_tick_range(target: usize, length: usize, hop: usize) -> Option<(isize, isize)> {
    if target == 0 {
        return None;
    }
    let half_hops = length / (2 * hop);
    let first = -(half_hops as isize) + 1;
    let last = ((target + length / 2 - 1) / hop) as isize;
    Some((first, last))
}

fn apply_mask(spectrum: &mut [Complex64], sample_rate: usize, scale: Scale) {
    let length = spectrum.len();
    let nyquist = sample_rate as f64 * 0.5;
    for (bin, value) in spectrum.iter_mut().enumerate() {
        let absolute = bin.min(length - bin);
        let frequency = absolute as f64 * sample_rate as f64 / length as f64;
        if !owns_frequency(scale, frequency, nyquist) {
            *value = Complex64::default();
        }
    }
}

fn conjugate_error(spectrum: &[Complex64]) -> f64 {
    (0..spectrum.len())
        .map(|bin| {
            let mirror = if bin == 0 { 0 } else { spectrum.len() - bin };
            (spectrum[bin] - spectrum[mirror].conj()).norm()
        })
        .fold(0.0_f64, f64::max)
}
