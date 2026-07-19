use super::*;

use fixtures::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct LockedPeakRelationAttribution {
    reset_relation_error: f64,
    attack_relation_error: f64,
    unlocked_rotation_separation: f64,
    borrowed_input_relation: f64,
    borrowed_output_relation: f64,
    borrowed_relation_error: f64,
    local_rotation_separation: f64,
    borrowed_locked_atoms: usize,
    local_locked_atoms: usize,
    hash: u64,
}

#[test]
fn direct_scale_timeline_rule_31aa_locked_peak_correction_preserves_relation() {
    let review = locked_peak_relation_attribution();
    assert_eq!(review, locked_peak_relation_attribution());
    eprintln!("direct_scale_timeline_rule_31z_locked_peak_relation {review:#?}");
    assert!(review.reset_relation_error <= 1.0e-12, "{review:#?}");
    assert!(review.attack_relation_error <= 1.0e-12, "{review:#?}");
    assert!(review.unlocked_rotation_separation > 1.0e-6, "{review:#?}");
    assert!(review.borrowed_input_relation.abs() > 1.0e-6, "{review:#?}");
    assert!(
        wrap(review.borrowed_output_relation - review.borrowed_input_relation).abs() <= 1.0e-12,
        "{review:#?}"
    );
    assert!(review.borrowed_relation_error <= 1.0e-12, "{review:#?}");
    assert_eq!(review.hash, 0x2b81_0452_5bad_0418, "{review:#?}");
    assert!(review.local_rotation_separation > 1.0e-6, "{review:#?}");
    assert!(review.borrowed_locked_atoms > 0, "{review:#?}");
    assert!(review.local_locked_atoms > 0, "{review:#?}");
}

fn locked_peak_relation_attribution() -> LockedPeakRelationAttribution {
    let mut prepared = prepare(48_000, 2, 1.5, false).expect("attribution geometry");
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let low_peak = 10;
    let high_peak = prepared.owned_bins[0] + prepared.owned_bins[1];
    let frame = |phases: [[f64; 2]; 2], owner: usize| {
        let mut values = vec![Complex64::default(); 2 * atoms];
        for (peak_index, peak) in [low_peak, high_peak].into_iter().enumerate() {
            for channel in 0..2 {
                let magnitude = if channel == owner { 1.0 } else { 0.5 };
                values[channel * atoms + peak] =
                    Complex64::from_polar(magnitude, phases[peak_index][channel]);
            }
        }
        values
    };
    let reset_input = frame([[0.2, -0.2], [-0.3, -0.7]], 1);
    let attack_input = frame([[1.0, 0.3], [0.5, -0.2]], 0);
    let unlocked_input = frame([[1.25, 0.45], [0.75, -0.05]], 0);
    let locked_input = frame([[1.5, 0.55], [1.0, 0.1]], 0);

    let (reset_output, _, _) = tick(
        &mut prepared,
        &reset_input,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let reset_relation_error = relation_error(&reset_input, &reset_output, atoms, low_peak);

    let (attack_output, attack_states, _) = tick(
        &mut prepared,
        &attack_input,
        &guidance(atoms, TerminalState::Attack),
        true,
        false,
    );
    assert_eq!(attack_states[low_peak], TerminalState::Attack);
    let attack_relation_error = relation_error(&attack_input, &attack_output, atoms, low_peak);

    let (unlocked_output, unlocked_states, _) = tick(
        &mut prepared,
        &unlocked_input,
        &guidance(atoms, TerminalState::Unlocked),
        false,
        false,
    );
    assert_eq!(unlocked_states[low_peak], TerminalState::Unlocked);
    let unlocked_rotation_separation =
        rotation_separation(&unlocked_input, &unlocked_output, atoms, low_peak);

    let (locked_output, locked_states, locked_report) = tick(
        &mut prepared,
        &locked_input,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    assert_eq!(locked_states[low_peak], TerminalState::Locked);
    assert_eq!(locked_states[high_peak], TerminalState::Locked);
    let borrowed_input_relation = relation(&locked_input, atoms, low_peak);
    let borrowed_output_relation = relation(&locked_output, atoms, low_peak);
    let borrowed_relation_error = wrap(borrowed_output_relation - borrowed_input_relation).abs();
    let local_rotation_separation =
        rotation_separation(&locked_input, &locked_output, atoms, high_peak);

    let mut hash = HASH_OFFSET;
    for value in [
        reset_relation_error,
        attack_relation_error,
        unlocked_rotation_separation,
        borrowed_input_relation,
        borrowed_output_relation,
        borrowed_relation_error,
        local_rotation_separation,
    ] {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in [
        locked_report.borrowed_locked_atoms,
        locked_report.local_locked_atoms,
    ] {
        hash_usize(&mut hash, value);
    }
    LockedPeakRelationAttribution {
        reset_relation_error,
        attack_relation_error,
        unlocked_rotation_separation,
        borrowed_input_relation,
        borrowed_output_relation,
        borrowed_relation_error,
        local_rotation_separation,
        borrowed_locked_atoms: locked_report.borrowed_locked_atoms,
        local_locked_atoms: locked_report.local_locked_atoms,
        hash,
    }
}

fn relation(values: &[Complex64], atoms: usize, atom: usize) -> f64 {
    wrap(values[atoms + atom].arg() - values[atom].arg())
}

fn relation_error(input: &[Complex64], output: &[Complex64], atoms: usize, atom: usize) -> f64 {
    wrap(relation(output, atoms, atom) - relation(input, atoms, atom)).abs()
}

fn rotation_separation(
    input: &[Complex64],
    output: &[Complex64],
    atoms: usize,
    atom: usize,
) -> f64 {
    let rotations = [0, 1].map(|channel| {
        let index = channel * atoms + atom;
        wrap(output[index].arg() - input[index].arg())
    });
    wrap(rotations[1] - rotations[0]).abs()
}
