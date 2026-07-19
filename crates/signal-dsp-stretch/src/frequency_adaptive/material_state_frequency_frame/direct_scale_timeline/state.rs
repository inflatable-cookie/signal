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
            for atom in first..scale_end {
                let previous_joint_support = self.has_state
                    && (0..self.channels).any(|channel| {
                        let peak = self.regions[current_base + channel * atoms + atom].peak;
                        self.regions[previous_base + channel * atoms + peak].supported
                    });
                let current_joint_support = (0..self.channels).any(|channel| {
                    let peak = self.regions[current_base + channel * atoms + atom].peak;
                    self.regions[current_base + channel * atoms + peak].supported
                });
                let material = guidance[atom];
                let atom_frequency = self.atom_frequency(scale, atom - first);
                let state = if control.ordinary_bypass {
                    TerminalState::Ordinary
                } else if !self.has_state || !current_joint_support || !previous_joint_support {
                    TerminalState::Reset
                } else if material.transientness > material.tonalness
                    && control.transient_center
                    && atom_frequency < LINK_LIMIT_HZ
                {
                    TerminalState::Attack
                } else if material.noisiness > material.tonalness {
                    TerminalState::Unlocked
                } else {
                    TerminalState::Locked
                };
                states[atom] = state;
                report.states[state.index()] += 1;
                report.channel_peak_disagreements += usize::from(
                    self.channels == 2
                        && self.regions[current_base + atom].peak
                            != self.regions[current_base + atoms + atom].peak,
                );

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
                                    atom_frequency,
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
                            atom_frequency,
                            control.analysis_advance,
                            prior_supported,
                        ),
                        TerminalState::Locked => {
                            let record = self.regions[current_base + index];
                            let peak = record.peak;
                            let candidate = dominant_channel(current, self.channels, atoms, atom);
                            let candidate_peak =
                                self.regions[current_base + candidate * atoms + atom].peak;
                            let requester_predecessor = self
                                .has_state
                                .then(|| self.regions[previous_base + channel * atoms + peak]);
                            let candidate_predecessor = self.has_state.then(|| {
                                self.regions[previous_base + candidate * atoms + candidate_peak]
                            });
                            let common_predecessor = requester_predecessor
                                .zip(candidate_predecessor)
                                .and_then(|(requester, candidate)| {
                                    (requester.peak == candidate.peak).then_some(requester.peak)
                                });
                            let peak_frequency = self.atom_frequency(scale, peak - first);
                            let borrowed = candidate != channel
                                && peak_frequency < LINK_LIMIT_HZ
                                && self.regions[current_base + candidate * atoms + atom].supported
                                && current[candidate * atoms + peak].norm_sqr() > SUPPORT_FLOOR
                                && common_predecessor.is_some_and(|predecessor| {
                                    self.regions[previous_base + candidate * atoms + predecessor]
                                        .supported
                                });
                            let trajectory_channel = if borrowed { candidate } else { channel };
                            let predecessor = if borrowed {
                                common_predecessor
                            } else {
                                requester_predecessor.map(|record| record.peak)
                            };
                            let trajectory = predecessor_anchored_phase(
                                current,
                                &self.phase,
                                &self.regions,
                                self.sample_rate,
                                self.hop,
                                atoms,
                                previous_base,
                                trajectory_channel,
                                peak,
                                predecessor,
                                peak_frequency,
                                control.analysis_advance,
                            );
                            report.borrowed_locked_atoms += usize::from(borrowed);
                            report.local_locked_atoms += usize::from(!borrowed);
                            report.trajectory_channel_switches += usize::from(
                                self.has_state
                                    && self.regions[previous_base + index].trajectory_channel
                                        != trajectory_channel,
                            );
                            self.regions[current_base + index].trajectory_channel =
                                trajectory_channel;
                            trajectory
                                + wrap(
                                    value.arg() - current[trajectory_channel * atoms + peak].arg(),
                                )
                        }
                    };
                    output[index] = if value.norm_sqr() == 0.0 {
                        Complex64::default()
                    } else {
                        Complex64::from_polar(value.norm(), phase)
                    };
                    report.non_finite_values +=
                        usize::from(!output[index].re.is_finite() || !output[index].im.is_finite());
                }
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
        report.borrowed_locked_atoms,
        report.local_locked_atoms,
        report.trajectory_channel_switches,
        report.channel_peak_disagreements,
        report.non_finite_values,
    ]) {
        hash_usize(hash, value);
    }
}

#[allow(clippy::too_many_arguments)]
fn predecessor_anchored_phase(
    current: &[Complex64],
    phase: &[f64],
    regions: &[RegionRecord],
    sample_rate: usize,
    hop: usize,
    atoms: usize,
    previous_base: usize,
    channel: usize,
    peak: usize,
    predecessor: Option<usize>,
    frequency: f64,
    analysis_advance: f64,
) -> f64 {
    let coefficients = phase.len() / 2;
    let index = channel * atoms + peak;
    let prior_supported = regions[previous_base + index].supported;
    let ordinary = ordinary_phase(
        current,
        phase,
        sample_rate,
        hop,
        atoms,
        channel,
        peak,
        frequency,
        analysis_advance,
        prior_supported,
    );
    predecessor
        .filter(|predecessor| {
            prior_supported && regions[previous_base + channel * atoms + predecessor].supported
        })
        .map_or(ordinary, |predecessor| {
            let advance = wrap(ordinary - phase[coefficients + index]);
            phase[coefficients + channel * atoms + predecessor] + advance
        })
}

#[cfg(test)]
mod tests;
