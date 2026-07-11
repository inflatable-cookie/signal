use rustfft::num_complex::Complex64;

use super::common_grid::{hash_u64, HASH_OFFSET};
use super::conditioning_attribution::conditioning_matrices;
use super::types::{
    StretchCommonGridJacobiEvidence as Evidence, StretchCommonGridJacobiReview as Review,
};

pub(crate) fn common_grid_hermitian_jacobi_review() -> Review {
    let controls = control_matrices()
        .into_iter()
        .map(|matrix| solve(&matrix))
        .collect::<Vec<_>>();
    let alias_blocks = conditioning_matrices()
        .iter()
        .map(|matrix| solve(matrix))
        .collect::<Vec<_>>();
    let mut maximum_errors = [0.0_f64; 4];
    for row in controls.iter().chain(&alias_blocks) {
        for (slot, value) in maximum_errors.iter_mut().zip(row.proof_errors) {
            *slot = slot.max(value);
        }
    }
    let passed = controls.iter().chain(&alias_blocks).all(passes);
    let mut evidence_hash = HASH_OFFSET;
    for row in controls.iter().chain(&alias_blocks) {
        for hash in row.hashes {
            hash_u64(&mut evidence_hash, hash);
        }
        for value in row.proof_errors {
            hash_u64(&mut evidence_hash, value.to_bits());
        }
    }
    Review {
        controls,
        alias_blocks,
        maximum_errors,
        evidence_hash,
        passed,
    }
}

fn solve(input: &[Complex64]) -> Evidence {
    let size = (input.len() as f64).sqrt() as usize;
    let hermitian_error = hermitian_error(input, size);
    if size == 0
        || size > 193
        || size * size != input.len()
        || hermitian_error > 1.0e-12
        || input.iter().any(|v| !v.re.is_finite() || !v.im.is_finite())
    {
        return rejected(size, hermitian_error);
    }
    let original = input.to_vec();
    let mut matrix = input.to_vec();
    let mut vectors = vec![Complex64::new(0.0, 0.0); size * size];
    for index in 0..size {
        vectors[index * size + index].re = 1.0;
    }
    let mut rotations = 0;
    let mut sweeps = 0;
    let mut converged = size == 1;
    let mut off_ratio = off_diagonal_ratio(&matrix, size);
    while sweeps < 64 && !converged {
        for p in 0..size {
            for q in p + 1..size {
                let pivot = matrix[p * size + q];
                let magnitude = pivot.norm();
                if magnitude == 0.0 {
                    continue;
                }
                let app = matrix[p * size + p].re;
                let aqq = matrix[q * size + q].re;
                let tau = (aqq - app) / (2.0 * magnitude);
                let t = if tau == 0.0 {
                    1.0
                } else {
                    tau.signum() / (tau.abs() + (1.0 + tau * tau).sqrt())
                };
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;
                let phase = pivot / magnitude;
                for k in 0..size {
                    if k != p && k != q {
                        let kp = matrix[k * size + p];
                        let kq = matrix[k * size + q];
                        let new_p = kp * phase * c - kq * s;
                        let new_q = kp * phase * s + kq * c;
                        matrix[k * size + p] = new_p;
                        matrix[p * size + k] = new_p.conj();
                        matrix[k * size + q] = new_q;
                        matrix[q * size + k] = new_q.conj();
                    }
                }
                matrix[p * size + p] = Complex64::new(app - t * magnitude, 0.0);
                matrix[q * size + q] = Complex64::new(aqq + t * magnitude, 0.0);
                matrix[p * size + q] = Complex64::new(0.0, 0.0);
                matrix[q * size + p] = Complex64::new(0.0, 0.0);
                for k in 0..size {
                    let kp = vectors[k * size + p];
                    let kq = vectors[k * size + q];
                    vectors[k * size + p] = kp * phase * c - kq * s;
                    vectors[k * size + q] = kp * phase * s + kq * c;
                }
                rotations += 1;
            }
        }
        sweeps += 1;
        off_ratio = off_diagonal_ratio(&matrix, size);
        converged = off_ratio <= 1.0e-13;
    }
    let mut order = (0..size).collect::<Vec<_>>();
    order.sort_by(|a, b| {
        matrix[*a * size + *a]
            .re
            .total_cmp(&matrix[*b * size + *b].re)
            .then_with(|| a.cmp(b))
    });
    let eigenvalues = order
        .iter()
        .map(|i| matrix[*i * size + *i].re)
        .collect::<Vec<_>>();
    let mut sorted = vec![Complex64::new(0.0, 0.0); size * size];
    for (column, source) in order.iter().enumerate() {
        let norm = (0..size)
            .map(|row| vectors[row * size + *source].norm_sqr())
            .sum::<f64>()
            .sqrt();
        for row in 0..size {
            sorted[row * size + column] = vectors[row * size + *source] / norm;
        }
        normalize_column_phase(&mut sorted, size, column);
    }
    let proof = proof_errors(&original, &eigenvalues, &sorted, size);
    Evidence {
        size,
        sweeps_and_rotations: [sweeps, rotations],
        converged,
        structural_errors: [hermitian_error, off_ratio],
        proof_errors: proof,
        hashes: [hash_reals(&eigenvalues), hash_complex(&sorted)],
        extrema: [eigenvalues[0], eigenvalues[size - 1]],
    }
}

