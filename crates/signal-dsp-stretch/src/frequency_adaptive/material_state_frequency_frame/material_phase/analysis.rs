use super::*;

pub(super) fn analyse(inputs: [&[f64]; 2], sample_rate: usize) -> Analysis {
    let fft_frames = padded_frames(inputs[0].len());
    let representation = build_representation_for(fft_frames, sample_rate, COMMON_HOP);
    let mut planner = FftPlanner::<f64>::new();
    let forward = planner.plan_fft_forward(fft_frames);
    let inverse_band = planner.plan_fft_inverse(representation.common_coefficients);
    let spectra: [Vec<Complex64>; 2] = std::array::from_fn(|channel| {
        let mut spectrum = (0..fft_frames)
            .map(|index| {
                Complex64::new(
                    reflected_sample(inputs[channel], index as isize - PAD_FRAMES as isize),
                    0.0,
                )
            })
            .collect::<Vec<_>>();
        forward.process(&mut spectrum);
        spectrum
    });
    let coefficients = std::array::from_fn(|channel| {
        representation
            .bands
            .iter()
            .map(|band| {
                let mut values = vec![Complex64::default(); representation.common_coefficients];
                for &(bin, weight) in &band.taps {
                    let local = local_coefficient(
                        bin,
                        band.center,
                        representation.common_coefficients,
                        fft_frames,
                    );
                    values[local] = spectra[channel][bin] * weight;
                }
                inverse_band.process(&mut values);
                let scale = 1.0 / representation.common_coefficients as f64;
                values.iter_mut().for_each(|value| *value *= scale);
                values
            })
            .collect::<Vec<_>>()
    });
    let magnitudes = linked_magnitudes(&coefficients);
    let material = material_map(&representation.bands, &magnitudes);
    let transient_centers = transient_centers(&material, &magnitudes);
    Analysis {
        representation,
        coefficients,
        material,
        transient_centers,
    }
}

fn linked_magnitudes(coefficients: &[Vec<Vec<Complex64>>; 2]) -> Vec<Vec<f64>> {
    coefficients[0]
        .iter()
        .zip(&coefficients[1])
        .map(|(left, right)| {
            left.iter()
                .zip(right)
                .map(|(left, right)| left.norm().max(right.norm()))
                .collect()
        })
        .collect()
}

fn material_map(bands: &[Band], magnitudes: &[Vec<f64>]) -> Vec<Vec<Material>> {
    let scale_bands = std::array::from_fn::<_, 3, _>(|scale| {
        bands
            .iter()
            .enumerate()
            .filter(|(_, band)| band.scale.index() == scale)
            .map(|(index, _)| index)
            .collect::<Vec<_>>()
    });
    bands
        .iter()
        .enumerate()
        .map(|(band, descriptor)| {
            let same_scale = &scale_bands[descriptor.scale.index()];
            let position = same_scale
                .iter()
                .position(|index| *index == band)
                .expect("owned band");
            let first = position.saturating_sub(1);
            let end = (position + 1).min(same_scale.len() - 1);
            (0..magnitudes[band].len())
                .map(|time| {
                    let half_time =
                        (SUPPORT_FRAMES[descriptor.scale.index()] / (2 * COMMON_HOP)).max(1);
                    let tonal = median(
                        (time.saturating_sub(half_time)
                            ..=(time + half_time).min(magnitudes[band].len() - 1))
                            .map(|index| magnitudes[band][index]),
                    );
                    let transient =
                        median((first..=end).map(|index| magnitudes[same_scale[index]][time]));
                    let sum = tonal + transient;
                    if sum == 0.0 {
                        Material::default()
                    } else {
                        let tonalness = tonal / sum;
                        let transientness = transient / sum;
                        Material {
                            tonalness,
                            noisiness: 1.0 - (tonalness - transientness).abs(),
                            transientness,
                        }
                    }
                })
                .collect()
        })
        .collect()
}

fn transient_centers(material: &[Vec<Material>], magnitudes: &[Vec<f64>]) -> Vec<bool> {
    let frames = material.first().map_or(0, Vec::len);
    let mut transientness = vec![0.0; frames];
    for time in 0..frames {
        let mut weighted = 0.0;
        let mut weight = 0.0;
        for band in 0..material.len() {
            weighted += material[band][time].transientness * magnitudes[band][time];
            weight += magnitudes[band][time];
        }
        if weight > 0.0 {
            transientness[time] = weighted / weight;
        }
    }
    let mut centers = vec![false; frames];
    for time in 1..frames.saturating_sub(1) {
        centers[time] = transientness[time] > transientness[time - 1]
            && transientness[time] >= transientness[time + 1];
    }
    centers
}

pub(super) fn material_sample(values: &[Material], position: f64) -> Material {
    let first = position.floor().clamp(0.0, values.len() as f64 - 1.0) as usize;
    let second = (first + 1).min(values.len() - 1);
    let fraction = (position - first as f64).clamp(0.0, 1.0);
    let lerp = |left: f64, right: f64| left + (right - left) * fraction;
    Material {
        tonalness: lerp(values[first].tonalness, values[second].tonalness),
        noisiness: lerp(values[first].noisiness, values[second].noisiness),
        transientness: lerp(values[first].transientness, values[second].transientness),
    }
}

pub(super) fn polar_sample(values: &[Complex64], position: f64) -> Complex64 {
    let first = position.floor().clamp(0.0, values.len() as f64 - 1.0) as usize;
    let second = (first + 1).min(values.len() - 1);
    let fraction = (position - first as f64).clamp(0.0, 1.0);
    let magnitude =
        values[first].norm() + (values[second].norm() - values[first].norm()) * fraction;
    let phase = values[first].arg() + wrap(values[second].arg() - values[first].arg()) * fraction;
    Complex64::from_polar(magnitude, phase)
}

fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut values = values.collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}
