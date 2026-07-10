use std::sync::Arc;

use rustfft::{num_complex::Complex32, Fft, FftPlanner};
use signal_primitives::Sample;

mod adaptive_timeline;
mod fixed_map_peak;

use adaptive_timeline::{build_adaptive_timeline_schedule, AdaptiveTimelineSchedule};
use fixed_map_peak::{FixedMapPeakEvidence, FixedMapPeakState};

pub(crate) use adaptive_timeline::AdaptiveTimelineEngineRender;

pub(crate) struct FixedMapPeakEngineRender {
    pub(crate) samples: Vec<Sample>,
    pub(crate) evidence: FixedMapPeakEvidence,
    pub(crate) uncovered_output_frames: usize,
}

/// Run the draft phase-vocoder backend.
pub(crate) fn phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IndependentBins,
    )
}

/// Run the identity phase-locked phase-vocoder prototype.
pub(crate) fn phase_locked_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IdentityLocked,
    )
}

/// Run a report-only phase-locked prototype that applies identity locking only
/// on spectrally stable frames.
pub(crate) fn stability_adaptive_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IdentityLockedStabilityAdaptive,
    )
}

/// Run a report-only phase-locked prototype that narrows peak-lock regions
/// for peaks not tracked from the previous analysis frame.
pub(crate) fn tracked_peak_region_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IdentityLockedTrackedPeakRegions,
    )
}

/// Run a report-only phase-locked prototype that limits per-bin magnitude
/// changes on spectrally stable frames.
pub(crate) fn magnitude_slew_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IdentityLockedMagnitudeSlew,
    )
}

/// Run the identity phase-locked prototype with transient phase resets.
pub(crate) fn transient_reset_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let mode = if ratio < 1.0 {
        PhasePropagationMode::IdentityLocked
    } else {
        PhasePropagationMode::IdentityLockedTransientReset
    };
    run_phase_vocoder(input, target_len, ratio, window_size, analysis_hop, mode)
}

/// Run a report-only compression transient-anchor candidate.
pub(crate) fn compression_transient_anchor_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let mode = if ratio < 1.0 {
        PhasePropagationMode::IdentityLockedCompressionTransientAnchor
    } else {
        PhasePropagationMode::IdentityLockedTransientReset
    };
    run_phase_vocoder(input, target_len, ratio, window_size, analysis_hop, mode)
}

/// Run the report-only current-grid adaptive transient-timeline candidate.
pub(crate) fn adaptive_transient_timeline_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    onset_frames: &[usize],
) -> AdaptiveTimelineEngineRender {
    let prefix_frames = window_size / 2;
    let suffix_frames = window_size + analysis_hop;
    let mut padded_input = vec![0.0; prefix_frames + input.len() + suffix_frames];
    padded_input[prefix_frames..prefix_frames + input.len()].copy_from_slice(input);

    let half_window = window_size as f64 * 0.5;
    let output_start =
        ((prefix_frames as f64 - half_window) * ratio + half_window).round() as usize;
    let output_end = output_start + target_len;
    let config =
        PhaseVocoderConfig::new(&padded_input, output_end, ratio, window_size, analysis_hop);
    let schedule = build_adaptive_timeline_schedule(
        config.frame_count,
        ratio,
        window_size,
        analysis_hop,
        onset_frames,
    );
    let mut engine = DraftPhaseVocoder::new_with_schedule(
        config,
        PhasePropagationMode::AdaptiveTransientTimeline,
        schedule.clone(),
    );
    engine.process(&padded_input);
    let uncovered_output_frames = engine.uncovered_frames(output_start, output_end);
    let output = engine.finish();

    AdaptiveTimelineEngineRender {
        samples: output[output_start..output_end].to_vec(),
        synthesis_positions: schedule.positions,
        reinitialized_frames: schedule
            .reinitialize_phase
            .iter()
            .enumerate()
            .filter_map(|(index, reinitialize)| reinitialize.then_some(index))
            .collect(),
        protected_onset_count: schedule.protected_onset_count,
        dense_conflict_count: schedule.dense_conflict_count,
        max_anchor_error_frames: schedule.max_anchor_error_frames,
        min_synthesis_hop_frames: schedule.min_synthesis_hop_frames,
        max_synthesis_hop_frames: schedule.max_synthesis_hop_frames,
        uncovered_output_frames,
        schedule_fallback: schedule.schedule_fallback,
    }
}

/// Run the report-only fixed-map peak-selective transient candidate.
pub(crate) fn fixed_map_peak_transient_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    onset_frames: &[usize],
) -> FixedMapPeakEngineRender {
    let prefix_frames = window_size / 2;
    let suffix_frames = window_size + analysis_hop;
    let mut padded_input = vec![0.0; prefix_frames + input.len() + suffix_frames];
    padded_input[prefix_frames..prefix_frames + input.len()].copy_from_slice(input);

    let half_window = window_size as f64 * 0.5;
    let output_start =
        ((prefix_frames as f64 - half_window) * ratio + half_window).round() as usize;
    let output_end = output_start + target_len;
    let config =
        PhaseVocoderConfig::new(&padded_input, output_end, ratio, window_size, analysis_hop);
    let mut engine = DraftPhaseVocoder::new_with_fixed_map_peak(config, onset_frames);
    engine.process(&padded_input);
    let uncovered_output_frames = engine.uncovered_frames(output_start, output_end);
    let evidence = engine
        .peak_transient
        .as_ref()
        .expect("fixed-map peak state")
        .evidence();
    let output = engine.finish();

    FixedMapPeakEngineRender {
        samples: output[output_start..output_end].to_vec(),
        evidence,
        uncovered_output_frames,
    }
}

