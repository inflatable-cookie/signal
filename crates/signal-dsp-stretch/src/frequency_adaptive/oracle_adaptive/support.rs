use super::{Evidence, Render, HASH_OFFSET};

pub(super) fn empty_evidence() -> Evidence {
    Evidence {
        window_counts: [0; 4],
        source_hop_extrema: [0; 2],
        output_hop_extrema: [0; 2],
        maximum_mapping_error: 0.0,
        frame_operator_bounds: [0.0; 2],
        uncovered_output_frames: 0,
        illegal_transitions: 0,
        reflected_reads: 0,
        coefficient_count: 0,
        phase_count: 0,
        conjugate_symmetry_error: 0.0,
        imaginary_residue: 0.0,
        non_finite_values: 0,
        hashes: [HASH_OFFSET; 3],
    }
}

pub(super) fn assert_mechanism(name: &str, ratio: f64, render: &Render) {
    let evidence = &render.evidence;
    assert_eq!(evidence.illegal_transitions, 0, "{name} {ratio} schedule");
    assert_eq!(
        evidence.uncovered_output_frames, 0,
        "{name} {ratio} coverage"
    );
    assert!(
        evidence.frame_operator_bounds[0] > 0.0,
        "{name} {ratio} operator"
    );
    assert!(
        evidence.maximum_mapping_error <= 0.5,
        "{name} {ratio} mapping"
    );
    assert!(
        evidence.conjugate_symmetry_error <= 1.0e-9,
        "{name} {ratio} symmetry"
    );
    assert!(
        evidence.imaginary_residue <= 1.0e-9,
        "{name} {ratio} residue"
    );
    assert_eq!(evidence.non_finite_values, 0, "{name} {ratio} finite");
}

pub(super) fn assert_identity(name: &str, input: &[f64], output: &[f64]) {
    let errors = input
        .iter()
        .zip(output)
        .map(|(a, b)| (a - b).abs())
        .collect::<Vec<_>>();
    let peak = errors.iter().copied().fold(0.0, f64::max);
    let rms = (errors.iter().map(|error| error * error).sum::<f64>() / input.len() as f64).sqrt();
    assert!(
        peak <= 1.0e-5 && rms <= 1.0e-6,
        "{name} identity peak={peak} rms={rms}"
    );
}

pub(super) fn peak_index(samples: &[f64]) -> usize {
    samples
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .map(|item| item.0)
        .unwrap_or(0)
}

type Control = (&'static str, Vec<f64>, Vec<usize>);

pub(super) fn controls() -> Vec<Control> {
    let len = 8_192;
    let sine = |frequency: f64| {
        (0..len)
            .map(|index| 0.5 * (std::f64::consts::TAU * frequency * index as f64 / 48_000.0).sin())
            .collect::<Vec<_>>()
    };
    let mut impulse = vec![0.0; len];
    impulse[len / 2] = 1.0;
    let mut dense = vec![0.0; len];
    dense[len / 2 - 128] = 1.0;
    dense[len / 2 + 128] = 0.75;
    let chirp = (0..len)
        .map(|index| {
            let t = index as f64 / 48_000.0;
            0.5 * (std::f64::consts::TAU * (100.0 * t + 2_000.0 * t * t)).sin()
        })
        .collect();
    let mut state = 0x243f_6a88_85a3_08d3_u64;
    let noise = (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 40) as f64 / (1_u64 << 24) as f64 - 0.5
        })
        .collect::<Vec<_>>();
    let mut boundary = vec![0.0; len];
    boundary[0] = 1.0;
    boundary[len - 1] = -0.75;
    let mut mixed = sine(220.0);
    mixed
        .iter_mut()
        .zip(&noise)
        .for_each(|(tone, noise)| *tone = *tone * 0.5 + noise * 0.1);
    mixed[len / 2] += 0.8;
    vec![
        ("tone", sine(440.0), vec![]),
        ("chirp", chirp, vec![]),
        ("impulse", impulse, vec![len / 2]),
        ("dense", dense, vec![len / 2 - 128, len / 2 + 128]),
        ("boundary", boundary, vec![0, len - 1]),
        ("mixed", mixed, vec![len / 2]),
        ("noise", noise, vec![]),
        ("silence", vec![0.0; len], vec![]),
    ]
}
