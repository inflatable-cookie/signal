use rustfft::num_complex::Complex64;

use super::common_grid::{
    build_boundary_candidate_filters, build_preconditioned_boundary_filters, filter_hash, hash_u64,
    CHANNELS, HASH_OFFSET, HOP, LOWPASS_CHANNELS,
};
use super::hermitian_jacobi::jacobi_solution;
use super::types::{
    StretchCommonGridConditioningBank as Bank,
    StretchCommonGridConditioningBinEvidence as BinEvidence,
    StretchCommonGridConditioningChannelEvidence as ChannelEvidence,
    StretchCommonGridConditioningDirection as Direction,
    StretchCommonGridConditioningModeEvidence as ModeEvidence,
    StretchCommonGridConditioningResidueEvidence as ResidueEvidence,
    StretchCommonGridConditioningReview as Review,
};

const FFT_FRAMES: usize = 4_224;
const TOP: usize = 16;

pub(crate) fn common_grid_conditioning_attribution_review() -> Review {
    let positive_bins = FFT_FRAMES / 2 + 1;
    let raw = build_boundary_candidate_filters(FFT_FRAMES);
    let raw_hash = filter_hash(&raw);
    let (exact, exact_hash) = normalize_exact(raw.clone(), positive_bins);
    let (endpoint, endpoint_raw_hash, endpoint_hash) =
        build_preconditioned_boundary_filters(FFT_FRAMES);
    debug_assert_eq!(raw_hash, endpoint_raw_hash);
    let banks = [raw, exact, endpoint];
    let names = [Bank::Raw, Bank::ExactPointwise, Bank::EndpointEven];
    let mut residues = Vec::with_capacity(3 * FFT_FRAMES / HOP);
    let mut modes = Vec::with_capacity(6);
    let mut maximum_residual = 0.0_f64;

    for (bank_index, filters) in banks.iter().enumerate() {
        let mut extrema: [Option<(usize, f64, Vec<Complex64>, Vec<usize>)>; 2] = [None, None];
        for residue in 0..FFT_FRAMES / HOP {
            let bins = residue_bins(residue, positive_bins);
            let matrix = frame_matrix(filters, &bins, positive_bins);
            let solution = jacobi_solution(&matrix);
            let minimum = solution.eigenvalues[0];
            let maximum = solution.eigenvalues[bins.len() - 1];
            let min_vector = eigenvector_column(&solution.eigenvectors, bins.len(), 0);
            let max_vector = eigenvector_column(&solution.eigenvectors, bins.len(), bins.len() - 1);
            let min_residual = solution.evidence.proof_errors[0];
            let max_residual = solution.evidence.proof_errors[0];
            maximum_residual = maximum_residual.max(min_residual).max(max_residual);
            update_extreme(
                &mut extrema[0],
                residue,
                minimum,
                min_vector.clone(),
                bins.clone(),
                false,
            );
            update_extreme(
                &mut extrema[1],
                residue,
                maximum,
                max_vector.clone(),
                bins.clone(),
                true,
            );
            residues.push(ResidueEvidence {
                bank: names[bank_index],
                residue,
                bin_count: bins.len(),
                eigenvalues: [minimum, maximum],
                residuals: [min_residual, max_residual],
                hashes: [
                    hash_usizes(&bins),
                    hash_complex(&matrix),
                    hash_complex(&min_vector),
                    hash_complex(&max_vector),
                ],
            });
        }
        for (maximum, extreme) in extrema.into_iter().enumerate() {
            let (residue, eigenvalue, vector, bins) = extreme.expect("residue extrema");
            modes.push(attribute_mode(
                names[bank_index],
                maximum == 1,
                residue,
                eigenvalue,
                &vector,
                &bins,
                &banks,
                positive_bins,
            ));
        }
    }
    let maximum_closure = modes
        .iter()
        .map(|mode| mode.contribution_sums[3])
        .fold(0.0, f64::max);
    let exact_rows = &residues[11..22];
    let exact_condition = exact_rows
        .iter()
        .map(|row| row.eigenvalues[1])
        .fold(0.0, f64::max)
        / exact_rows
            .iter()
            .map(|row| row.eigenvalues[0])
            .fold(f64::INFINITY, f64::min);
    let endpoint_modes = &modes[4..6];
    let boundary_local = endpoint_modes
        .iter()
        .all(|mode| mode.region_mass[0] + mode.region_mass[2] >= 0.9);
    let direction = if maximum_residual > 1.0e-6 || maximum_closure > 1.0e-8 {
        Direction::Inconclusive
    } else if exact_condition > 1.25 || !boundary_local {
        Direction::BoundaryGeometry
    } else {
        Direction::BlockAwareBoundary
    };
    let mut review = Review {
        residues,
        modes,
        hashes: [raw_hash, exact_hash, endpoint_hash, 0],
        maximum_errors: [maximum_residual, maximum_closure],
        direction,
    };
    review.hashes[3] = evidence_hash(&review);
    review
}