/// Run the OfflineHighQuality prototype over interleaved stereo with a linked
/// mid/side analysis surface instead of independent left/right stretching.
pub(crate) fn transient_reset_phase_vocoder_linked_stereo(
    input_interleaved: &[Sample],
    target_frames: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let frame_count = input_interleaved.len() / 2;
    if frame_count == 0 || target_frames == 0 {
        return Vec::new();
    }

    let mut mid = Vec::with_capacity(frame_count);
    let mut side = Vec::with_capacity(frame_count);
    for frame in input_interleaved.chunks_exact(2) {
        let left = frame[0];
        let right = frame[1];
        mid.push((left + right) * 0.5);
        side.push((left - right) * 0.5);
    }

    let stretched_mid =
        transient_reset_phase_vocoder(&mid, target_frames, ratio, window_size, analysis_hop);
    let stretched_side =
        transient_reset_phase_vocoder(&side, target_frames, ratio, window_size, analysis_hop);

    let out_frames = stretched_mid
        .len()
        .min(stretched_side.len())
        .min(target_frames);
    let mut output = Vec::with_capacity(target_frames * 2);
    for index in 0..out_frames {
        let mid = stretched_mid[index];
        let side = stretched_side[index];
        output.push(mid + side);
        output.push(mid - side);
    }
    output.resize(target_frames * 2, 0.0);
    output
}

fn run_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    mode: PhasePropagationMode,
) -> Vec<Sample> {
    // Give the first and last source samples complete overlapping analysis
    // windows. The extra post-roll hop guarantees that the cropped target
    // remains inside synthesized coverage for both compression and expansion.
    let prefix_frames = window_size / 2;
    let suffix_frames = window_size + analysis_hop;
    let mut padded_input = vec![0.0; prefix_frames + input.len() + suffix_frames];
    padded_input[prefix_frames..prefix_frames + input.len()].copy_from_slice(input);

    // Synthesis hops scale while the samples inside each synthesis window do
    // not. Align the analysis and synthesis window centres before cropping.
    let half_window = window_size as f64 * 0.5;
    let output_start =
        ((prefix_frames as f64 - half_window) * ratio + half_window).round() as usize;
    let output_end = output_start + target_len;
    let config =
        PhaseVocoderConfig::new(&padded_input, output_end, ratio, window_size, analysis_hop);
    let mut engine = DraftPhaseVocoder::new(config, mode);
    engine.process(&padded_input);
    engine.finish()[output_start..output_end].to_vec()
}

