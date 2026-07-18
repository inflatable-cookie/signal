use super::{analysis::FrameAnalysis, *};
use crate::frequency_adaptive::{
    material_state_frequency_frame::material_phase::phase::{material_operator, MaterialState},
    source_studied::faithful_predictor::linked_stereo::shared_rotation_region_locked::phase::{
        regions, Region,
    },
};

#[derive(Clone)]
struct RegionMemory {
    region: Region,
    owner: usize,
    rotation: f64,
    phase: [f64; 2],
    energy: [f64; 2],
    frequency: f64,
}

pub(super) struct PhaseState {
    previous_regions: Vec<RegionMemory>,
    previous_source: Option<f64>,
    pub states: StateCounts,
    pub relations: RelationCounts,
    pub maximum_relation_error: f64,
}

impl PhaseState {
    pub fn new() -> Self {
        Self {
            previous_regions: Vec::new(),
            previous_source: None,
            states: StateCounts::default(),
            relations: RelationCounts::default(),
            maximum_relation_error: 0.0,
        }
    }

    pub fn advance(
        &mut self,
        analysis: &FrameAnalysis,
        representation: &Representation,
        positive: &[usize],
        source_position: f64,
        ratio: f64,
        output_time: usize,
    ) -> [[Vec<Complex64>; 2]; 2] {
        let mut output = std::array::from_fn(|_| {
            std::array::from_fn(|_| vec![Complex64::default(); positive.len()])
        });
        let channel_energy = analysis
            .atoms
            .iter()
            .map(|atom| {
                std::array::from_fn(|channel| {
                    (0..2)
                        .map(|layer| atom.source.magnitudes(layer)[channel].powi(2))
                        .fold(0.0_f64, f64::max)
                })
            })
            .collect::<Vec<[f64; 2]>>();
        let energy = channel_energy
            .iter()
            .map(|channels| channels[0].max(channels[1]))
            .collect::<Vec<_>>();
        if energy.iter().all(|value| *value == 0.0) {
            self.states.silent += positive.len();
            self.relations.silent += positive.len();
            self.previous_regions.clear();
            self.previous_source = Some(source_position);
            return output;
        }

        let continuous = self
            .previous_source
            .is_some_and(|previous| source_position > previous);
        let analysis_delta = self
            .previous_source
            .map_or(0.0, |previous| source_position - previous);
        let mut next_regions = Vec::new();
        let frame_regions = regions(&energy);
        self.states.regions += frame_regions.len();
        for region in frame_regions {
            let owner =
                usize::from(channel_energy[region.peak][1] > channel_energy[region.peak][0]);
            let decision_layer = usize::from(
                analysis.atoms[region.peak].source.magnitudes(1)[owner]
                    > analysis.atoms[region.peak].source.magnitudes(0)[owner],
            );
            let (decision, _, _) = analysis.atoms[region.peak]
                .source
                .base(decision_layer, owner);
            let phase = [decision[0].arg(), decision[1].arg()];
            let owner_energy = channel_energy[region.peak];
            let band = positive[region.peak];
            let frequency = std::f64::consts::TAU * representation.bands[band].center as f64
                / representation.fft_frames as f64;
            let predecessor = continuous
                .then(|| {
                    self.previous_regions
                        .iter()
                        .find(|prior| (prior.region.first..prior.region.end).contains(&region.peak))
                })
                .flatten();
            let rotation = predecessor
                .filter(|prior| analysis_delta > 0.0 && prior.energy[owner] > 0.0)
                .map(|prior| {
                    tracked_rotation(prior, owner, phase[owner], frequency, analysis_delta)
                })
                .unwrap_or(0.0);
            if predecessor.is_some_and(|prior| analysis_delta > 0.0 && prior.energy[owner] > 0.0) {
                self.states.tracked += 1;
                self.states.owner_switches +=
                    usize::from(predecessor.is_some_and(|prior| prior.owner != owner));
            } else {
                self.states.reset += 1;
            }

            for local in region.first..region.end {
                let band = positive[local];
                let atom = &analysis.atoms[local];
                let (gain, phase_delta, material_state) = material_operator(
                    atom.material,
                    &analysis.centers,
                    analysis.center_position,
                    representation.bands[band].scale,
                    ratio,
                    output_time,
                    band,
                    rotation,
                );
                match material_state {
                    MaterialState::Shoulder => self.states.shoulder += 1,
                    MaterialState::Reset => self.states.reset += 1,
                    MaterialState::Locked => self.states.locked += 1,
                    MaterialState::Diffuse => self.states.diffuse += 1,
                }
                let reference = usize::from(channel_energy[local][1] > channel_energy[local][0]);
                let relation_layer = usize::from(
                    atom.source.magnitudes(1)[reference] > atom.source.magnitudes(0)[reference],
                );
                let (relation, counts) = atom.source.shared_relation(relation_layer, reference);
                self.relations.add(counts);
                let operator = Complex64::from_polar(gain, phase_delta);
                for layer in 0..2 {
                    let (base, error) = atom.source.base_with_relation(layer, reference, relation);
                    self.maximum_relation_error = self.maximum_relation_error.max(error);
                    for channel in 0..2 {
                        output[layer][channel][local] = base[channel] * operator;
                    }
                }
            }
            next_regions.push(RegionMemory {
                region,
                owner,
                rotation,
                phase,
                energy: owner_energy,
                frequency,
            });
        }
        self.previous_regions = next_regions;
        self.previous_source = Some(source_position);
        output
    }
}

fn tracked_rotation(
    prior: &RegionMemory,
    owner: usize,
    current_phase: f64,
    current_frequency: f64,
    analysis_delta: f64,
) -> f64 {
    let expected = (prior.frequency + current_frequency) * 0.5 * analysis_delta;
    let observed = expected + wrap(current_phase - prior.phase[owner] - expected);
    let synthesis_phase =
        prior.phase[owner] + prior.rotation + observed * COMMON_HOP as f64 / analysis_delta;
    wrap(synthesis_phase - current_phase)
}
