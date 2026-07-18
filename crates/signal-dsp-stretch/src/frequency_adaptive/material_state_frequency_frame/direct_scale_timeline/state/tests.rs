use super::*;

mod fixtures;
use fixtures::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct LockedPeakRelationAttribution {
    reset_relation_error: f64,
    attack_relation_error: f64,
    unlocked_rotation_separation: f64,
    borrowed_input_relation: f64,
    borrowed_output_relation: f64,
    borrowed_relation_loss: f64,
    local_rotation_separation: f64,
    borrowed_regions: usize,
    local_regions: usize,
    hash: u64,
}

#[test]
fn direct_scale_timeline_rule_31z_state_terminal_order_and_repeat_pass() {
    let first = terminal_sequence();
    let second = terminal_sequence();
    assert_eq!(first, second);
    let (hash, [reset, attack, unlocked, ordinary, locked]) = first;
    assert_eq!(hash, 0x4305_43f8_e1dc_e721);
    assert_eq!(reset.states, [631, 0, 0, 0, 0]);
    assert_eq!(attack.states, [0, 270, 0, 361, 0]);
    assert_eq!(unlocked.states, [0, 0, 0, 631, 0]);
    assert_eq!(ordinary.states, [0, 0, 631, 0, 0]);
    assert_eq!(locked.states, [0, 0, 0, 0, 631]);
    assert_eq!(reset.borrowed_regions, 0);
    assert_eq!(attack.borrowed_regions, 0);
    assert_eq!(unlocked.borrowed_regions, 0);
    assert_eq!(ordinary.borrowed_regions, 0);
    assert!(locked.borrowed_regions > 0);
    assert!(locked.local_regions > 0);
    assert!([reset, attack, unlocked, ordinary, locked]
        .iter()
        .all(|report| report.non_finite_values == 0));
    eprintln!("direct_scale_timeline_rule_31z_state_hash {hash:016x} {locked:#?}");
}