struct PhaseVocoderConfig {
    target_len: usize,
    window_size: usize,
    analysis_hop: usize,
    synthesis_hop: f64,
    bins: usize,
    frame_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SpectralPeak {
    bin: usize,
    magnitude: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhasePropagationMode {
    IndependentBins,
    IdentityLocked,
    IdentityLockedStabilityAdaptive,
    IdentityLockedTrackedPeakRegions,
    IdentityLockedMagnitudeSlew,
    IdentityLockedTransientReset,
    IdentityLockedCompressionTransientAnchor,
    AdaptiveTransientTimeline,
    FixedMapPeakTransient,
}

impl PhaseVocoderConfig {
    fn new(
        input: &[Sample],
        target_len: usize,
        ratio: f64,
        window_size: usize,
        analysis_hop: usize,
    ) -> Self {
        Self {
            target_len,
            window_size,
            analysis_hop,
            synthesis_hop: analysis_hop as f64 * ratio,
            bins: window_size / 2 + 1,
            frame_count: (input.len().saturating_sub(window_size)) / analysis_hop + 1,
        }
    }
}

struct DraftPhaseVocoder {
    config: PhaseVocoderConfig,
    mode: PhasePropagationMode,
    window: Vec<f32>,
    analysis: PhaseVocoderAnalysisState,
    propagation: PhaseVocoderPropagationState,
    synthesis: PhaseVocoderSynthesisState,
    frame_schedule: Option<AdaptiveTimelineSchedule>,
    peak_transient: Option<FixedMapPeakState>,
}

struct PhaseVocoderAnalysisState {
    forward: Arc<dyn Fft<f32>>,
    current_magnitudes: Vec<f32>,
    current_phases: Vec<f32>,
    current_peaks: Vec<SpectralPeak>,
    previous_peaks: Vec<SpectralPeak>,
    previous_magnitudes: Vec<f32>,
    current_spectral_stability: f64,
    current_energy: f64,
    previous_energy: f64,
    transient_reset_current_frame: bool,
    buffer: Vec<Complex32>,
    time_weighted_buffer: Option<Vec<Complex32>>,
}

struct PhaseVocoderPropagationState {
    omega: Vec<f32>,
    previous_phase: Vec<f32>,
    synthesis_phase: Vec<f32>,
    previous_synthesis_magnitudes: Vec<f32>,
}

struct PhaseVocoderSynthesisState {
    inverse: Arc<dyn Fft<f32>>,
    spectrum: Vec<Complex32>,
    output: Vec<f32>,
    normalization: Vec<f32>,
}

impl DraftPhaseVocoder {
    fn new(config: PhaseVocoderConfig, mode: PhasePropagationMode) -> Self {
        Self::new_internal(config, mode, None)
    }

    fn new_with_schedule(
        config: PhaseVocoderConfig,
        mode: PhasePropagationMode,
        schedule: AdaptiveTimelineSchedule,
    ) -> Self {
        Self::new_internal(config, mode, Some(schedule))
    }

    fn new_with_fixed_map_peak(config: PhaseVocoderConfig, onset_frames: &[usize]) -> Self {
        let mut engine =
            Self::new_internal(config, PhasePropagationMode::FixedMapPeakTransient, None);
        engine.peak_transient = Some(FixedMapPeakState::new(
            engine.config.frame_count,
            engine.config.bins,
            &engine.window,
            engine.config.analysis_hop,
            onset_frames,
        ));
        engine
    }

    fn new_internal(
        config: PhaseVocoderConfig,
        mode: PhasePropagationMode,
        frame_schedule: Option<AdaptiveTimelineSchedule>,
    ) -> Self {
        let window: Vec<f32> = (0..config.window_size)
            .map(|index| {
                0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / config.window_size as f32).cos()
            })
            .collect();
        let omega: Vec<f32> = (0..config.bins)
            .map(|bin| {
                std::f32::consts::TAU * bin as f32 * config.analysis_hop as f32
                    / config.window_size as f32
            })
            .collect();

        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(config.window_size);
        let inverse = planner.plan_fft_inverse(config.window_size);
        let final_synthesis_start = frame_schedule
            .as_ref()
            .and_then(|schedule| schedule.positions.last().copied())
            .unwrap_or_else(|| {
                ((config.frame_count.saturating_sub(1)) as f64 * config.synthesis_hop).ceil()
                    as usize
            });
        let ola_len = final_synthesis_start + config.window_size + 1;
        let output_len = ola_len.max(config.target_len);

        let time_weighted_buffer = (mode == PhasePropagationMode::FixedMapPeakTransient)
            .then(|| vec![Complex32::new(0.0, 0.0); config.window_size]);
        let analysis = PhaseVocoderAnalysisState {
            forward,
            current_magnitudes: vec![0.0; config.bins],
            current_phases: vec![0.0; config.bins],
            current_peaks: Vec::with_capacity(config.bins / 4),
            previous_peaks: Vec::with_capacity(config.bins / 4),
            previous_magnitudes: vec![0.0; config.bins],
            current_spectral_stability: 1.0,
            current_energy: 0.0,
            previous_energy: 0.0,
            transient_reset_current_frame: false,
            buffer: vec![Complex32::new(0.0, 0.0); config.window_size],
            time_weighted_buffer,
        };
        let propagation = PhaseVocoderPropagationState {
            omega,
            previous_phase: vec![0.0; config.bins],
            synthesis_phase: vec![0.0; config.bins],
            previous_synthesis_magnitudes: vec![0.0; config.bins],
        };
        let synthesis = PhaseVocoderSynthesisState {
            inverse,
            spectrum: vec![Complex32::new(0.0, 0.0); config.window_size],
            output: vec![0.0; output_len],
            normalization: vec![0.0; output_len],
        };

        Self {
            config,
            mode,
            window,
            analysis,
            propagation,
            synthesis,
            frame_schedule,
            peak_transient: None,
        }
    }

    fn process(&mut self, input: &[Sample]) {
        for frame_index in 0..self.config.frame_count {
            self.analyze_frame(input, frame_index);
            self.track_spectral_peaks(frame_index);
            self.prepare_fixed_map_peak_transient(frame_index);
            self.propagate_phase(frame_index);
            self.synthesize_frame(frame_index);
        }
    }

    fn analyze_frame(&mut self, input: &[Sample], frame_index: usize) {
        let analysis_start = frame_index * self.config.analysis_hop;
        self.analysis.current_energy = 0.0;
        let time_center = (self.config.window_size.saturating_sub(1)) as f32 * 0.5;
        for (index, (slot, (sample, weight))) in self
            .analysis
            .buffer
            .iter_mut()
            .zip(
                input[analysis_start..analysis_start + self.config.window_size]
                    .iter()
                    .zip(self.window.iter()),
            )
            .enumerate()
        {
            let windowed = sample * weight;
            self.analysis.current_energy += (windowed * windowed) as f64;
            *slot = Complex32::new(windowed, 0.0);
            if let Some(time_weighted) = self.analysis.time_weighted_buffer.as_mut() {
                time_weighted[index] = Complex32::new(windowed * (index as f32 - time_center), 0.0);
            }
        }
        self.analysis.current_energy /= self.config.window_size as f64;
        self.analysis.forward.process(&mut self.analysis.buffer);
        if let Some(time_weighted) = self.analysis.time_weighted_buffer.as_mut() {
            self.analysis.forward.process(time_weighted);
        }
    }

    fn prepare_fixed_map_peak_transient(&mut self, frame_index: usize) {
        let Some(peak_transient) = self.peak_transient.as_mut() else {
            return;
        };
        let time_weighted = self
            .analysis
            .time_weighted_buffer
            .as_ref()
            .expect("fixed-map peak time-weighted spectrum");
        peak_transient.process_frame(
            frame_index,
            self.config.analysis_hop,
            &self.analysis.current_peaks,
            &self.analysis.current_magnitudes,
            &self.analysis.buffer,
            time_weighted,
        );
    }

    fn track_spectral_peaks(&mut self, frame_index: usize) {
        self.analysis.previous_peaks.clear();
        self.analysis
            .previous_peaks
            .extend(self.analysis.current_peaks.iter().copied());
        self.analysis.current_peaks.clear();
        for (bin, magnitude) in self.analysis.current_magnitudes.iter_mut().enumerate() {
            *magnitude = self.analysis.buffer[bin].norm();
        }
        self.analysis.current_spectral_stability = self.spectral_stability_score(frame_index);
        self.analysis.transient_reset_current_frame =
            self.should_reset_phase_at_transient(frame_index);

        if self.config.bins < 3 {
            self.analysis
                .previous_magnitudes
                .copy_from_slice(&self.analysis.current_magnitudes);
            self.analysis.previous_energy = self.analysis.current_energy;
            return;
        }

        for bin in 1..self.config.bins - 1 {
            let magnitude = self.analysis.current_magnitudes[bin];
            if magnitude <= 1.0e-6 {
                continue;
            }
            if magnitude > self.analysis.current_magnitudes[bin - 1]
                && magnitude >= self.analysis.current_magnitudes[bin + 1]
            {
                self.analysis
                    .current_peaks
                    .push(SpectralPeak { bin, magnitude });
            }
        }

        self.analysis
            .previous_magnitudes
            .copy_from_slice(&self.analysis.current_magnitudes);
        self.analysis.previous_energy = self.analysis.current_energy;
    }

    fn propagate_phase(&mut self, frame_index: usize) {
        let reinitialize_phase = self
            .frame_schedule
            .as_ref()
            .is_some_and(|schedule| schedule.reinitialize_phase[frame_index]);
        let synthesis_hop = self.synthesis_hop_for_frame(frame_index);
        for bin in 0..self.config.bins {
            let phase = self.analysis.buffer[bin].arg();
            self.analysis.current_phases[bin] = phase;
            if frame_index == 0 || self.analysis.transient_reset_current_frame || reinitialize_phase
            {
                self.propagation.synthesis_phase[bin] = phase;
            } else {
                let deviation = wrap_phase(
                    phase - self.propagation.previous_phase[bin] - self.propagation.omega[bin],
                );
                let advance = (self.propagation.omega[bin] + deviation)
                    * (synthesis_hop / self.config.analysis_hop as f64) as f32;
                self.propagation.synthesis_phase[bin] =
                    wrap_phase(self.propagation.synthesis_phase[bin] + advance);
            }
            self.propagation.previous_phase[bin] = phase;
        }

        if self.should_lock_phase_to_peaks(frame_index) {
            match self.mode {
                PhasePropagationMode::IdentityLockedTrackedPeakRegions => {
                    self.lock_phase_to_tracked_peak_regions(frame_index);
                }
                _ => self.lock_phase_to_peaks(),
            }
        }

        if let Some(peak_transient) = self.peak_transient.as_ref() {
            for (bin, selected) in peak_transient
                .reinitialize_bins()
                .iter()
                .copied()
                .enumerate()
            {
                if selected {
                    self.propagation.synthesis_phase[bin] = self.analysis.current_phases[bin];
                }
            }
        }

        for bin in 0..self.config.bins {
            let magnitude = self.synthesis_magnitude(bin, frame_index);
            self.synthesis.spectrum[bin] =
                Complex32::from_polar(magnitude, self.propagation.synthesis_phase[bin]);
            self.propagation.previous_synthesis_magnitudes[bin] = magnitude;
        }
        for bin in 1..self.config.window_size.div_ceil(2) {
            self.synthesis.spectrum[self.config.window_size - bin] =
                self.synthesis.spectrum[bin].conj();
        }
    }

    fn synthesis_magnitude(&self, bin: usize, frame_index: usize) -> f32 {
        let current = self.analysis.current_magnitudes[bin];
        if self.mode != PhasePropagationMode::IdentityLockedMagnitudeSlew
            || frame_index == 0
            || self.analysis.current_spectral_stability < 0.70
        {
            return current;
        }

        let previous = self.propagation.previous_synthesis_magnitudes[bin];
        if previous <= 1.0e-6 || current <= 1.0e-6 {
            return current;
        }
        current.clamp(previous * 0.5, previous * 2.0)
    }

    fn spectral_stability_score(&self, frame_index: usize) -> f64 {
        if frame_index == 0 {
            return 1.0;
        }

        let mut magnitude_sum = 0.0f64;
        let mut absolute_delta_sum = 0.0f64;
        for (current, previous) in self
            .analysis
            .current_magnitudes
            .iter()
            .zip(self.analysis.previous_magnitudes.iter())
        {
            magnitude_sum += (*current + *previous) as f64;
            absolute_delta_sum += (*current - *previous).abs() as f64;
        }

        (1.0 - absolute_delta_sum / (magnitude_sum + 1.0e-12)).clamp(0.0, 1.0)
    }

    fn should_reset_phase_at_transient(&self, frame_index: usize) -> bool {
        if frame_index == 0 {
            return false;
        }

        let mut flux = 0.0f32;
        let mut magnitude_sum = 0.0f32;
        for (current, previous) in self
            .analysis
            .current_magnitudes
            .iter()
            .zip(self.analysis.previous_magnitudes.iter())
        {
            flux += (current - previous).max(0.0);
            magnitude_sum += *current;
        }

        let flux_ratio = flux as f64 / (magnitude_sum as f64 + 1.0e-12);
        let energy_ratio = self.analysis.current_energy / (self.analysis.previous_energy + 1.0e-12);
        match self.mode {
            PhasePropagationMode::IdentityLockedTransientReset => {
                flux_ratio >= 0.30 && energy_ratio >= 1.20
            }
            PhasePropagationMode::IdentityLockedCompressionTransientAnchor => {
                flux_ratio >= 0.55 && energy_ratio >= 1.35
            }
            PhasePropagationMode::IndependentBins
            | PhasePropagationMode::IdentityLocked
            | PhasePropagationMode::IdentityLockedStabilityAdaptive
            | PhasePropagationMode::IdentityLockedTrackedPeakRegions
            | PhasePropagationMode::IdentityLockedMagnitudeSlew
            | PhasePropagationMode::AdaptiveTransientTimeline
            | PhasePropagationMode::FixedMapPeakTransient => false,
        }
    }

    fn should_lock_phase_to_peaks(&self, frame_index: usize) -> bool {
        match self.mode {
            PhasePropagationMode::IdentityLocked
            | PhasePropagationMode::IdentityLockedTransientReset
            | PhasePropagationMode::IdentityLockedCompressionTransientAnchor
            | PhasePropagationMode::IdentityLockedMagnitudeSlew
            | PhasePropagationMode::AdaptiveTransientTimeline
            | PhasePropagationMode::FixedMapPeakTransient => true,
            PhasePropagationMode::IdentityLockedStabilityAdaptive => {
                frame_index == 0 || self.analysis.current_spectral_stability >= 0.70
            }
            PhasePropagationMode::IdentityLockedTrackedPeakRegions => true,
            PhasePropagationMode::IndependentBins => false,
        }
    }

    fn lock_phase_to_peaks(&mut self) {
        if self.analysis.current_peaks.is_empty() {
            return;
        }

        for peak_index in 0..self.analysis.current_peaks.len() {
            let peak = self.analysis.current_peaks[peak_index];
            let peak_phase = self.propagation.synthesis_phase[peak.bin];
            let analysis_peak_phase = self.analysis.current_phases[peak.bin];
            let (left, right) = self.peak_region_bounds(peak_index);

            for bin in left..right {
                let relative_phase =
                    wrap_phase(self.analysis.current_phases[bin] - analysis_peak_phase);
                self.propagation.synthesis_phase[bin] = wrap_phase(peak_phase + relative_phase);
            }
        }
    }

    fn lock_phase_to_tracked_peak_regions(&mut self, frame_index: usize) {
        if frame_index == 0 || self.analysis.previous_peaks.is_empty() {
            self.lock_phase_to_peaks();
            return;
        }
        if self.analysis.current_peaks.is_empty() {
            return;
        }

        for peak_index in 0..self.analysis.current_peaks.len() {
            let peak = self.analysis.current_peaks[peak_index];
            let peak_phase = self.propagation.synthesis_phase[peak.bin];
            let analysis_peak_phase = self.analysis.current_phases[peak.bin];
            let (left, right) = if self.analysis.previous_peaks.iter().any(|previous| {
                previous.bin.abs_diff(peak.bin) <= tracked_peak_max_bin_drift(self.config.bins)
            }) {
                self.peak_region_bounds(peak_index)
            } else {
                (
                    peak.bin.saturating_sub(1),
                    (peak.bin + 2).min(self.config.bins),
                )
            };

            for bin in left..right {
                let relative_phase =
                    wrap_phase(self.analysis.current_phases[bin] - analysis_peak_phase);
                self.propagation.synthesis_phase[bin] = wrap_phase(peak_phase + relative_phase);
            }
        }
    }

    fn peak_region_bounds(&self, peak_index: usize) -> (usize, usize) {
        let peak = self.analysis.current_peaks[peak_index];
        let left = if peak_index == 0 {
            0
        } else {
            (self.analysis.current_peaks[peak_index - 1].bin + peak.bin) / 2 + 1
        };
        let right = self
            .analysis
            .current_peaks
            .get(peak_index + 1)
            .map(|next| (peak.bin + next.bin) / 2 + 1)
            .unwrap_or(self.config.bins);
        (left, right)
    }

    fn synthesize_frame(&mut self, frame_index: usize) {
        self.synthesis.inverse.process(&mut self.synthesis.spectrum);
        let synthesis_start = self
            .frame_schedule
            .as_ref()
            .map(|schedule| schedule.positions[frame_index])
            .unwrap_or_else(|| (frame_index as f64 * self.config.synthesis_hop).round() as usize);
        let scale = 1.0 / self.config.window_size as f32;
        for (index, weight) in self.window.iter().enumerate() {
            let out_index = synthesis_start + index;
            if out_index >= self.synthesis.output.len() {
                break;
            }
            self.synthesis.output[out_index] += self.synthesis.spectrum[index].re * scale * weight;
            self.synthesis.normalization[out_index] += weight * weight;
        }
    }

    fn synthesis_hop_for_frame(&self, frame_index: usize) -> f64 {
        if frame_index == 0 {
            return self.config.synthesis_hop;
        }
        self.frame_schedule
            .as_ref()
            .map(|schedule| {
                schedule.positions[frame_index].saturating_sub(schedule.positions[frame_index - 1])
                    as f64
            })
            .unwrap_or(self.config.synthesis_hop)
    }

    fn uncovered_frames(&self, start: usize, end: usize) -> usize {
        self.synthesis.normalization[start..end]
            .iter()
            .filter(|weight| **weight <= 1.0e-3)
            .count()
    }

    fn finish(mut self) -> Vec<Sample> {
        for (sample, weight) in self
            .synthesis
            .output
            .iter_mut()
            .zip(self.synthesis.normalization.iter())
        {
            if *weight > 1.0e-3 {
                *sample /= *weight;
            }
        }
        self.synthesis.output.resize(self.config.target_len, 0.0);
        self.synthesis.output
    }
}

fn tracked_peak_max_bin_drift(bin_count: usize) -> usize {
    (bin_count / 128).clamp(2, 6)
}

fn wrap_phase(phase: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    phase - tau * (phase / tau).round()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bin_centered_sine(bin: usize, window_size: usize) -> Vec<Sample> {
        (0..window_size)
            .map(|index| {
                (std::f32::consts::TAU * bin as f32 * index as f32 / window_size as f32).sin()
            })
            .collect()
    }

    fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
        let margin = samples.len() / 8;
        let interior = &samples[margin..samples.len() - margin];
        let crossings = interior
            .windows(2)
            .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
            .count();
        crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
    }

