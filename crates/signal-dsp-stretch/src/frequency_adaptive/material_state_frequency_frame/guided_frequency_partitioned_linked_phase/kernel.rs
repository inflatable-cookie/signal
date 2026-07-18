use super::*;

#[derive(Clone, Debug)]
pub(in super::super) struct Workspace {
    sample_rate_hz: usize,
    common_hop: usize,
    previous_analysis: [Vec<f64>; CHANNEL_CAPACITY],
    previous_synthesis: [Vec<f64>; CHANNEL_CAPACITY],
    previous_energy: [Vec<f64>; CHANNEL_CAPACITY],
    previous_regions: Vec<Region>,
    has_previous: bool,
    pub(in super::super) region_high_water: usize,
    pub(in super::super) counts: StateCounts,
    pub(in super::super) updates: usize,
    pub(in super::super) region_visits: usize,
}

impl Workspace {
    pub(in super::super) fn new(sample_rate_hz: usize, common_hop: usize) -> Self {
        Self {
            sample_rate_hz,
            common_hop,
            previous_analysis: std::array::from_fn(|_| Vec::with_capacity(POSITIVE_ATOM_CAPACITY)),
            previous_synthesis: std::array::from_fn(|_| Vec::with_capacity(POSITIVE_ATOM_CAPACITY)),
            previous_energy: std::array::from_fn(|_| Vec::with_capacity(POSITIVE_ATOM_CAPACITY)),
            previous_regions: Vec::with_capacity(REGION_CAPACITY),
            has_previous: false,
            region_high_water: 0,
            counts: StateCounts::default(),
            updates: 0,
            region_visits: 0,
        }
    }

    pub(in super::super) fn process(
        &mut self,
        current: &[Vec<Complex64>; CHANNEL_CAPACITY],
        frequencies_hz: &[f64],
        decision: Decision,
    ) -> Result<[Vec<Complex64>; CHANNEL_CAPACITY], CapacityExceeded> {
        self.process_decisions(current, frequencies_hz, &vec![decision; current[0].len()])
    }

    fn process_decisions(
        &mut self,
        current: &[Vec<Complex64>; CHANNEL_CAPACITY],
        frequencies_hz: &[f64],
        decisions: &[Decision],
    ) -> Result<[Vec<Complex64>; CHANNEL_CAPACITY], CapacityExceeded> {
        let bands = current[0].len();
        validate_request(CHANNEL_CAPACITY, SIGNED_ATOM_CAPACITY, bands, 1)?;
        if current[1].len() != bands || frequencies_hz.len() != bands || decisions.len() != bands {
            return Err(CapacityExceeded::PositiveAtoms);
        }

        let energy = std::array::from_fn::<_, CHANNEL_CAPACITY, _>(|channel| {
            current[channel]
                .iter()
                .map(Complex64::norm_sqr)
                .collect::<Vec<_>>()
        });
        let joint = (0..bands)
            .map(|band| energy[0][band].max(energy[1][band]))
            .collect::<Vec<_>>();
        let mut regions = peak_regions(&joint)?;
        for region in &mut regions {
            region.owner = usize::from(energy[1][region.peak] > energy[0][region.peak]);
        }
        self.region_high_water = self.region_high_water.max(regions.len());
        self.region_visits += regions.len();
        let ordinary = self.ordinary(current, &energy, frequencies_hz);
        let mut output = current.clone();

        for region in &regions {
            let prior = self
                .previous_regions
                .iter()
                .find(|candidate| (region.first..region.end).contains(&candidate.peak));
            let owner = region.owner;
            let compatible = self.has_previous
                && prior.is_some()
                && energy[owner][region.peak] > ENERGY_FLOOR
                && self.previous_energy[owner][region.peak] > ENERGY_FLOOR;
            let linked = compatible && frequencies_hz[region.peak] < LINK_LIMIT_HZ;
            if decisions[region.first..region.end].contains(&Decision::Locked) {
                self.counts.linked_regions += usize::from(linked);
                self.counts.unlinked_regions += usize::from(!linked);
                self.counts.owner_switches +=
                    usize::from(prior.is_some_and(|candidate| candidate.owner != region.owner));
            }
            for channel in 0..CHANNEL_CAPACITY {
                let trajectory_channel = if linked { owner } else { channel };
                let reference_analysis_channel = if linked { owner } else { channel };
                let trajectory = ordinary[trajectory_channel][region.peak];
                let reference_analysis = current[reference_analysis_channel][region.peak].arg();
                for band in region.first..region.end {
                    let phase = match decisions[band] {
                        Decision::Reset | Decision::Attack => current[channel][band].arg(),
                        Decision::Ordinary | Decision::Unlocked => ordinary[channel][band],
                        Decision::Locked => {
                            trajectory + wrap(current[channel][band].arg() - reference_analysis)
                        }
                    };
                    output[channel][band] =
                        Complex64::from_polar(current[channel][band].norm(), phase);
                }
            }
        }
        for decision in decisions {
            self.counts.states[decision.index()] += 1;
        }
        self.store(current, &output, &energy, regions);
        self.updates += 1;
        Ok(output)
    }

    fn ordinary(
        &self,
        current: &[Vec<Complex64>; CHANNEL_CAPACITY],
        energy: &[Vec<f64>; CHANNEL_CAPACITY],
        frequencies_hz: &[f64],
    ) -> [Vec<f64>; CHANNEL_CAPACITY] {
        std::array::from_fn(|channel| {
            (0..current[channel].len())
                .map(|band| {
                    let analysis = current[channel][band].arg();
                    if !self.has_previous
                        || energy[channel][band] <= ENERGY_FLOOR
                        || self.previous_energy[channel][band] <= ENERGY_FLOOR
                    {
                        analysis
                    } else {
                        let expected = std::f64::consts::TAU * frequencies_hz[band]
                            / self.sample_rate_hz as f64
                            * self.common_hop as f64;
                        let observed = expected
                            + wrap(analysis - self.previous_analysis[channel][band] - expected);
                        self.previous_synthesis[channel][band] + observed
                    }
                })
                .collect()
        })
    }

    fn store(
        &mut self,
        current: &[Vec<Complex64>; CHANNEL_CAPACITY],
        output: &[Vec<Complex64>; CHANNEL_CAPACITY],
        energy: &[Vec<f64>; CHANNEL_CAPACITY],
        regions: Vec<Region>,
    ) {
        for channel in 0..CHANNEL_CAPACITY {
            self.previous_analysis[channel].clear();
            self.previous_analysis[channel]
                .extend(current[channel].iter().map(|value| value.arg()));
            self.previous_synthesis[channel].clear();
            self.previous_synthesis[channel]
                .extend(output[channel].iter().map(|value| value.arg()));
            self.previous_energy[channel].clear();
            self.previous_energy[channel].extend_from_slice(&energy[channel]);
        }
        self.previous_regions.clear();
        self.previous_regions.extend(regions);
        self.has_previous = true;
    }
}