fn normalize_exact(mut filters: Vec<Complex64>, positive_bins: usize) -> (Vec<Complex64>, u64) {
    let mut hash = HASH_OFFSET;
    for bin in 0..positive_bins {
        let energy = (0..CHANNELS)
            .map(|channel| filters[channel * positive_bins + bin].norm_sqr())
            .sum::<f64>();
        let multiplier = energy.sqrt().recip();
        hash_u64(&mut hash, multiplier.to_bits());
        for channel in 0..CHANNELS {
            filters[channel * positive_bins + bin] *= multiplier;
        }
    }
    (filters, hash)
}

fn residue_bins(residue: usize, positive_bins: usize) -> Vec<usize> {
    (residue..positive_bins).step_by(FFT_FRAMES / HOP).collect()
}

fn frame_matrix(filters: &[Complex64], bins: &[usize], positive_bins: usize) -> Vec<Complex64> {
    let size = bins.len();
    let mut matrix = vec![Complex64::new(0.0, 0.0); size * size];
    for channel in 0..CHANNELS {
        for (row, bin) in bins.iter().enumerate() {
            let value = filters[channel * positive_bins + *bin];
            if value.norm_sqr() == 0.0 {
                continue;
            }
            for (column, other) in bins.iter().enumerate() {
                matrix[row * size + column] +=
                    value * filters[channel * positive_bins + *other].conj();
            }
        }
    }
    matrix
}

pub(super) fn conditioning_matrices() -> Vec<Vec<Complex64>> {
    let positive_bins = FFT_FRAMES / 2 + 1;
    let raw = build_boundary_candidate_filters(FFT_FRAMES);
    let exact = normalize_exact(raw.clone(), positive_bins).0;
    let endpoint = build_preconditioned_boundary_filters(FFT_FRAMES).0;
    [raw, exact, endpoint]
        .iter()
        .flat_map(|filters| {
            (0..FFT_FRAMES / HOP).map(|residue| {
                let bins = residue_bins(residue, positive_bins);
                frame_matrix(filters, &bins, positive_bins)
            })
        })
        .collect()
}

fn eigenvector_column(vectors: &[Complex64], size: usize, column: usize) -> Vec<Complex64> {
    (0..size).map(|row| vectors[row * size + column]).collect()
}

fn update_extreme(
    slot: &mut Option<(usize, f64, Vec<Complex64>, Vec<usize>)>,
    residue: usize,
    value: f64,
    vector: Vec<Complex64>,
    bins: Vec<usize>,
    maximum: bool,
) {
    let replace = slot.as_ref().is_none_or(|old| {
        if maximum {
            value > old.1
        } else {
            value < old.1
        }
    });
    if replace {
        *slot = Some((residue, value, vector, bins));
    }
}

