use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn ordinary_phase(
    current: &[Complex64],
    phase: &[f64],
    sample_rate: usize,
    hop: usize,
    atoms: usize,
    channel: usize,
    atom: usize,
    frequency: f64,
    analysis_advance: f64,
    prior_supported: bool,
) -> f64 {
    let coefficients = phase.len() / 2;
    let index = channel * atoms + atom;
    let analysis = current[index].arg();
    if current[index].norm_sqr() <= SUPPORT_FLOOR || !prior_supported {
        return analysis;
    }
    let expected = std::f64::consts::TAU * frequency / sample_rate as f64 * analysis_advance;
    let observed = expected + wrap(analysis - phase[index] - expected);
    phase[coefficients + index] + observed * hop as f64 / analysis_advance
}

pub(super) fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
