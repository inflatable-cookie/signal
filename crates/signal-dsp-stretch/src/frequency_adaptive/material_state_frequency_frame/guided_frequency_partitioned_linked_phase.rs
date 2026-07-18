use rustfft::num_complex::Complex64;

use super::{
    absolute_bin, hash_u64, hash_usize,
    material_phase::{analyse_for_stage_a, synthesise_for_stage_a, Analysis},
    paired_max_error, reconstruct_channel, HASH_OFFSET, SAMPLE_RATE_HZ,
};

mod coefficients;
mod evidence;
mod forced;
mod kernel;
mod regions;
use coefficients::mirror_coefficients;
use evidence::{deterministic_probe, hash_channels, review_hash};
use forced::forced_render;
pub(super) use kernel::Workspace;
use regions::{peak_regions, validate_request};

const CHANNEL_CAPACITY: usize = 2;
const SIGNED_ATOM_CAPACITY: usize = 1_344;
const POSITIVE_ATOM_CAPACITY: usize = 673;
const COEFFICIENT_CAPACITY: usize = 32;
const REGION_CAPACITY: usize = 673;
const COMMON_HOP: usize = 512;
const LINK_LIMIT_HZ: f64 = 6_000.0;
pub(super) const ENERGY_FLOOR: f64 = 1.0e-24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CapacityExceeded {
    Channels,
    SignedAtoms,
    PositiveAtoms,
    Coefficients,
    Regions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Decision {
    Reset,
    Attack,
    Ordinary,
    Unlocked,
    Locked,
}

impl Decision {
    pub(super) fn index(self) -> usize {
        match self {
            Self::Reset => 0,
            Self::Attack => 1,
            Self::Ordinary => 2,
            Self::Unlocked => 3,
            Self::Locked => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct StateCounts {
    pub(super) states: [usize; 5],
    pub(super) linked_regions: usize,
    pub(super) unlinked_regions: usize,
    pub(super) owner_switches: usize,
}

#[derive(Clone, Copy, Debug)]
struct Region {
    first: usize,
    end: usize,
    peak: usize,
    owner: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct StageAReview {
    geometry: [usize; 6],
    capacities: [usize; 5],
    owner_counts: [usize; 3],
    structural_failures: [usize; 5],
    identity_errors: [f64; 4],
    mechanics_errors: [f64; 4],
    state_counts: [usize; 5],
    linked_regions: usize,
    unlinked_regions: usize,
    owner_switches: usize,
    region_high_water: usize,
    overflow_failures: usize,
    non_finite_values: usize,
    hashes: [u64; 3],
}

fn stage_a_review() -> StageAReview {
    let source = deterministic_probe();
    let second = source
        .iter()
        .enumerate()
        .map(|(index, sample)| sample * 0.47 + ((index * 31 % 509) as f64 - 254.0) / 8_192.0)
        .collect::<Vec<_>>();
    let silence = vec![0.0; source.len()];

    let identity_analysis = analyse_for_stage_a([&source, &source], SAMPLE_RATE_HZ);
    let representation = &identity_analysis.representation;
    let (identity, identity_non_finite) = synthesise_for_stage_a(
        representation,
        identity_analysis.coefficients.clone(),
        source.len(),
    );
    let mono = reconstruct_channel(&source, representation);
    let identity_errors = [
        paired_max_error(&source, &identity[0]),
        paired_max_error(&source, &identity[1]),
        paired_max_error(&identity[0], &identity[1]),
        paired_max_error(&source, &mono.samples),
    ];

    let (duplicate, duplicate_counts, duplicate_regions, duplicate_non_finite) =
        forced_render([&source, &source]);
    let (mono_pair, _, _, mono_non_finite) = forced_render([&source, &silence]);
    let (ordinary, _, _, ordinary_non_finite) = forced_render([&source, &second]);
    let (swapped, _, _, swapped_non_finite) = forced_render([&second, &source]);
    let mechanics_errors = [
        paired_max_error(&duplicate[0], &duplicate[1]),
        paired_max_error(&duplicate[0], &mono_pair[0]),
        mono_pair[1]
            .iter()
            .copied()
            .map(f64::abs)
            .fold(0.0, f64::max),
        paired_max_error(&ordinary[0], &swapped[1])
            .max(paired_max_error(&ordinary[1], &swapped[0])),
    ];

    let positive_atoms = representation
        .bands
        .iter()
        .filter(|band| band.center <= representation.fft_frames / 2)
        .count();
    let structural_failures = [
        representation.structural_failures.iter().sum(),
        usize::from(representation.bands.len() != SIGNED_ATOM_CAPACITY),
        usize::from(positive_atoms != POSITIVE_ATOM_CAPACITY),
        usize::from(representation.common_coefficients != COEFFICIENT_CAPACITY),
        usize::from(representation.owner_counts.contains(&0)),
    ];
    let overflow_failures = [
        (
            CHANNEL_CAPACITY + 1,
            SIGNED_ATOM_CAPACITY,
            POSITIVE_ATOM_CAPACITY,
            COEFFICIENT_CAPACITY,
            CapacityExceeded::Channels,
        ),
        (
            CHANNEL_CAPACITY,
            SIGNED_ATOM_CAPACITY + 1,
            POSITIVE_ATOM_CAPACITY,
            COEFFICIENT_CAPACITY,
            CapacityExceeded::SignedAtoms,
        ),
        (
            CHANNEL_CAPACITY,
            SIGNED_ATOM_CAPACITY,
            POSITIVE_ATOM_CAPACITY + 1,
            COEFFICIENT_CAPACITY,
            CapacityExceeded::PositiveAtoms,
        ),
        (
            CHANNEL_CAPACITY,
            SIGNED_ATOM_CAPACITY,
            POSITIVE_ATOM_CAPACITY,
            COEFFICIENT_CAPACITY + 1,
            CapacityExceeded::Coefficients,
        ),
    ]
    .into_iter()
    .filter(|(channels, signed, positive, coefficients, expected)| {
        validate_request(*channels, *signed, *positive, *coefficients) != Err(*expected)
    })
    .count()
        + usize::from(!matches!(
            peak_regions(&vec![1.0; REGION_CAPACITY + 1]),
            Err(CapacityExceeded::Regions)
        ));

    let mut review = StageAReview {
        geometry: [
            representation.fft_frames,
            source.len(),
            COMMON_HOP,
            4_096,
            2_048,
            1_024,
        ],
        capacities: [
            CHANNEL_CAPACITY,
            SIGNED_ATOM_CAPACITY,
            POSITIVE_ATOM_CAPACITY,
            COEFFICIENT_CAPACITY,
            REGION_CAPACITY,
        ],
        owner_counts: representation.owner_counts,
        structural_failures,
        identity_errors,
        mechanics_errors,
        state_counts: duplicate_counts.states,
        linked_regions: duplicate_counts.linked_regions,
        unlinked_regions: duplicate_counts.unlinked_regions,
        owner_switches: duplicate_counts.owner_switches,
        region_high_water: duplicate_regions,
        overflow_failures,
        non_finite_values: identity_non_finite
            + duplicate_non_finite
            + mono_non_finite
            + ordinary_non_finite
            + swapped_non_finite
            + mono.non_finite_values,
        hashes: [hash_channels(&identity), hash_channels(&duplicate), 0],
    };
    review.hashes[2] = review_hash(&review);
    review
}

pub(super) fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

#[cfg(test)]
mod tests;
