use rustfft::num_complex::Complex32;

use super::SpectralPeak;

mod evidence;

pub(crate) use evidence::{
    FixedMapPeakEventEvidence, FixedMapPeakEvidence, FixedMapPeakRegionEvidence,
};

const PEAK_TRANSIENT_SENSITIVITY: f64 = 1.5;
const MINIMUM_REGION_ENERGY: f64 = 1.0e-12;

#[derive(Clone, Debug)]
struct PeakEventState {
    onset_frame: usize,
    first_analysis_frame: Option<usize>,
    last_analysis_frame: Option<usize>,
    reinitialized_analysis_frame: Option<usize>,
    collected_peak_regions: usize,
    reinitialized_bins: usize,
}

pub(super) struct FixedMapPeakState {
    frame_events: Vec<Option<usize>>,
    events: Vec<PeakEventState>,
    active_event: Option<usize>,
    collected_bins: Vec<bool>,
    reinitialize_bins: Vec<bool>,
    previous_energy_position_frames: Option<f64>,
    center_threshold_frames: f64,
    candidate_regions: Vec<FixedMapPeakRegionEvidence>,
    threshold_crossings: usize,
}

impl FixedMapPeakState {
    pub(super) fn new(
        frame_count: usize,
        bins: usize,
        window: &[f32],
        analysis_hop: usize,
        onset_frames: &[usize],
    ) -> Self {
        let mut onsets = onset_frames.to_vec();
        onsets.sort_unstable();
        onsets.dedup();
        let mut events = onsets
            .iter()
            .copied()
            .map(|onset_frame| PeakEventState {
                onset_frame,
                first_analysis_frame: None,
                last_analysis_frame: None,
                reinitialized_analysis_frame: None,
                collected_peak_regions: 0,
                reinitialized_bins: 0,
            })
            .collect::<Vec<_>>();
        let mut frame_events = vec![None; frame_count];
        let half_window = window.len() / 2;
        for (frame_index, frame_event) in frame_events.iter_mut().enumerate() {
            let source_center = frame_index * analysis_hop;
            let nearest = onsets
                .iter()
                .enumerate()
                .min_by_key(|(_, onset)| source_center.abs_diff(**onset));
            let Some((event_index, onset)) = nearest else {
                continue;
            };
            if source_center.abs_diff(*onset) > half_window {
                continue;
            }
            *frame_event = Some(event_index);
            let event = &mut events[event_index];
            event.first_analysis_frame.get_or_insert(frame_index);
            event.last_analysis_frame = Some(frame_index);
        }

        Self {
            frame_events,
            events,
            active_event: None,
            collected_bins: vec![false; bins],
            reinitialize_bins: vec![false; bins],
            previous_energy_position_frames: None,
            center_threshold_frames: reference_ramp_energy_position(window),
            candidate_regions: Vec::new(),
            threshold_crossings: 0,
        }
    }

    pub(super) fn process_frame(
        &mut self,
        frame_index: usize,
        analysis_hop: usize,
        peaks: &[SpectralPeak],
        magnitudes: &[f32],
        spectrum: &[Complex32],
        time_weighted_spectrum: &[Complex32],
    ) {
        self.reinitialize_bins.fill(false);
        let event_index = self.frame_events.get(frame_index).copied().flatten();
        if event_index != self.active_event {
            self.active_event = event_index;
            self.collected_bins.fill(false);
            self.previous_energy_position_frames = None;
        }
        let Some(event_index) = event_index else {
            return;
        };
        if self.events[event_index]
            .reinitialized_analysis_frame
            .is_some()
        {
            return;
        }

        let candidate_threshold = self.center_threshold_frames * PEAK_TRANSIENT_SENSITIVITY;
        for peak in peaks {
            let (first_bin, end_bin) = peak_minimum_region_bounds(peak.bin, magnitudes);
            let Some(position) =
                region_energy_position(first_bin, end_bin, spectrum, time_weighted_spectrum)
            else {
                continue;
            };
            if position <= candidate_threshold {
                continue;
            }
            self.collected_bins[first_bin..end_bin].fill(true);
            self.events[event_index].collected_peak_regions += 1;
            self.candidate_regions.push(FixedMapPeakRegionEvidence {
                event_index,
                analysis_frame_index: frame_index,
                source_center_frame: frame_index * analysis_hop,
                peak_bin: peak.bin,
                first_bin,
                end_bin,
                energy_position_frames: position,
            });
        }

        let Some(current_position) =
            masked_energy_position(&self.collected_bins, spectrum, time_weighted_spectrum)
        else {
            return;
        };
        let crossed_center_threshold =
            self.previous_energy_position_frames
                .is_some_and(|previous| {
                    previous > self.center_threshold_frames
                        && current_position <= self.center_threshold_frames
                });
        self.previous_energy_position_frames = Some(current_position);
        if !crossed_center_threshold {
            return;
        }

        self.reinitialize_bins.copy_from_slice(&self.collected_bins);
        let event = &mut self.events[event_index];
        event.reinitialized_analysis_frame = Some(frame_index);
        event.reinitialized_bins = self
            .reinitialize_bins
            .iter()
            .filter(|selected| **selected)
            .count();
        self.threshold_crossings += 1;
    }

