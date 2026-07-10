use signal_primitives::{Sample, SampleRate};

mod stft;
#[cfg(test)]
mod tests;
mod types;

use stft::{binary_median_stage, stage_config};
pub use types::{
    StretchHprComponentEvidence, StretchHprSeparationEvidence, StretchHprSeparationReview,
};

const LONG_WINDOW_SECONDS: f64 = 0.186;
const SHORT_WINDOW_SECONDS: f64 = 0.0116;
const HARMONIC_FACTOR: f32 = 2.0;
const PERCUSSIVE_FACTOR: f32 = 2.0;

pub(crate) fn separate_hpr_review_mono(
    input: &[Sample],
    sample_rate: SampleRate,
) -> StretchHprSeparationReview {
    let sample_rate_hz = sample_rate.0.max(1) as f64;
    let long = stage_config(sample_rate_hz, LONG_WINDOW_SECONDS);
    let short = stage_config(sample_rate_hz, SHORT_WINDOW_SECONDS);
    let harmonic_stage = binary_median_stage(input, &long, |horizontal, vertical| {
        horizontal > HARMONIC_FACTOR * vertical
    });
    let percussive_stage = binary_median_stage(
        &harmonic_stage.complement,
        &short,
        |horizontal, vertical| vertical > PERCUSSIVE_FACTOR * horizontal,
    );
    let harmonic = harmonic_stage.selected;
    let residual = percussive_stage.complement;
    let percussive = percussive_stage.selected;

    let reconstruction = reconstruction_error(input, &harmonic, &residual, &percussive);
    let energies = [energy(&harmonic), energy(&residual), energy(&percussive)];
    let total_energy = energies.iter().sum::<f64>();
    let component_evidence = |samples: &[Sample], component_index: usize| {
        let component_energy = energies[component_index];
        let strongest_other = energies
            .iter()
            .enumerate()
            .filter_map(|(index, energy)| (index != component_index).then_some(*energy))
            .fold(0.0_f64, f64::max);
        StretchHprComponentEvidence {
            energy: component_energy,
            energy_share: if total_energy > 0.0 {
                component_energy / total_energy
            } else {
                0.0
            },
            dominance_margin_db: energy_margin_db(component_energy, strongest_other),
            sample_hash: sample_hash(samples),
            all_samples_finite: samples.iter().all(|sample| sample.is_finite()),
        }
    };
    let evidence = StretchHprSeparationEvidence {
        long_window_frames: long.window_size,
        short_window_frames: short.window_size,
        long_hop_frames: long.hop_size,
        short_hop_frames: short.hop_size,
        long_horizontal_median_frames: long.horizontal_span,
        long_vertical_median_bins: long.vertical_span,
        short_horizontal_median_frames: short.horizontal_span,
        short_vertical_median_bins: short.vertical_span,
        harmonic_mask_bins: harmonic_stage.selected_bins,
        long_complement_mask_bins: harmonic_stage.complement_bins,
        percussive_mask_bins: percussive_stage.selected_bins,
        residual_mask_bins: percussive_stage.complement_bins,
        masks_partition_exactly: harmonic_stage.partition_exact && percussive_stage.partition_exact,
        mask_partition_error_bins: usize::from(!harmonic_stage.partition_exact)
            + usize::from(!percussive_stage.partition_exact),
        long_uncovered_source_samples: harmonic_stage.uncovered_samples,
        short_uncovered_source_samples: percussive_stage.uncovered_samples,
        reconstruction_peak_error: reconstruction.peak,
        reconstruction_rms_error: reconstruction.rms,
        reconstruction_head_error: reconstruction.head,
        reconstruction_tail_error: reconstruction.tail,
        harmonic: component_evidence(&harmonic, 0),
        residual: component_evidence(&residual, 1),
        percussive: component_evidence(&percussive, 2),
    };
    StretchHprSeparationReview {
        harmonic,
        residual,
        percussive,
        evidence,
    }
}

struct ReconstructionError {
    peak: f64,
    rms: f64,
    head: f64,
    tail: f64,
}

fn reconstruction_error(
    input: &[Sample],
    harmonic: &[Sample],
    residual: &[Sample],
    percussive: &[Sample],
) -> ReconstructionError {
    if input.is_empty() {
        return ReconstructionError {
            peak: 0.0,
            rms: 0.0,
            head: 0.0,
            tail: 0.0,
        };
    }
    let mut peak = 0.0_f64;
    let mut squared = 0.0_f64;
    for index in 0..input.len() {
        let reconstructed = harmonic[index] + residual[index] + percussive[index];
        let error = f64::from(input[index] - reconstructed);
        peak = peak.max(error.abs());
        squared += error * error;
    }
    let sample_error = |index: usize| {
        let reconstructed = harmonic[index] + residual[index] + percussive[index];
        f64::from(input[index] - reconstructed).abs()
    };
    ReconstructionError {
        peak,
        rms: (squared / input.len() as f64).sqrt(),
        head: sample_error(0),
        tail: sample_error(input.len() - 1),
    }
}

fn energy_margin_db(owner: f64, strongest_other: f64) -> f64 {
    10.0 * (owner / strongest_other.max(f64::MIN_POSITIVE)).log10()
}

fn energy(samples: &[Sample]) -> f64 {
    samples
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum()
}

fn sample_hash(samples: &[Sample]) -> u64 {
    samples.iter().fold(0xcbf2_9ce4_8422_2325, |hash, sample| {
        sample
            .to_bits()
            .to_le_bytes()
            .iter()
            .fold(hash, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
            })
    })
}
