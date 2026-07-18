use super::*;
use crate::frequency_adaptive::material_state_frequency_frame::{
    guided_frequency_partitioned_linked_phase::wrap, HASH_OFFSET,
};

const EXACT_FLOOR: f64 = 1.0e-12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Stage {
    SourceToCurrent,
    StateCommit,
    LayerProjection,
    InverseSlice,
    OuterOverlap,
}

impl Stage {
    pub const ALL: [Self; 5] = [
        Self::SourceToCurrent,
        Self::StateCommit,
        Self::LayerProjection,
        Self::InverseSlice,
        Self::OuterOverlap,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::SourceToCurrent => 0,
            Self::StateCommit => 1,
            Self::LayerProjection => 2,
            Self::InverseSlice => 3,
            Self::OuterOverlap => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::SourceToCurrent => "source-to-current",
            Self::StateCommit => "state-commit",
            Self::LayerProjection => "layer-projection",
            Self::InverseSlice => "inverse-slice",
            Self::OuterOverlap => "outer-overlap",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Event {
    pub stage: Stage,
    pub residual: f64,
    pub magnitude_residual: f64,
    pub phase_residual: f64,
    pub source_position: f64,
    pub output_position: isize,
    pub slice_start: isize,
    pub layer: usize,
    pub scale: usize,
    pub state: usize,
    pub region: usize,
    pub owner_channel: usize,
    pub owner_switched: bool,
    pub boundary: bool,
    pub before_magnitudes: [f64; 2],
    pub after_magnitudes: [f64; 2],
    pub before_phase_relation: f64,
    pub after_phase_relation: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct StageSummary {
    pub observations: usize,
    pub divergent: usize,
    pub maximum_residual: f64,
    pub first: Option<Event>,
    pub worst: Option<Event>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RenderAttribution {
    pub stages: [StageSummary; 5],
    pub by_state: [usize; 5],
    pub by_scale: [usize; 3],
    pub by_layer: [usize; 2],
    pub boundary_divergences: usize,
    pub owner_switch_divergences: usize,
    pub hash: u64,
}

pub(super) struct TraceCollector {
    target_length: usize,
    reference_output: [Vec<f64>; 2],
    stages: [StageSummary; 5],
    by_state: [usize; 5],
    by_scale: [usize; 3],
    by_layer: [usize; 2],
    boundary_divergences: usize,
    owner_switch_divergences: usize,
    previous_owners: Vec<usize>,
    hash: u64,
}

impl TraceCollector {
    pub fn new(target_length: usize) -> Self {
        Self {
            target_length,
            reference_output: std::array::from_fn(|_| vec![0.0; target_length]),
            stages: std::array::from_fn(|_| StageSummary::default()),
            by_state: [0; 5],
            by_scale: [0; 3],
            by_layer: [0; 2],
            boundary_divergences: 0,
            owner_switch_divergences: 0,
            previous_owners: Vec::new(),
            hash: HASH_OFFSET,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn observe_frame(
        &mut self,
        analysis: &analysis::FrameAnalysis,
        decisions: &[Decision],
        decided: &Frame,
        projected: &[Frame; OUTPUT_SLICE_CAPACITY],
        geometry: &Geometry,
        positive: &[usize],
        source_position: f64,
        output_position: isize,
        boundary: bool,
    ) {
        let (regions, owners) = regions(&analysis.current);
        for band in 0..positive.len() {
            let region = regions[band];
            let owner_channel = owners[region];
            let owner_switched = self
                .previous_owners
                .get(region)
                .is_some_and(|owner| *owner != owner_channel);
            let scale = geometry.representation.bands[positive[band]].scale.index();
            for layer in 0..OUTPUT_SLICE_CAPACITY {
                self.observe_complex(
                    Stage::SourceToCurrent,
                    pair(&analysis.layers[layer], band),
                    pair(&analysis.current, band),
                    source_position,
                    output_position,
                    0,
                    layer,
                    scale,
                    decisions[band],
                    region,
                    owner_channel,
                    owner_switched,
                    boundary,
                );
                self.observe_complex(
                    Stage::LayerProjection,
                    pair(&analysis.layers[layer], band),
                    pair(&projected[layer], band),
                    source_position,
                    output_position,
                    0,
                    layer,
                    scale,
                    decisions[band],
                    region,
                    owner_channel,
                    owner_switched,
                    boundary,
                );
            }
            self.observe_complex(
                Stage::StateCommit,
                pair(&analysis.current, band),
                pair(decided, band),
                source_position,
                output_position,
                0,
                0,
                scale,
                decisions[band],
                region,
                owner_channel,
                owner_switched,
                boundary,
            );
        }
        self.previous_owners = owners;
    }

    pub fn observe_slice(
        &mut self,
        start: isize,
        reference: &[Vec<f64>; 2],
        actual: &[Vec<f64>; 2],
        output: &[Vec<f64>; 2],
    ) {
        let inverse = waveform_residual(reference, actual);
        self.observe_waveform(Stage::InverseSlice, inverse, start, false);
        for channel in 0..2 {
            for (local, sample) in reference[channel].iter().enumerate() {
                let logical = start + local as isize;
                if (0..self.target_length as isize).contains(&logical) {
                    self.reference_output[channel][logical as usize] += sample;
                }
            }
        }
        let first = start.max(0) as usize;
        let end =
            (start + reference[0].len() as isize).clamp(0, self.target_length as isize) as usize;
        if first < end {
            let reference =
                std::array::from_fn(|channel| self.reference_output[channel][first..end].to_vec());
            let actual = std::array::from_fn(|channel| output[channel][first..end].to_vec());
            self.observe_waveform(
                Stage::OuterOverlap,
                waveform_residual(&reference, &actual),
                start,
                first == 0 || end == self.target_length,
            );
        }
    }

    pub fn finish(self) -> RenderAttribution {
        RenderAttribution {
            stages: self.stages,
            by_state: self.by_state,
            by_scale: self.by_scale,
            by_layer: self.by_layer,
            boundary_divergences: self.boundary_divergences,
            owner_switch_divergences: self.owner_switch_divergences,
            hash: self.hash,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn observe_complex(
        &mut self,
        stage: Stage,
        before: [Complex64; 2],
        after: [Complex64; 2],
        source_position: f64,
        output_position: isize,
        slice_start: isize,
        layer: usize,
        scale: usize,
        state: Decision,
        region: usize,
        owner_channel: usize,
        owner_switched: bool,
        boundary: bool,
    ) {
        let Some((magnitude_residual, phase_residual)) = relation_residual(before, after) else {
            return;
        };
        let event = Event {
            stage,
            residual: magnitude_residual.max(phase_residual),
            magnitude_residual,
            phase_residual,
            source_position,
            output_position,
            slice_start,
            layer,
            scale,
            state: state.index(),
            region,
            owner_channel,
            owner_switched,
            boundary,
            before_magnitudes: before.map(|value| value.norm()),
            after_magnitudes: after.map(|value| value.norm()),
            before_phase_relation: wrap(before[1].arg() - before[0].arg()),
            after_phase_relation: wrap(after[1].arg() - after[0].arg()),
        };
        self.record(event);
    }

    fn observe_waveform(&mut self, stage: Stage, residual: f64, start: isize, boundary: bool) {
        self.record(Event {
            stage,
            residual,
            magnitude_residual: residual,
            phase_residual: 0.0,
            source_position: 0.0,
            output_position: start,
            slice_start: start,
            layer: 0,
            scale: 0,
            state: Decision::Ordinary.index(),
            region: 0,
            owner_channel: 0,
            owner_switched: false,
            boundary,
            before_magnitudes: [0.0; 2],
            after_magnitudes: [0.0; 2],
            before_phase_relation: 0.0,
            after_phase_relation: 0.0,
        });
    }

    fn record(&mut self, event: Event) {
        let summary = &mut self.stages[event.stage.index()];
        summary.observations += 1;
        if event.residual > EXACT_FLOOR {
            summary.divergent += 1;
            if matches!(
                event.stage,
                Stage::SourceToCurrent | Stage::StateCommit | Stage::LayerProjection
            ) {
                self.by_state[event.state] += 1;
                self.by_scale[event.scale] += 1;
                self.by_layer[event.layer] += 1;
                self.owner_switch_divergences += usize::from(event.owner_switched);
            }
            self.boundary_divergences += usize::from(event.boundary);
            if summary.first.is_none() {
                summary.first = Some(event.clone());
            }
        }
        if event.residual > summary.maximum_residual {
            summary.maximum_residual = event.residual;
            summary.worst = Some(event.clone());
        }
        hash_u64(&mut self.hash, event.stage.index() as u64);
        hash_u64(&mut self.hash, event.residual.to_bits());
        hash_u64(&mut self.hash, event.output_position as u64);
        hash_u64(&mut self.hash, event.state as u64);
        hash_u64(&mut self.hash, event.region as u64);
    }
}

fn pair(frame: &Frame, band: usize) -> [Complex64; 2] {
    [frame[0][band], frame[1][band]]
}

fn relation_residual(before: [Complex64; 2], after: [Complex64; 2]) -> Option<(f64, f64)> {
    let before_energy = before.map(|value| value.norm_sqr());
    let after_energy = after.map(|value| value.norm_sqr());
    if before_energy.iter().any(|energy| *energy <= 1.0e-24)
        || after_energy.iter().any(|energy| *energy <= 1.0e-24)
    {
        return None;
    }
    let normalize = |energy: [f64; 2]| energy[0] / (energy[0] + energy[1]);
    let magnitude = (normalize(before_energy) - normalize(after_energy)).abs();
    let before_phase = wrap(before[1].arg() - before[0].arg());
    let after_phase = wrap(after[1].arg() - after[0].arg());
    Some((magnitude, wrap(after_phase - before_phase).abs()))
}

fn waveform_residual(before: &[Vec<f64>; 2], after: &[Vec<f64>; 2]) -> f64 {
    let normalized = |channels: &[Vec<f64>; 2]| {
        let gram = channels[0]
            .iter()
            .zip(&channels[1])
            .fold([0.0; 3], |sum, (left, right)| {
                [
                    sum[0] + left * left,
                    sum[1] + right * right,
                    sum[2] + left * right,
                ]
            });
        let trace = (gram[0] + gram[1]).max(f64::MIN_POSITIVE);
        gram.map(|value| value / trace)
    };
    normalized(before)
        .into_iter()
        .zip(normalized(after))
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn regions(current: &Frame) -> (Vec<usize>, Vec<usize>) {
    let energy = (0..current[0].len())
        .map(|band| current[0][band].norm_sqr().max(current[1][band].norm_sqr()))
        .collect::<Vec<_>>();
    let mut peaks = (0..energy.len())
        .filter(|band| {
            energy[*band] > 1.0e-24
                && !(band.saturating_sub(2)..(*band + 3).min(energy.len())).any(|other| {
                    other != *band
                        && (energy[other] > energy[*band]
                            || (other < *band && energy[other] == energy[*band]))
                })
        })
        .collect::<Vec<_>>();
    if peaks.is_empty() && !energy.is_empty() {
        peaks.push(
            energy
                .iter()
                .enumerate()
                .max_by(|left, right| left.1.total_cmp(right.1))
                .map_or(0, |(band, _)| band),
        );
    }
    let boundaries = peaks
        .windows(2)
        .map(|pair| {
            (pair[0] + 1..pair[1])
                .min_by(|left, right| energy[*left].total_cmp(&energy[*right]))
                .unwrap_or(pair[0])
        })
        .collect::<Vec<_>>();
    let mut regions = vec![0; energy.len()];
    for index in 0..peaks.len() {
        let first = index
            .checked_sub(1)
            .map_or(0, |prior| boundaries[prior] + 1);
        let end = boundaries
            .get(index)
            .map_or(energy.len(), |boundary| boundary + 1);
        regions[first..end].fill(index);
    }
    let owners = peaks
        .iter()
        .map(|peak| usize::from(current[1][*peak].norm_sqr() > current[0][*peak].norm_sqr()))
        .collect();
    (regions, owners)
}