fn attribute_mode(
    bank: Bank,
    maximum: bool,
    residue: usize,
    eigenvalue: f64,
    vector: &[Complex64],
    bins: &[usize],
    banks: &[Vec<Complex64>; 3],
    positive_bins: usize,
) -> ModeEvidence {
    let matrices = banks
        .each_ref()
        .map(|filters| frame_matrix(filters, bins, positive_bins));
    let rayleigh = matrices
        .each_ref()
        .map(|matrix| dot(vector, &multiply(matrix, vector, bins.len())).re);
    let width = LOWPASS_CHANNELS as f64 * 0.5 / (CHANNELS - 1) as f64;
    let mut region = [0.0; 3];
    let mut bin_rows = bins
        .iter()
        .zip(vector)
        .map(|(bin, value)| {
            let f = *bin as f64 / FFT_FRAMES as f64;
            region[if f < width {
                0
            } else if f > 0.5 - width {
                2
            } else {
                1
            }] += value.norm_sqr();
            BinEvidence {
                bin: *bin,
                weight: value.norm_sqr(),
            }
        })
        .collect::<Vec<_>>();
    bin_rows.sort_by(|a, b| {
        b.weight
            .total_cmp(&a.weight)
            .then_with(|| a.bin.cmp(&b.bin))
    });
    bin_rows.truncate(TOP);
    let filters = &banks[match bank {
        Bank::Raw => 0,
        Bank::ExactPointwise => 1,
        Bank::EndpointEven => 2,
    }];
    let mut channels = (0..CHANNELS)
        .map(|channel| {
            let mut projection = Complex64::new(0.0, 0.0);
            let mut diagonal = 0.0;
            for (index, bin) in bins.iter().enumerate() {
                let h = filters[channel * positive_bins + *bin];
                projection += h.conj() * vector[index];
                diagonal += h.norm_sqr() * vector[index].norm_sqr();
            }
            let total = projection.norm_sqr();
            ChannelEvidence {
                channel,
                total,
                diagonal,
                cross: total - diagonal,
            }
        })
        .collect::<Vec<_>>();
    let total = channels.iter().map(|v| v.total).sum::<f64>();
    let diagonal = channels.iter().map(|v| v.diagonal).sum::<f64>();
    let cross = channels.iter().map(|v| v.cross).sum::<f64>();
    let closure = (total - eigenvalue).abs() / eigenvalue.abs().max(f64::MIN_POSITIVE);
    channels.sort_by(|a, b| {
        b.total
            .total_cmp(&a.total)
            .then_with(|| a.channel.cmp(&b.channel))
    });
    let mut top_total = channels.clone();
    top_total.truncate(TOP);
    channels.sort_by(|a, b| {
        b.cross
            .abs()
            .total_cmp(&a.cross.abs())
            .then_with(|| a.channel.cmp(&b.channel))
    });
    channels.truncate(TOP);
    ModeEvidence {
        bank,
        maximum,
        residue,
        eigenvalue,
        cross_bank_rayleigh: rayleigh,
        region_mass: region,
        top_bins: bin_rows,
        top_total_channels: top_total,
        top_cross_channels: channels,
        contribution_sums: [total, diagonal, cross, closure],
    }
}

fn multiply(matrix: &[Complex64], vector: &[Complex64], size: usize) -> Vec<Complex64> {
    (0..size)
        .map(|r| (0..size).map(|c| matrix[r * size + c] * vector[c]).sum())
        .collect()
}
fn dot(a: &[Complex64], b: &[Complex64]) -> Complex64 {
    a.iter().zip(b).map(|(x, y)| x.conj() * y).sum()
}
fn hash_usizes(values: &[usize]) -> u64 {
    let mut h = HASH_OFFSET;
    for v in values {
        hash_u64(&mut h, *v as u64);
    }
    h
}
fn hash_complex(values: &[Complex64]) -> u64 {
    let mut h = HASH_OFFSET;
    for v in values {
        hash_u64(&mut h, v.re.to_bits());
        hash_u64(&mut h, v.im.to_bits());
    }
    h
}
fn evidence_hash(review: &Review) -> u64 {
    let mut h = HASH_OFFSET;
    for row in &review.residues {
        hash_u64(&mut h, row.residue as u64);
        for v in row.eigenvalues {
            hash_u64(&mut h, v.to_bits());
        }
        for v in row.hashes {
            hash_u64(&mut h, v);
        }
    }
    for mode in &review.modes {
        hash_u64(&mut h, mode.eigenvalue.to_bits());
        for v in mode.region_mass {
            hash_u64(&mut h, v.to_bits());
        }
    }
    h
}
