use rustfft::num_complex::Complex64;

use super::*;
use crate::frequency_adaptive::material_state_frequency_frame::guided_frequency_partitioned_linked_phase::{
    wrap, CapacityExceeded, ENERGY_FLOOR,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BoundaryError {
    OutputLayers,
    State(CapacityExceeded),
}

pub(super) struct Projected {
    pub(super) decision: Frame,
    pub(super) layers: Vec<Frame>,
}

pub(super) fn project_layers(
    workspace: &mut Workspace,
    current: &Frame,
    layers: &[Frame],
    frequencies_hz: &[f64],
    decision: Decision,
) -> Result<Projected, BoundaryError> {
    if layers.len() > OUTPUT_SLICE_CAPACITY {
        return Err(BoundaryError::OutputLayers);
    }
    let decided = workspace
        .process(current, frequencies_hz, decision)
        .map_err(BoundaryError::State)?;
    let layers = layers
        .iter()
        .map(|layer| {
            std::array::from_fn(|channel| {
                layer[channel]
                    .iter()
                    .zip(&current[channel])
                    .zip(&decided[channel])
                    .map(|((local, analysis), shared)| {
                        if local.norm_sqr() <= ENERGY_FLOOR {
                            Complex64::default()
                        } else {
                            Complex64::from_polar(
                                local.norm(),
                                shared.arg() + wrap(local.arg() - analysis.arg()),
                            )
                        }
                    })
                    .collect()
            })
        })
        .collect();
    Ok(Projected {
        decision: decided,
        layers,
    })
}

pub(super) fn projection_errors(
    current: &Frame,
    layers: &[Frame],
    projected: &Projected,
    errors: &mut [f64; 6],
) {
    for (input, output) in layers.iter().zip(&projected.layers) {
        for channel in 0..CHANNEL_CAPACITY {
            for band in 0..input[channel].len() {
                errors[4] = errors[4]
                    .max((input[channel][band].norm() - output[channel][band].norm()).abs());
                if input[channel][band].norm_sqr() > ENERGY_FLOOR {
                    let expected = wrap(input[channel][band].arg() - current[channel][band].arg());
                    let actual =
                        wrap(output[channel][band].arg() - projected.decision[channel][band].arg());
                    errors[5] = errors[5].max(wrap(actual - expected).abs());
                }
            }
        }
    }
}

pub(super) fn frame_error(left: &[Complex64], right: &[Complex64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (*left - *right).norm())
        .fold(0.0_f64, f64::max)
}

pub(super) fn maximum_norm(frame: &[Complex64]) -> f64 {
    frame
        .iter()
        .map(|value| value.norm())
        .fold(0.0_f64, f64::max)
}

pub(super) fn non_finite_values(projected: &Projected) -> usize {
    projected
        .decision
        .iter()
        .chain(projected.layers.iter().flat_map(|layer| layer.iter()))
        .flat_map(|channel| channel.iter())
        .map(|value| usize::from(!value.re.is_finite()) + usize::from(!value.im.is_finite()))
        .sum()
}

pub(super) fn hash_projected(hash: &mut u64, projected: &Projected) {
    for value in projected
        .decision
        .iter()
        .chain(projected.layers.iter().flat_map(|layer| layer.iter()))
        .flat_map(|channel| channel.iter())
    {
        hash_u64(hash, value.re.to_bits());
        hash_u64(hash, value.im.to_bits());
    }
}
