use super::analysis::{material_sample, polar_sample};
use super::*;

pub(super) fn transport(
    analysis: &Analysis,
    output: &Representation,
    ratio: f64,
    target_length: usize,
) -> [Vec<Vec<Complex64>>; 2] {
    let positive = analysis
        .representation
        .bands
        .iter()
        .enumerate()
        .filter(|(_, band)| band.center <= analysis.representation.fft_frames / 2)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let mut result: [Vec<Vec<Complex64>>; 2] = std::array::from_fn(|_| {
        vec![vec![Complex64::default(); output.common_coefficients]; output.bands.len()]
    });
    let mut previous_regions = Vec::<RegionMemory>::new();
    let mut previous_source = None;
    let mut states = StateCounts::default();

    for time in 0..output.common_coefficients {
        let logical = time as isize * COMMON_HOP as isize - PAD_FRAMES as isize;
        let reflected = reflect_index(logical, target_length) as f64;
        let source_position = reflected / ratio;
        let source_time = (source_position + PAD_FRAMES as f64) / COMMON_HOP as f64;
        let current: [Vec<Complex64>; 2] = std::array::from_fn(|channel| {
            positive
                .iter()
                .map(|band| polar_sample(&analysis.coefficients[channel][*band], source_time))
                .collect()
        });
        let energy = (0..positive.len())
            .map(|band| current[0][band].norm_sqr().max(current[1][band].norm_sqr()))
            .collect::<Vec<_>>();
        if energy.iter().all(|value| *value == 0.0) {
            states.silent += positive.len();
            previous_regions.clear();
            previous_source = Some(source_position);
            continue;
        }

        let continuous = previous_source.is_some_and(|prior| source_position > prior);
        let analysis_delta = previous_source.map_or(0.0, |prior| source_position - prior);
        let mut next_regions = Vec::new();
        let frame_regions = regions(&energy);
        states.regions += frame_regions.len();
        for region in frame_regions {
            let peak_band = positive[region.peak];
            let owner = usize::from(
                current[1][region.peak].norm_sqr() > current[0][region.peak].norm_sqr(),
            );
            let phase = [current[0][region.peak].arg(), current[1][region.peak].arg()];
            let owner_energy = [
                current[0][region.peak].norm_sqr(),
                current[1][region.peak].norm_sqr(),
            ];
            let frequency = std::f64::consts::TAU
                * analysis.representation.bands[peak_band].center as f64
                / analysis.representation.fft_frames as f64;
            let predecessor = continuous
                .then(|| {
                    previous_regions
                        .iter()
                        .find(|prior| (prior.first..prior.end).contains(&region.peak))
                })
                .flatten();
            let rotation = predecessor
                .filter(|prior| analysis_delta > 0.0 && prior.energy[owner] > 0.0)
                .map(|prior| {
                    tracked_rotation(prior, owner, phase[owner], frequency, analysis_delta)
                })
                .unwrap_or(0.0);
            if predecessor.is_some_and(|prior| analysis_delta > 0.0 && prior.energy[owner] > 0.0) {
                states.tracked += 1;
                states.owner_switches +=
                    usize::from(predecessor.is_some_and(|prior| prior.owner != owner));
            } else {
                states.reset += 1;
            }

            for local in region.first..region.end {
                let band = positive[local];
                let material = material_sample(analysis.material[band].as_slice(), source_time);
                let (gain, phase_delta, state) = material_operator(
                    material,
                    &analysis.transient_centers,
                    source_time,
                    analysis.representation.bands[band].scale,
                    ratio,
                    time,
                    band,
                    rotation,
                );
                match state {
                    MaterialState::Shoulder => states.shoulder += 1,
                    MaterialState::Reset => states.reset += 1,
                    MaterialState::Locked => states.locked += 1,
                    MaterialState::Diffuse => states.diffuse += 1,
                }
                let operator = Complex64::from_polar(gain, phase_delta);
                for channel in 0..2 {
                    result[channel][band][time] = current[channel][local] * operator;
                }
            }
            next_regions.push(RegionMemory {
                first: region.first,
                end: region.end,
                owner,
                rotation,
                phase,
                energy: owner_energy,
                frequency,
            });
        }
        previous_regions = next_regions;
        previous_source = Some(source_position);
    }

    mirror_coefficients(output, &positive, &mut result);
    STATE_COUNTS.with(|slot| *slot.borrow_mut() = states);
    result
}

#[derive(Clone, Copy)]
enum MaterialState {
    Shoulder,
    Reset,
    Locked,
    Diffuse,
}

fn material_operator(
    material: Material,
    centers: &[bool],
    source_time: f64,
    scale: Scale,
    ratio: f64,
    output_time: usize,
    band: usize,
    rotation: f64,
) -> (f64, f64, MaterialState) {
    let time = source_time.round().clamp(0.0, centers.len() as f64 - 1.0) as usize;
    let radius = (SUPPORT_FRAMES[scale.index()] / (2 * COMMON_HOP)).max(1);
    let transient_owned = material.transientness > material.tonalness;
    if transient_owned && centers[time] {
        return (1.0, 0.0, MaterialState::Reset);
    }
    let near_center = transient_owned
        && (time.saturating_sub(radius)..=(time + radius).min(centers.len() - 1))
            .any(|index| centers[index]);
    if near_center {
        return (
            1.0 - material.transientness,
            rotation,
            MaterialState::Shoulder,
        );
    }
    let distance = (ratio - 1.0).abs().min(1.0);
    let amplitude = std::f64::consts::FRAC_PI_2 * material.noisiness * distance;
    if amplitude == 0.0 {
        (1.0, rotation, MaterialState::Locked)
    } else {
        let perturbation = amplitude * deterministic_unit(output_time, scale, band);
        (1.0, wrap(rotation + perturbation), MaterialState::Diffuse)
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

fn mirror_coefficients(
    representation: &Representation,
    positive: &[usize],
    coefficients: &mut [Vec<Vec<Complex64>>; 2],
) {
    for &band in positive {
        let center = representation.bands[band].center;
        if center == 0 || center == representation.fft_frames / 2 {
            for channel in coefficients.iter_mut() {
                for value in &mut channel[band] {
                    value.im = 0.0;
                }
            }
            continue;
        }
        let mirror_center = representation.fft_frames - center;
        let mirror = representation
            .bands
            .binary_search_by_key(&mirror_center, |candidate| candidate.center)
            .expect("conjugate band");
        for channel in coefficients.iter_mut() {
            let mirrored = channel[band]
                .iter()
                .map(Complex64::conj)
                .collect::<Vec<_>>();
            channel[mirror] = mirrored;
        }
    }
}

fn reflect_index(index: isize, length: usize) -> usize {
    let period = (length * 2) as isize;
    let wrapped = index.rem_euclid(period) as usize;
    if wrapped < length {
        wrapped
    } else {
        length * 2 - 1 - wrapped
    }
}

thread_local! {
    static STATE_COUNTS: std::cell::RefCell<StateCounts> = const { std::cell::RefCell::new(StateCounts {
        tracked: 0, reset: 0, silent: 0, regions: 0, owner_switches: 0,
        shoulder: 0, locked: 0, diffuse: 0,
    }) };
}

pub(super) fn analysis_state_counts(_analysis: &Analysis) -> StateCounts {
    STATE_COUNTS.with(|slot| *slot.borrow())
}