fn passes(row: &Evidence) -> bool {
    row.converged
        && row.structural_errors[0] <= 1.0e-12
        && row.proof_errors[0] <= 1.0e-8
        && row.proof_errors[1] <= 1.0e-10
        && row.proof_errors[2] <= 1.0e-12
        && row.proof_errors[3] <= 1.0e-10
        && row.hashes.iter().all(|h| *h != 0)
}
fn rejected(size: usize, error: f64) -> Evidence {
    Evidence {
        size,
        sweeps_and_rotations: [0, 0],
        converged: false,
        structural_errors: [error, f64::INFINITY],
        proof_errors: [f64::INFINITY; 4],
        hashes: [0; 2],
        extrema: [f64::NAN; 2],
    }
}

fn proof_errors(a: &[Complex64], values: &[f64], v: &[Complex64], n: usize) -> [f64; 4] {
    let mut residual = 0.0_f64;
    let mut orth = 0.0_f64;
    for col in 0..n {
        let mut norm = 0.0;
        for row in 0..n {
            let av = (0..n)
                .map(|k| a[row * n + k] * v[k * n + col])
                .sum::<Complex64>();
            norm += (av - v[row * n + col] * values[col]).norm_sqr();
        }
        residual = residual.max(norm.sqrt() / values[col].abs().max(f64::MIN_POSITIVE));
        for other in 0..n {
            let dot = (0..n)
                .map(|row| v[row * n + col].conj() * v[row * n + other])
                .sum::<Complex64>();
            orth = orth.max((dot - Complex64::new((col == other) as u8 as f64, 0.0)).norm());
        }
    }
    let trace_a = (0..n).map(|i| a[i * n + i].re).sum::<f64>();
    let trace_e = values.iter().sum::<f64>();
    let frob_a = a.iter().map(|x| x.norm_sqr()).sum::<f64>();
    let frob_e = values.iter().map(|x| x * x).sum::<f64>();
    [
        residual,
        orth,
        (trace_a - trace_e).abs() / trace_a.abs().max(f64::MIN_POSITIVE),
        (frob_a - frob_e).abs() / frob_a.max(f64::MIN_POSITIVE),
    ]
}
fn normalize_column_phase(v: &mut [Complex64], n: usize, col: usize) {
    let pivot = (0..n)
        .max_by(|a, b| {
            v[*a * n + col]
                .norm_sqr()
                .total_cmp(&v[*b * n + col].norm_sqr())
                .then_with(|| b.cmp(a))
        })
        .unwrap();
    let x = v[pivot * n + col];
    if x.norm() > 0.0 {
        let phase = x.conj() / x.norm();
        for row in 0..n {
            v[row * n + col] *= phase;
        }
    }
}
fn hermitian_error(a: &[Complex64], n: usize) -> f64 {
    let scale = a
        .iter()
        .map(|x| x.norm_sqr())
        .sum::<f64>()
        .sqrt()
        .max(f64::MIN_POSITIVE);
    let mut e = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            e = e.max((a[i * n + j] - a[j * n + i].conj()).norm());
        }
    }
    e / scale
}
fn off_diagonal_ratio(a: &[Complex64], n: usize) -> f64 {
    let total = a
        .iter()
        .map(|x| x.norm_sqr())
        .sum::<f64>()
        .sqrt()
        .max(f64::MIN_POSITIVE);
    let off = (0..n)
        .flat_map(|i| (0..n).map(move |j| (i, j)))
        .filter(|(i, j)| i != j)
        .map(|(i, j)| a[i * n + j].norm_sqr())
        .sum::<f64>()
        .sqrt();
    off / total
}
fn hash_complex(v: &[Complex64]) -> u64 {
    let mut h = HASH_OFFSET;
    for x in v {
        hash_u64(&mut h, x.re.to_bits());
        hash_u64(&mut h, x.im.to_bits());
    }
    h
}
fn hash_reals(v: &[f64]) -> u64 {
    let mut h = HASH_OFFSET;
    for x in v {
        hash_u64(&mut h, x.to_bits());
    }
    h
}
fn control_matrices() -> Vec<Vec<Complex64>> {
    vec![
        vec![Complex64::new(2.0, 0.0)],
        vec![
            Complex64::new(2.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(3.0, 0.0),
        ],
        vec![
            Complex64::new(2.0, 0.0),
            Complex64::new(0.0, 1.0),
            Complex64::new(0.0, -1.0),
            Complex64::new(3.0, 0.0),
        ],
        vec![
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(4.0, 0.0),
        ],
        vec![
            Complex64::new(2.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(2.0, 0.0),
        ],
        vec![
            Complex64::new(1.0, 0.0),
            Complex64::new(1.0e-10, 0.0),
            Complex64::new(1.0e-10, 0.0),
            Complex64::new(1.0 + 1.0e-9, 0.0),
        ],
    ]
}
