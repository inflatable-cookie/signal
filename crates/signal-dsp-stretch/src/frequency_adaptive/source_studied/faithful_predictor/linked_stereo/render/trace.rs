use rustfft::num_complex::Complex64;

use super::super::super::{constrain_real_edges, TransformGrid};

pub(super) fn relation_errors(
    current: &[Vec<Complex64>; 2],
    projected: &[Vec<Complex64>; 2],
    significant_energy: f64,
    grid: TransformGrid,
) -> [f64; 2] {
    let projected_error = maximum_relation_error(current, projected, significant_energy);
    let mut constrained = projected.clone();
    for channel in &mut constrained {
        constrain_real_edges(channel, grid);
    }
    [
        projected_error,
        maximum_relation_error(current, &constrained, significant_energy),
    ]
}

fn maximum_relation_error(
    current: &[Vec<Complex64>; 2],
    output: &[Vec<Complex64>; 2],
    significant_energy: f64,
) -> f64 {
    (0..current[0].len())
        .filter(|bin| {
            current[0][*bin].norm_sqr() > significant_energy
                && current[1][*bin].norm_sqr() > significant_energy
        })
        .map(|bin| {
            let input_relation = current[1][bin] * current[0][bin].conj();
            let output_relation = output[1][bin] * output[0][bin].conj();
            wrap(output_relation.arg() - input_relation.arg()).abs()
        })
        .fold(0.0, f64::max)
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