    pub(super) fn reinitialize_bins(&self) -> &[bool] {
        &self.reinitialize_bins
    }

    pub(super) fn evidence(&self) -> FixedMapPeakEvidence {
        FixedMapPeakEvidence {
            center_threshold_frames: self.center_threshold_frames,
            events: self
                .events
                .iter()
                .map(|event| FixedMapPeakEventEvidence {
                    onset_frame: event.onset_frame,
                    first_analysis_frame: event.first_analysis_frame,
                    last_analysis_frame: event.last_analysis_frame,
                    reinitialized_analysis_frame: event.reinitialized_analysis_frame,
                    collected_peak_regions: event.collected_peak_regions,
                    reinitialized_bins: event.reinitialized_bins,
                })
                .collect(),
            candidate_regions: self.candidate_regions.clone(),
            threshold_crossings: self.threshold_crossings,
        }
    }
}

fn reference_ramp_energy_position(window: &[f32]) -> f64 {
    let center = (window.len().saturating_sub(1)) as f64 * 0.5;
    let scale = window.len().saturating_sub(1).max(1) as f64;
    let mut weighted_position = 0.0;
    let mut energy = 0.0;
    for (index, weight) in window.iter().enumerate() {
        let ramp = index as f64 / scale;
        let amplitude = *weight as f64 * ramp;
        let bin_energy = amplitude * amplitude;
        weighted_position += (index as f64 - center) * bin_energy;
        energy += bin_energy;
    }
    weighted_position / (energy + MINIMUM_REGION_ENERGY)
}

fn peak_minimum_region_bounds(peak_bin: usize, magnitudes: &[f32]) -> (usize, usize) {
    let mut first_bin = peak_bin.min(magnitudes.len().saturating_sub(1));
    while first_bin > 0 && magnitudes[first_bin - 1] <= magnitudes[first_bin] {
        first_bin -= 1;
    }
    let mut last_bin = peak_bin.min(magnitudes.len().saturating_sub(1));
    while last_bin + 1 < magnitudes.len() && magnitudes[last_bin + 1] <= magnitudes[last_bin] {
        last_bin += 1;
    }
    (first_bin, last_bin.saturating_add(1))
}

fn region_energy_position(
    first_bin: usize,
    end_bin: usize,
    spectrum: &[Complex32],
    time_weighted_spectrum: &[Complex32],
) -> Option<f64> {
    let mut numerator = 0.0;
    let mut energy = 0.0;
    for bin in first_bin..end_bin {
        let value = spectrum[bin];
        numerator += (time_weighted_spectrum[bin] * value.conj()).re as f64;
        energy += value.norm_sqr() as f64;
    }
    (energy > MINIMUM_REGION_ENERGY).then_some(numerator / energy)
}

fn masked_energy_position(
    selected: &[bool],
    spectrum: &[Complex32],
    time_weighted_spectrum: &[Complex32],
) -> Option<f64> {
    let mut numerator = 0.0;
    let mut energy = 0.0;
    for (bin, selected) in selected.iter().copied().enumerate() {
        if !selected {
            continue;
        }
        let value = spectrum[bin];
        numerator += (time_weighted_spectrum[bin] * value.conj()).re as f64;
        energy += value.norm_sqr() as f64;
    }
    (energy > MINIMUM_REGION_ENERGY).then_some(numerator / energy)
}

#[cfg(test)]
mod tests;
