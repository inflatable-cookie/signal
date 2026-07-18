use super::{geometry::*, *};

mod phase;
mod regions;
use phase::*;
use regions::*;

const SUPPORT_FLOOR: f64 = 1.0e-24;
const LINK_LIMIT_HZ: f64 = 6_000.0;

impl Prepared {
    pub(super) fn process_state_tick(
        &mut self,
        current: &[Complex64],
        guidance: &[MaterialGuidance],
        control: StateTickControl,
        output: &mut [Complex64],
        states: &mut [TerminalState],
    ) -> Result<StateTickReport, StateError> {
        let atoms = self.owned_bins.iter().sum::<usize>();
        let coefficients = self.channels * atoms;
        if current.len() != coefficients {
            return Err(StateError::CurrentShape);
        }
        if guidance.len() != atoms {
            return Err(StateError::GuidanceShape);
        }
        if output.len() != coefficients {
            return Err(StateError::OutputShape);
        }
        if states.len() != atoms
            || self.phase.len() != 2 * coefficients
            || self.regions.len() != 2 * coefficients
        {
            return Err(StateError::StateShape);
        }
        if !control.analysis_advance.is_finite() || control.analysis_advance <= 0.0 {
            return Err(StateError::AnalysisAdvance);
        }

        let current_slot = usize::from(self.has_state && self.region_slot == 0);
        let current_base = current_slot * coefficients;
        build_regions(
            current,
            self.channels,
            atoms,
            self.owned_bins,
            &mut self.regions[current_base..current_base + coefficients],
        );
        let previous_base = self.region_slot * coefficients;
        let analysis_base = 0;
        let synthesis_base = coefficients;
        let mut report = StateTickReport::default();
        let mut first = 0;

        for scale in Scale::ALL {
            let scale_end = first + self.owned_bins[scale.index()];
            let mut region_first = first;
            while region_first < scale_end {
                let record = self.regions[current_base + region_first];
                let peak = record.peak;
                let mut region_end = region_first + 1;
                while region_end < scale_end && self.regions[current_base + region_end].peak == peak
                {
                    region_end += 1;
                }
                let frequency = self.atom_frequency(scale, peak - first);
                let owner_record = self.regions[current_base + record.owner * atoms + peak];
                let previous = self
                    .has_state
                    .then(|| self.regions[previous_base + record.owner * atoms + peak]);
                let compatible = previous.is_some_and(|prior| {
                    (region_first..region_end).contains(&prior.peak)
                        && owner_record.supported
                        && prior.supported
                });
                let borrowed = compatible && frequency < LINK_LIMIT_HZ;
                let owner_switch = previous.is_some_and(|prior| prior.owner != record.owner);
                let previous_joint_support = self.has_state
                    && (0..self.channels).any(|channel| {
                        self.regions[previous_base + channel * atoms + peak].supported
                    });
                let current_joint_support = (0..self.channels)
                    .any(|channel| self.regions[current_base + channel * atoms + peak].supported);

                let mut region_locked = false;
                for atom in region_first..region_end {
                    let material = guidance[atom];
                    let state = if control.ordinary_bypass {
                        TerminalState::Ordinary
                    } else if !self.has_state || !current_joint_support || !previous_joint_support {
                        TerminalState::Reset
                    } else if material.transientness > material.tonalness
                        && control.transient_center
                        && self.atom_frequency(scale, atom - first) < LINK_LIMIT_HZ
                    {
                        TerminalState::Attack
                    } else if material.noisiness > material.tonalness {
                        TerminalState::Unlocked
                    } else {
                        TerminalState::Locked
                    };
                    states[atom] = state;
                    report.states[state.index()] += 1;
                    region_locked |= state == TerminalState::Locked;

                    for channel in 0..self.channels {
                        let index = channel * atoms + atom;
                        let value = current[index];
                        let prior_supported = self.has_state
                            && self.regions[previous_base + channel * atoms + atom].supported;
                        let phase = match state {
                            TerminalState::Reset => {
                                if prior_supported {
                                    ordinary_phase(
                                        current,
                                        &self.phase,
                                        self.sample_rate,
                                        self.hop,
                                        atoms,
                                        channel,
                                        atom,
                                        self.atom_frequency(scale, atom - first),
                                        control.analysis_advance,
                                        true,
                                    )
                                } else {
                                    value.arg()
                                }
                            }
                            TerminalState::Attack => value.arg(),
                            TerminalState::Ordinary | TerminalState::Unlocked => ordinary_phase(
                                current,
                                &self.phase,
                                self.sample_rate,
                                self.hop,
                                atoms,
                                channel,
                                atom,
                                self.atom_frequency(scale, atom - first),
                                control.analysis_advance,
                                prior_supported,
                            ),
                            TerminalState::Locked => {
                                let trajectory_channel =
                                    if borrowed { record.owner } else { channel };
                                let trajectory_supported = self.has_state
                                    && self.regions
                                        [previous_base + trajectory_channel * atoms + peak]
                                        .supported;
                                let trajectory = ordinary_phase(
                                    current,
                                    &self.phase,
                                    self.sample_rate,
                                    self.hop,
                                    atoms,
                                    trajectory_channel,
                                    peak,
                                    frequency,
                                    control.analysis_advance,
                                    trajectory_supported,
                                );
                                trajectory
                                    + wrap(value.arg() - current[channel * atoms + peak].arg())
                            }
                        };
                        output[index] = if value.norm_sqr() == 0.0 {
                            Complex64::default()
                        } else {
                            Complex64::from_polar(value.norm(), phase)
                        };
                        report.non_finite_values += usize::from(
                            !output[index].re.is_finite() || !output[index].im.is_finite(),
                        );
                    }
                }
                if region_locked {
                    report.borrowed_regions += usize::from(borrowed);
                    report.local_regions += usize::from(!borrowed);
                    report.owner_switches += usize::from(owner_switch);
                }
                region_first = region_end;
            }
            first = scale_end;
        }

        for channel in 0..self.channels {
            for atom in 0..atoms {
                let index = channel * atoms + atom;
                self.phase[analysis_base + index] = current[index].arg();
                self.phase[synthesis_base + index] = output[index].arg();
            }
        }
        self.has_state = true;
        self.region_slot = current_slot;
        let mut hash = HASH_OFFSET;
        for value in output.iter() {
            hash_u64(&mut hash, value.re.to_bits());
            hash_u64(&mut hash, value.im.to_bits());
        }
        for state in states.iter() {
            hash_usize(&mut hash, state.index());
        }
        hash_workless_report(&mut hash, report);
        report.hash = hash;
        Ok(report)
    }

    fn atom_frequency(&self, scale: Scale, local: usize) -> f64 {
        let length = self.lengths[scale.index()];
        let bin = owned_start_bin(self.sample_rate, length, scale) + local;
        bin as f64 * self.sample_rate as f64 / length as f64
    }
}

fn hash_workless_report(hash: &mut u64, report: StateTickReport) {
    for value in report.states.into_iter().chain([
        report.borrowed_regions,
        report.local_regions,
        report.owner_switches,
        report.non_finite_values,
    ]) {
        hash_usize(hash, value);
    }
}

#[cfg(test)]
mod tests;
