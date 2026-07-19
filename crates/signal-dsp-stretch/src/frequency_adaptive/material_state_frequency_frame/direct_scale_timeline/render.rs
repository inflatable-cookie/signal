use super::{geometry::*, *};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RenderMetrics {
    pub imaginary_residue: f64,
    pub conjugate_error: f64,
    pub non_finite_values: usize,
    pub work: WorkCounts,
}

impl Prepared {
    fn analyse_tick(
        &mut self,
        inputs: [&[f64]; CHANNEL_CAPACITY],
        source_frames: usize,
        target_frames: usize,
        tick: isize,
        retain_coefficients: bool,
    ) {
        let atoms = self.owned_bins.iter().sum::<usize>();
        let coefficients = self.channels * atoms;
        let pending_slot = tick.rem_euclid(PENDING_TICKS as isize) as usize;
        let guidance_slot = tick.rem_euclid(GUIDANCE_TICKS as isize) as usize;
        self.guidance[guidance_slot * atoms..(guidance_slot + 1) * atoms].fill(0.0);
        let center = source_center(tick, self.hop, source_frames, target_frames)
            .expect("supported direct timeline source projection");
        let frame_stride = self.lengths.iter().sum::<usize>();
        let mut atom_first = 0;
        let mut frame_prefix = 0;
        for scale in Scale::ALL {
            let scale_index = scale.index();
            let length = self.lengths[scale_index];
            let owned = self.owned_bins[scale_index];
            let start_bin = owned_start_bin(self.sample_rate, length, scale);
            let half = length as isize / 2;
            for channel in 0..self.channels {
                let frame_offset = channel * frame_stride + frame_prefix;
                let buffer = &mut self.transform[frame_offset..frame_offset + length];
                for (local, (value, window)) in buffer
                    .iter_mut()
                    .zip(&self.plans[scale_index].window)
                    .enumerate()
                {
                    *value = Complex64::new(
                        reflected(inputs[channel], center - half + local as isize) * window,
                        0.0,
                    );
                }
                self.plans[scale_index]
                    .forward
                    .process_with_scratch(buffer, &mut self.scratch[..self.planner_scratch]);
                for local in 0..owned {
                    let bin = start_bin + local;
                    let mut value = buffer[bin];
                    if bin == 0 || bin == length / 2 {
                        value.im = 0.0;
                    }
                    let atom = atom_first + local;
                    let magnitude = value.norm();
                    let guidance_index = guidance_slot * atoms + atom;
                    self.guidance[guidance_index] = self.guidance[guidance_index].max(magnitude);
                    if retain_coefficients {
                        self.pending[pending_slot * coefficients + channel * atoms + atom] = value;
                    }
                }
            }
            atom_first += owned;
            frame_prefix += length;
        }
    }