    fn rms(samples: &[Sample]) -> f32 {
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len().max(1) as f32)
            .sqrt()
    }

    fn boundary_content_probe(len: usize, edge_frames: usize) -> Vec<Sample> {
        let mut input = vec![0.0; len];
        input[..edge_frames].fill(0.5);
        input[len - edge_frames..].fill(-0.5);
        input
    }

    fn sample_bit_hash(samples: &[Sample]) -> u64 {
        samples.iter().fold(0xcbf2_9ce4_8422_2325, |hash, sample| {
            sample
                .to_bits()
                .to_le_bytes()
                .into_iter()
                .fold(hash, |hash, byte| {
                    (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
                })
        })
    }

    #[test]
    fn phase_vocoder_bit_exact_baseline() {
        let input = (0..8_192)
            .map(|index| {
                let fundamental =
                    (std::f32::consts::TAU * 11.0 * index as f32 / 2_048.0).sin() * 0.6;
                let partial = (std::f32::consts::TAU * 37.0 * index as f32 / 2_048.0).sin() * 0.2;
                let impulse = if index % 997 == 0 { 0.75 } else { 0.0 };
                fundamental + partial + impulse
            })
            .collect::<Vec<_>>();
        let output = transient_reset_phase_vocoder(&input, 12_288, 1.5, 2_048, 512);

        assert_eq!(sample_bit_hash(&output), 0x8255_b183_11f7_78f9);
    }

    #[test]
    fn phase_vocoder_boundary_expansion_preserves_head_and_tail_content() {
        let input = boundary_content_probe(48_000, 384);
        let output = transient_reset_phase_vocoder(&input, 96_000, 2.0, 2_048, 512);
        let edge_span = 2_048;

        assert!(rms(&output[..edge_span]) > 0.01);
        assert!(rms(&output[output.len() - edge_span..]) > 0.01);
    }

    #[test]
    fn phase_vocoder_boundary_compression_preserves_head_and_tail_content() {
        let input = boundary_content_probe(48_000, 384);
        let output = transient_reset_phase_vocoder(&input, 24_000, 0.5, 2_048, 512);
        let edge_span = 1_024;

        assert!(rms(&output[..edge_span]) > 0.01);
        assert!(rms(&output[output.len() - edge_span..]) > 0.01);
    }

    #[test]
    fn tracks_local_spectral_peaks_for_current_frame() {
        let window_size = 256;
        let target_bin = 17;
        let input = bin_centered_sine(target_bin, window_size);
        let config =
            PhaseVocoderConfig::new(&input, window_size, 1.0, window_size, window_size / 4);
        let mut engine = DraftPhaseVocoder::new(config, PhasePropagationMode::IndependentBins);

        engine.analyze_frame(&input, 0);
        engine.track_spectral_peaks(0);

        assert!(
            engine
                .analysis
                .current_peaks
                .iter()
                .any(|peak| peak.bin.abs_diff(target_bin) <= 1),
            "expected a peak near bin {target_bin}, got {:?}",
            engine.analysis.current_peaks
        );
    }

    #[test]
    fn transient_reset_detector_flags_energy_and_flux_jump() {
        let window_size = 256;
        let mut input = vec![0.0; window_size * 2];
        for sample in &mut input[window_size..] {
            *sample = 1.0;
        }
        let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, window_size, window_size);
        let mut engine =
            DraftPhaseVocoder::new(config, PhasePropagationMode::IdentityLockedTransientReset);

        engine.analyze_frame(&input, 0);
        engine.track_spectral_peaks(0);
        assert!(!engine.analysis.transient_reset_current_frame);

        engine.analyze_frame(&input, 1);
        engine.track_spectral_peaks(1);
        assert!(engine.analysis.transient_reset_current_frame);
    }

    #[test]
    fn identity_locking_preserves_peak_neighborhood_phase_offsets() {
        let input = vec![0.0; 512];
        let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, 512, 128);
        let mut engine = DraftPhaseVocoder::new(config, PhasePropagationMode::IdentityLocked);
        engine.analysis.current_peaks.push(SpectralPeak {
            bin: 10,
            magnitude: 1.0,
        });
        engine.analysis.current_phases[9] = 0.20;
        engine.analysis.current_phases[10] = 0.50;
        engine.analysis.current_phases[11] = 0.90;
        engine.propagation.synthesis_phase[10] = 1.25;

        engine.lock_phase_to_peaks();

        assert!((wrap_phase(engine.propagation.synthesis_phase[9] - 0.95)).abs() < 1.0e-6);
        assert!((wrap_phase(engine.propagation.synthesis_phase[10] - 1.25)).abs() < 1.0e-6);
        assert!((wrap_phase(engine.propagation.synthesis_phase[11] - 1.65)).abs() < 1.0e-6);
    }

    #[test]
    fn tracked_peak_regions_narrow_unmatched_peak_locks() {
        let input = vec![0.0; 512];
        let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, 512, 128);
        let mut engine = DraftPhaseVocoder::new(
            config,
            PhasePropagationMode::IdentityLockedTrackedPeakRegions,
        );
        engine.analysis.previous_peaks.push(SpectralPeak {
            bin: 40,
            magnitude: 1.0,
        });
        engine.analysis.current_peaks.push(SpectralPeak {
            bin: 10,
            magnitude: 1.0,
        });
        engine.analysis.current_phases[9] = 0.20;
        engine.analysis.current_phases[10] = 0.50;
        engine.analysis.current_phases[11] = 0.90;
        engine.propagation.synthesis_phase[10] = 1.25;

        engine.lock_phase_to_tracked_peak_regions(1);

        assert_eq!(engine.propagation.synthesis_phase[8], 0.0);
        assert!((wrap_phase(engine.propagation.synthesis_phase[9] - 0.95)).abs() < 1.0e-6);
        assert!((wrap_phase(engine.propagation.synthesis_phase[10] - 1.25)).abs() < 1.0e-6);
        assert!((wrap_phase(engine.propagation.synthesis_phase[11] - 1.65)).abs() < 1.0e-6);
        assert_eq!(engine.propagation.synthesis_phase[12], 0.0);
    }

    #[test]
    fn magnitude_slew_limits_stable_frame_bin_changes() {
        let input = vec![0.0; 512];
        let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, 512, 128);
        let mut engine =
            DraftPhaseVocoder::new(config, PhasePropagationMode::IdentityLockedMagnitudeSlew);

        engine.analysis.current_spectral_stability = 0.75;
        engine.analysis.current_magnitudes[10] = 10.0;
        engine.propagation.previous_synthesis_magnitudes[10] = 2.0;

        assert_eq!(engine.synthesis_magnitude(10, 1), 4.0);

        engine.analysis.current_spectral_stability = 0.69;
        assert_eq!(engine.synthesis_magnitude(10, 1), 10.0);
    }

    #[test]
    fn stability_adaptive_locking_uses_frame_spectral_stability() {
        let input = vec![0.0; 512];
        let config = PhaseVocoderConfig::new(&input, input.len(), 1.0, 512, 128);
        let mut engine = DraftPhaseVocoder::new(
            config,
            PhasePropagationMode::IdentityLockedStabilityAdaptive,
        );

        assert!(engine.should_lock_phase_to_peaks(0));
        engine.analysis.current_spectral_stability = 0.69;
        assert!(!engine.should_lock_phase_to_peaks(1));
        engine.analysis.current_spectral_stability = 0.70;
        assert!(engine.should_lock_phase_to_peaks(1));
    }

    #[test]
    fn draft_phase_vocoder_keeps_independent_bin_baseline() {
        let input = bin_centered_sine(9, 4096);
        let draft = phase_vocoder(&input, 6144, 1.5, 512, 128);
        let baseline = run_phase_vocoder(
            &input,
            6144,
            1.5,
            512,
            128,
            PhasePropagationMode::IndependentBins,
        );

        assert_eq!(draft, baseline);
    }

    #[test]
    fn phase_locked_prototype_honors_output_length_contract() {
        let input = bin_centered_sine(11, 8192);
        for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let output = phase_locked_phase_vocoder(&input, target_len, ratio, 1024, 256);
            assert_eq!(output.len(), target_len, "ratio {ratio}");
        }
    }

    #[test]
    fn stability_adaptive_prototype_honors_output_length_contract() {
        let input = bin_centered_sine(11, 8192);
        for ratio in [0.75, 1.25, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let output = stability_adaptive_phase_vocoder(&input, target_len, ratio, 1024, 256);
            assert_eq!(output.len(), target_len, "ratio {ratio}");
        }
    }

    #[test]
    fn tracked_peak_region_prototype_honors_output_length_contract() {
        let input = bin_centered_sine(11, 8192);
        for ratio in [0.75, 1.25, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let output = tracked_peak_region_phase_vocoder(&input, target_len, ratio, 1024, 256);
            assert_eq!(output.len(), target_len, "ratio {ratio}");
        }
    }

    #[test]
    fn magnitude_slew_prototype_honors_output_length_contract() {
        let input = bin_centered_sine(11, 8192);
        for ratio in [0.75, 1.25, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let output = magnitude_slew_phase_vocoder(&input, target_len, ratio, 1024, 256);
            assert_eq!(output.len(), target_len, "ratio {ratio}");
        }
    }

    #[test]
    fn transient_reset_prototype_honors_output_length_contract() {
        let input = bin_centered_sine(11, 8192);
        for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let output = transient_reset_phase_vocoder(&input, target_len, ratio, 1024, 256);
            assert_eq!(output.len(), target_len, "ratio {ratio}");
        }
    }

    #[test]
    fn transient_reset_uses_phase_locking_for_time_compression() {
        let ratio = 0.75;
        let input = bin_centered_sine(11, 8192);
        let target_len = (input.len() as f64 * ratio).round() as usize;

        assert_eq!(
            transient_reset_phase_vocoder(&input, target_len, ratio, 1024, 256),
            phase_locked_phase_vocoder(&input, target_len, ratio, 1024, 256)
        );
    }

    #[test]
    fn phase_locked_prototype_preserves_tonal_pitch_near_draft_baseline() {
        let sample_rate = 48_000.0;
        let frequency_hz = 468.75;
        let input = (0..48_000)
            .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate).sin())
            .collect::<Vec<_>>();

        for ratio in [0.75, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let draft = phase_vocoder(&input, target_len, ratio, 2048, 512);
            let locked = phase_locked_phase_vocoder(&input, target_len, ratio, 2048, 512);
            let draft_frequency = dominant_frequency_hz(&draft, sample_rate);
            let locked_frequency = dominant_frequency_hz(&locked, sample_rate);

            assert!(
                (locked_frequency - frequency_hz).abs() <= (draft_frequency - frequency_hz).abs() + 3.0,
                "ratio {ratio}: locked frequency {locked_frequency} Hz regressed from draft {draft_frequency} Hz"
            );
        }
    }

    #[test]
    fn transient_reset_prototype_preserves_tonal_pitch_near_draft_baseline() {
        let sample_rate = 48_000.0;
        let frequency_hz = 468.75;
        let input = (0..48_000)
            .map(|index| (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate).sin())
            .collect::<Vec<_>>();

        for ratio in [0.75, 1.5, 2.0] {
            let target_len = (input.len() as f64 * ratio).round() as usize;
            let draft = phase_vocoder(&input, target_len, ratio, 2048, 512);
            let reset = transient_reset_phase_vocoder(&input, target_len, ratio, 2048, 512);
            let draft_frequency = dominant_frequency_hz(&draft, sample_rate);
            let reset_frequency = dominant_frequency_hz(&reset, sample_rate);

            assert!(
                (reset_frequency - frequency_hz).abs()
                    <= (draft_frequency - frequency_hz).abs() + 3.0,
                "ratio {ratio}: reset frequency {reset_frequency} Hz regressed from draft {draft_frequency} Hz"
            );
        }
    }

    #[test]
    fn phase_vocoder_output_is_deterministic_with_peak_tracking() {
        let input = bin_centered_sine(9, 4096);
        let first = phase_vocoder(&input, 6144, 1.5, 512, 128);
        let repeated = phase_vocoder(&input, 6144, 1.5, 512, 128);

        assert_eq!(first, repeated);
    }
}