#[test]
fn direct_scale_timeline_rule_31z_state_borrowing_preserves_peer_ownership() {
    let mut prepared = prepare(48_000, 2, 1.5, false).expect("state proof geometry");
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let low_peak = 10;
    let high_peak = prepared.owned_bins[0] + prepared.owned_bins[1];
    let mut seed = vec![Complex64::default(); 2 * atoms];
    let mut current = seed.clone();
    for (peak, phase) in [(low_peak, 0.2), (high_peak, -0.3)] {
        seed[peak] = Complex64::from_polar(0.5, phase);
        seed[atoms + peak] = Complex64::from_polar(1.0, phase - 0.4);
        current[peak] = Complex64::from_polar(1.2, phase + 0.8);
        current[atoms + peak] = Complex64::from_polar(0.6, phase + 0.1);
    }
    current[low_peak + 1] = Complex64::from_polar(0.3, 1.7);
    current[atoms + low_peak + 1] = Complex64::from_polar(0.2, -1.1);
    current[high_peak + 1] = Complex64::from_polar(0.25, 0.9);
    current[atoms + high_peak + 1] = Complex64::from_polar(0.15, -0.7);
    tick(
        &mut prepared,
        &seed,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let (output, states, report) = tick(
        &mut prepared,
        &current,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );

    assert_eq!(report.borrowed_regions, 1, "{report:#?}");
    assert_eq!(report.local_regions, 1, "{report:#?}");
    assert!(report.owner_switches >= 2, "{report:#?}");
    assert_eq!(states[low_peak], TerminalState::Locked);
    assert_eq!(states[high_peak], TerminalState::Locked);
    for index in [low_peak, low_peak + 1, high_peak, high_peak + 1] {
        for channel in 0..2 {
            let index = channel * atoms + index;
            assert!((output[index].norm() - current[index].norm()).abs() <= 1.0e-12);
        }
    }
    let peer_input_offset =
        wrap(current[atoms + low_peak + 1].arg() - current[atoms + low_peak].arg());
    let peer_output_offset =
        wrap(output[atoms + low_peak + 1].arg() - output[atoms + low_peak].arg());
    assert!(wrap(peer_output_offset - peer_input_offset).abs() <= 1.0e-12);
    let high_input_offset = wrap(current[high_peak + 1].arg() - current[high_peak].arg());
    let high_output_offset = wrap(output[high_peak + 1].arg() - output[high_peak].arg());
    assert!(wrap(high_output_offset - high_input_offset).abs() <= 1.0e-12);
}

#[test]
fn direct_scale_timeline_rule_31z_locked_peak_relation_attribution_confirms_collapse() {
    let review = locked_peak_relation_attribution();
    eprintln!("direct_scale_timeline_rule_31z_locked_peak_relation {review:#?}");
    assert!(review.reset_relation_error <= 1.0e-12, "{review:#?}");
    assert!(review.attack_relation_error <= 1.0e-12, "{review:#?}");
    assert!(review.unlocked_rotation_separation > 1.0e-6, "{review:#?}");
    assert!(review.borrowed_input_relation.abs() > 1.0e-6, "{review:#?}");
    assert!(
        review.borrowed_output_relation.abs() <= 1.0e-12,
        "{review:#?}"
    );
    assert!(review.borrowed_relation_loss > 1.0e-6, "{review:#?}");
    assert!(review.local_rotation_separation > 1.0e-6, "{review:#?}");
    assert_eq!(review.borrowed_regions, 1, "{review:#?}");
    assert_eq!(review.local_regions, 1, "{review:#?}");
    assert_eq!(review.hash, 0x346e_3290_81ad_f701, "{review:#?}");
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
    let borrowed_relation_loss = wrap(borrowed_output_relation - borrowed_input_relation).abs();
    let local_rotation_separation =
        rotation_separation(&locked_input, &locked_output, atoms, high_peak);

    let mut hash = HASH_OFFSET;
    for value in [
        reset_relation_error,
        attack_relation_error,
        unlocked_rotation_separation,
        borrowed_input_relation,
        borrowed_output_relation,
        borrowed_relation_loss,
        local_rotation_separation,
    ] {
        hash_u64(&mut hash, value.to_bits());
    }
    for value in [locked_report.borrowed_regions, locked_report.local_regions] {
        hash_usize(&mut hash, value);
    }
    LockedPeakRelationAttribution {
        reset_relation_error,
        attack_relation_error,
        unlocked_rotation_separation,
        borrowed_input_relation,
        borrowed_output_relation,
        borrowed_relation_loss,
        local_rotation_separation,
        borrowed_regions: locked_report.borrowed_regions,
        local_regions: locked_report.local_regions,
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

#[test]
fn direct_scale_timeline_rule_31z_state_unlocked_is_channel_local() {
    let mut prepared = prepare(44_100, 2, 0.75, false).expect("state proof geometry");
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let first = dense_frame(&prepared, 0);
    let second = dense_frame(&prepared, 1);
    tick(
        &mut prepared,
        &first,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let (output, states, report) = tick(
        &mut prepared,
        &second,
        &guidance(atoms, TerminalState::Unlocked),
        false,
        false,
    );
    assert_eq!(report.states, [0, 0, 0, atoms, 0]);
    assert!(states.iter().all(|state| *state == TerminalState::Unlocked));
    let atom = 37;
    let rotations = [0, 1].map(|channel| {
        wrap(output[channel * atoms + atom].arg() - second[channel * atoms + atom].arg())
    });
    assert!(wrap(rotations[0] - rotations[1]).abs() > 1.0e-6);
}

#[test]
fn direct_scale_timeline_rule_31z_state_recovery_boundaries_and_shape_pass() {
    for sample_rate in PROOF_RATES {
        let mut prepared = prepare(sample_rate, 2, 4.0, false).expect("state proof geometry");
        let atoms = prepared.owned_bins.iter().sum::<usize>();
        assert!((prepared.atom_frequency(Scale::Middle, 0) - 750.0).abs() <= f64::EPSILON);
        if prepared.owned_bins[Scale::Short.index()] > 0 {
            assert!((prepared.atom_frequency(Scale::Short, 0) - 6_000.0).abs() <= f64::EPSILON);
        }
        let source = dense_frame(&prepared, 0);
        tick(
            &mut prepared,
            &source,
            &guidance(atoms, TerminalState::Locked),
            false,
            false,
        );
        let silence = vec![Complex64::default(); 2 * atoms];
        let (silent_output, _, silent) = tick(
            &mut prepared,
            &silence,
            &guidance(atoms, TerminalState::Locked),
            false,
            false,
        );
        assert_eq!(silent.states, [atoms, 0, 0, 0, 0]);
        assert!(silent_output
            .iter()
            .all(|value| *value == Complex64::default()));
        let (_, _, recovered) = tick(
            &mut prepared,
            &source,
            &guidance(atoms, TerminalState::Locked),
            false,
            false,
        );
        assert_eq!(recovered.states, [atoms, 0, 0, 0, 0]);
    }

    let mut prepared = prepare(48_000, 2, 1.0, false).expect("state proof geometry");
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let snapshot = (
        prepared.phase.clone(),
        prepared.regions.clone(),
        prepared.has_state,
        prepared.region_slot,
    );
    let mut output = vec![Complex64::default(); 2 * atoms];
    let mut states = vec![TerminalState::Reset; atoms];
    assert_eq!(
        prepared.process_state_tick(
            &vec![Complex64::default(); 2 * atoms - 1],
            &guidance(atoms, TerminalState::Locked),
            StateTickControl {
                transient_center: false,
                ordinary_bypass: false,
                analysis_advance: prepared.hop as f64
            },
            &mut output,
            &mut states,
        ),
        Err(StateError::CurrentShape)
    );
    assert_eq!(
        snapshot,
        (
            prepared.phase.clone(),
            prepared.regions.clone(),
            prepared.has_state,
            prepared.region_slot
        )
    );
}