    fn synthesise_tick(
        &mut self,
        decided: &[Complex64],
        tick: isize,
        target_frames: usize,
        non_finite: &mut usize,
    ) {
        let atoms = self.owned_bins.iter().sum::<usize>();
        let frame_stride = self.lengths.iter().sum::<usize>();
        let ring_length = self.lengths[Scale::Long.index()];
        let center = tick * self.hop as isize;
        let mut atom_first = 0;
        let mut frame_prefix = 0;
        for scale in Scale::ALL {
            let scale_index = scale.index();
            let length = self.lengths[scale_index];
            let owned = self.owned_bins[scale_index];
            let start_bin = owned_start_bin(self.sample_rate, length, scale);
            let half = length as isize / 2;
            for channel in 0..self.channels {
                let frame_offset = channel * frame_stride + frame_prefix;
                let buffer = &mut self.transform[frame_offset..frame_offset + length];
                buffer.fill(Complex64::default());
                for local in 0..owned {
                    let bin = start_bin + local;
                    let mut value = decided[channel * atoms + atom_first + local];
                    if bin == 0 || bin == length / 2 {
                        value.im = 0.0;
                    }
                    buffer[bin] = value;
                    if bin != 0 && bin != length / 2 {
                        buffer[length - bin] = value.conj();
                    }
                }
                self.plans[scale_index]
                    .inverse
                    .process_with_scratch(buffer, &mut self.scratch[..self.planner_scratch]);
                let inverse = 1.0 / length as f64;
                let ring_offset = channel * ring_length;
                for (local, (value, window)) in buffer
                    .iter()
                    .zip(&self.plans[scale_index].window)
                    .enumerate()
                {
                    let logical = center - half + local as isize;
                    let sample = value.re * inverse * window;
                    *non_finite += usize::from(!sample.is_finite() || !value.im.is_finite());
                    if (0..target_frames as isize).contains(&logical) {
                        self.output_ring[ring_offset + logical as usize % ring_length] += sample;
                    }
                }
            }
            atom_first += owned;
            frame_prefix += length;
        }
    }

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

pub(super) fn render(
    inputs: [&[f64]; CHANNEL_CAPACITY],
    ratio: f64,
    sample_rate: usize,
) -> CandidateRender {
    assert_eq!(inputs[0].len(), inputs[1].len(), "linked channel lengths");
    assert!(!inputs[0].is_empty(), "nonempty direct timeline input");
    if ratio == 1.0 {
        return finish_candidate(
            [inputs[0].to_vec(), inputs[1].to_vec()],
            inputs[0].len(),
            0,
            [0; 5],
            StateTickReport::default(),
            0,
            0,
        );
    }
    let source_frames = inputs[0].len();
    let target_frames = (source_frames as f64 * ratio).round() as usize;
    let mut prepared = prepare(sample_rate, CHANNEL_CAPACITY, ratio, false)
        .expect("supported direct objective geometry");
    let (first, last) = synthesis_tick_range(target_frames, prepared.hop)
        .expect("nonempty direct objective target");
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let coefficients = CHANNEL_CAPACITY * atoms;
    let mut current = vec![Complex64::default(); coefficients];
    let mut decided = vec![Complex64::default(); coefficients];
    let mut materials = vec![MaterialGuidance::default(); atoms];
    let mut terminal = vec![TerminalState::Reset; atoms];
    let mut channels = std::array::from_fn(|_| vec![0.0; target_frames]);
    let mut total = StateTickReport::default();
    let mut non_finite = 0;
    let mut emitted = 0;

    for tick in first - 9..=first + 9 {
        prepared.analyse_tick(inputs, source_frames, target_frames, tick, tick >= first);
    }
    for tick in first..=last {
        let slot = tick.rem_euclid(PENDING_TICKS as isize) as usize;
        current.copy_from_slice(&prepared.pending[slot * coefficients..(slot + 1) * coefficients]);
        let transient_center = prepared.guidance_at(tick, &mut materials);
        let center = source_center(tick, prepared.hop, source_frames, target_frames)
            .expect("supported direct source centre");
        let previous = source_center(tick - 1, prepared.hop, source_frames, target_frames)
            .expect("supported prior direct source centre");
        let report = prepared
            .process_state_tick(
                &current,
                &materials,
                StateTickControl {
                    transient_center,
                    ordinary_bypass: false,
                    analysis_advance: (center - previous) as f64,
                },
                &mut decided,
                &mut terminal,
            )
            .expect("valid direct objective state tick");
        for (target, value) in total.states.iter_mut().zip(report.states) {
            *target += value;
        }
        total.borrowed_locked_atoms += report.borrowed_locked_atoms;
        total.local_locked_atoms += report.local_locked_atoms;
        total.trajectory_channel_switches += report.trajectory_channel_switches;
        total.channel_peak_disagreements += report.channel_peak_disagreements;
        total.non_finite_values += report.non_finite_values;
        prepared.synthesise_tick(&decided, tick, target_frames, &mut non_finite);

        let safe_end = (tick * prepared.hop as isize - 3 * prepared.hop as isize)
            .clamp(0, target_frames as isize) as usize;
        let ring_length = prepared.lengths[Scale::Long.index()];
        while emitted < safe_end {
            for channel in 0..CHANNEL_CAPACITY {
                let index = channel * ring_length + emitted % ring_length;
                channels[channel][emitted] = prepared.output_ring[index];
                prepared.output_ring[index] = 0.0;
            }
            emitted += 1;
        }
        if tick < last {
            let next = tick + 10;
            prepared.analyse_tick(inputs, source_frames, target_frames, next, next <= last);
        }
    }
    non_finite += total.non_finite_values;
    let uncovered = target_frames - emitted;
    let maximum_output_samples = prepared.memory.output_samples;
    finish_candidate(
        channels,
        target_frames,
        uncovered,
        total.states,
        total,
        non_finite,
        maximum_output_samples,
    )
}

fn finish_candidate(
    channels: [Vec<f64>; CHANNEL_CAPACITY],
    target_length: usize,
    uncovered: usize,
    states: [usize; 5],
    report: StateTickReport,
    extra_non_finite: usize,
    maximum_output_samples: usize,
) -> CandidateRender {
    let non_finite = extra_non_finite
        + channels
            .iter()
            .flatten()
            .filter(|sample| !sample.is_finite())
            .count();
    let boundary_failures = channels
        .iter()
        .map(|channel| {
            usize::from(channel.first().is_none_or(|value| !value.is_finite()))
                + usize::from(channel.last().is_none_or(|value| !value.is_finite()))
        })
        .sum();
    let mut hash = HASH_OFFSET;
    for sample in channels.iter().flatten() {
        hash_u64(&mut hash, sample.to_bits());
    }
    CandidateRender {
        channels,
        target_length,
        uncovered,
        non_finite,
        boundary_failures,
        states,
        borrowed_locked_channel_atoms: report.borrowed_locked_atoms,
        local_locked_channel_atoms: report.local_locked_atoms,
        trajectory_channel_switches: report.trajectory_channel_switches,
        channel_peak_disagreements: report.channel_peak_disagreements,
        maximum_pending_ticks: PENDING_TICKS,
        maximum_guidance_ticks: GUIDANCE_TICKS,
        maximum_output_samples,
        hash,
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
