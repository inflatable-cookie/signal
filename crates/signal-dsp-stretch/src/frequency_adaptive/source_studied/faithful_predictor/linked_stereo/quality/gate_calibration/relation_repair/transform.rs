use super::super::metrics::gram;

const RANK_FLOOR: f64 = 1.0e-10;

pub(super) struct Repair {
    pub(super) before: [Vec<f64>; 2],
    pub(super) channels: [Vec<f64>; 2],
    pub(super) matrix: [[f64; 2]; 2],
    pub(super) applied: bool,
    pub(super) energy_error: f64,
}

pub(super) fn repair(input: &[Vec<f64>; 2], output: [Vec<f64>; 2]) -> Repair {
    let input_gram = normalized_gram(input);
    let output_gram = normalized_gram(&output);
    let input_determinant = determinant(input_gram);
    let output_determinant = determinant(output_gram);
    let should_apply = input_determinant > RANK_FLOOR && output_determinant > RANK_FLOOR;
    if !should_apply {
        return Repair {
            before: output.clone(),
            channels: output,
            matrix: [[1.0, 0.0], [0.0, 1.0]],
            applied: false,
            energy_error: 0.0,
        };
    }

    let matrix = multiply(
        symmetric_sqrt(input_gram),
        inverse(symmetric_sqrt(output_gram)),
    );
    let before_energy = energy(&output);
    let channels = apply(&output, matrix);
    let energy_error =
        (energy(&channels) - before_energy).abs() / before_energy.max(f64::MIN_POSITIVE);
    Repair {
        before: output,
        channels,
        matrix,
        applied: true,
        energy_error,
    }
}

pub(in super::super) fn local_evidence(
    input: &[Vec<f64>; 2],
    before: &[Vec<f64>; 2],
    after: &[Vec<f64>; 2],
) -> (usize, usize, [f64; 2]) {
    let mut improved = 0;
    let mut maximum = [0.0_f64; 2];
    for window in 0..8 {
        let section = |channels: &[Vec<f64>; 2]| {
            std::array::from_fn(|channel| {
                let start = window * channels[channel].len() / 8;
                let end = (window + 1) * channels[channel].len() / 8;
                channels[channel][start..end].to_vec()
            })
        };
        let source = section(input);
        let before = super::super::metrics::gram_residual(&source, &section(before));
        let after = super::super::metrics::gram_residual(&source, &section(after));
        improved += usize::from(after < before);
        maximum[0] = maximum[0].max(before);
        maximum[1] = maximum[1].max(after);
    }
    (improved, 8, maximum)
}

fn normalized_gram(channels: &[Vec<f64>; 2]) -> [[f64; 2]; 2] {
    let [left, right, cross] = gram(channels);
    let trace = (left + right).max(f64::MIN_POSITIVE);
    [
        [left / trace, cross / trace],
        [cross / trace, right / trace],
    ]
}

fn symmetric_sqrt(matrix: [[f64; 2]; 2]) -> [[f64; 2]; 2] {
    let root_determinant = determinant(matrix).max(0.0).sqrt();
    let scale = (matrix[0][0] + matrix[1][1] + 2.0 * root_determinant).sqrt();
    [
        [
            (matrix[0][0] + root_determinant) / scale,
            matrix[0][1] / scale,
        ],
        [
            matrix[1][0] / scale,
            (matrix[1][1] + root_determinant) / scale,
        ],
    ]
}

fn inverse(matrix: [[f64; 2]; 2]) -> [[f64; 2]; 2] {
    let determinant = determinant(matrix);
    [
        [matrix[1][1] / determinant, -matrix[0][1] / determinant],
        [-matrix[1][0] / determinant, matrix[0][0] / determinant],
    ]
}

fn multiply(left: [[f64; 2]; 2], right: [[f64; 2]; 2]) -> [[f64; 2]; 2] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            left[row][0] * right[0][column] + left[row][1] * right[1][column]
        })
    })
}

fn apply(channels: &[Vec<f64>; 2], matrix: [[f64; 2]; 2]) -> [Vec<f64>; 2] {
    std::array::from_fn(|row| {
        channels[0]
            .iter()
            .zip(&channels[1])
            .map(|(left, right)| matrix[row][0] * left + matrix[row][1] * right)
            .collect()
    })
}

fn determinant(matrix: [[f64; 2]; 2]) -> f64 {
    matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0]
}

fn energy(channels: &[Vec<f64>; 2]) -> f64 {
    channels
        .iter()
        .flatten()
        .map(|sample| sample * sample)
        .sum()
}
