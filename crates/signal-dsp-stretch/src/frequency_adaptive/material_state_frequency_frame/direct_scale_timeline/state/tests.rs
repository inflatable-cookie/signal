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
    borrowed_relation_error: f64,
    local_rotation_separation: f64,
    borrowed_locked_atoms: usize,
    local_locked_atoms: usize,
    hash: u64,
}

#[test]
fn direct_scale_timeline_rule_31z_state_terminal_order_and_repeat_pass() {
    let first = terminal_sequence();
    let second = terminal_sequence();
    assert_eq!(first, second);
    let (hash, [reset, attack, unlocked, ordinary, locked]) = first;
    assert_eq!(hash, 0x5ae6_5416_2d4e_d279);
    assert_eq!(reset.states, [631, 0, 0, 0, 0]);
    assert_eq!(attack.states, [0, 270, 0, 361, 0]);
    assert_eq!(unlocked.states, [0, 0, 0, 631, 0]);
    assert_eq!(ordinary.states, [0, 0, 631, 0, 0]);
    assert_eq!(locked.states, [0, 0, 0, 0, 631]);
    assert_eq!(reset.borrowed_locked_atoms, 0);
    assert_eq!(attack.borrowed_locked_atoms, 0);
    assert_eq!(unlocked.borrowed_locked_atoms, 0);
    assert_eq!(ordinary.borrowed_locked_atoms, 0);
    assert!(locked.borrowed_locked_atoms > 0);
    assert!(locked.local_locked_atoms > 0);
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
        seed[peak + 1] = Complex64::from_polar(0.2, phase + 0.3);
        seed[atoms + peak + 1] = Complex64::from_polar(0.15, phase - 0.2);
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

    assert!(report.borrowed_locked_atoms > 0, "{report:#?}");
    assert!(report.local_locked_atoms > 0, "{report:#?}");
    assert!(report.trajectory_channel_switches > 0, "{report:#?}");
    assert_eq!(states[low_peak], TerminalState::Locked);
    assert_eq!(states[high_peak], TerminalState::Locked);
    for index in [low_peak, low_peak + 1, high_peak, high_peak + 1] {
        for channel in 0..2 {
            let index = channel * atoms + index;
            assert!((output[index].norm() - current[index].norm()).abs() <= 1.0e-12);
        }
    }
    let peer_input_offset = wrap(current[atoms + low_peak + 1].arg() - current[low_peak].arg());
    let peer_output_offset = wrap(output[atoms + low_peak + 1].arg() - output[low_peak].arg());
    assert!(
        wrap(peer_output_offset - peer_input_offset).abs() <= 1.0e-12,
        "input={peer_input_offset} output={peer_output_offset} records={:?}",
        [
            current_record(&prepared, atoms, 0, low_peak),
            current_record(&prepared, atoms, 1, low_peak + 1),
        ]
    );
    let high_input_offset = wrap(current[high_peak + 1].arg() - current[high_peak].arg());
    let high_output_offset = wrap(output[high_peak + 1].arg() - output[high_peak].arg());
    assert!(wrap(high_output_offset - high_input_offset).abs() <= 1.0e-12);
}

#[derive(Clone, Debug, PartialEq)]
struct StaggeredRun {
    atoms: usize,
    current: Vec<Complex64>,
    output: Vec<Complex64>,
    records: Vec<RegionRecord>,
    report: StateTickReport,
}

#[test]
fn direct_scale_timeline_rule_31ac_staggered_peaks_survive_compatible_borrowing() {
    let first = staggered_run(false);
    let second = staggered_run(false);
    assert_eq!(first, second);
    eprintln!(
        "direct_scale_timeline_rule_31ac_staggered_hash {:016x}",
        first.report.hash
    );
    assert_eq!(first.report.hash, 0xfcbd_fd99_1bd0_4db1);
    let range = 8..13;
    assert_eq!(
        joint_fixture_peaks(&first.current, first.atoms, range),
        [11]
    );

    let left = first.records[9];
    let right = first.records[first.atoms + 9];
    assert_eq!(left.peak, 9);
    assert_eq!(right.peak, 11);
    assert_eq!(right.trajectory_channel, 0);
    assert!(first.report.borrowed_locked_atoms > 0, "{first:#?}");
    assert!(first.report.local_locked_atoms > 0, "{first:#?}");
    assert!(first.report.trajectory_channel_switches > 0, "{first:#?}");
    assert!(first.report.channel_peak_disagreements > 0, "{first:#?}");

    let input_offset =
        wrap(first.current[first.atoms + 9].arg() - first.current[first.atoms + 8].arg());
    let output_offset =
        wrap(first.output[first.atoms + 9].arg() - first.output[first.atoms + 8].arg());
    assert!(wrap(output_offset - input_offset).abs() <= 1.0e-12);
}

#[test]
fn direct_scale_timeline_rule_31ac_fallback_tie_and_boundary_matrix() {
    let mut prepared = prepare(48_000, 2, 1.5, false).expect("fallback geometry");
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let mut seed = vec![Complex64::default(); 2 * atoms];
    for (channel, peak) in [(0, 8), (1, 12)] {
        seed[channel * atoms + peak] = Complex64::from_polar(1.0, 0.2 + channel as f64);
        seed[channel * atoms + 10] = Complex64::from_polar(0.2, 0.4 + channel as f64);
    }
    tick(
        &mut prepared,
        &seed,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let mut current = vec![Complex64::default(); 2 * atoms];
    current[10] = Complex64::from_polar(1.0, 0.7);
    current[atoms + 10] = Complex64::from_polar(0.5, -0.5);
    tick(
        &mut prepared,
        &current,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    assert_eq!(
        current_record(&prepared, atoms, 1, 10).trajectory_channel,
        1
    );

    let mut prepared = prepare(48_000, 2, 1.5, false).expect("unsupported owner geometry");
    let mut owner_seed = vec![Complex64::default(); 2 * atoms];
    for channel in 0..2 {
        for atom in 8..12 {
            let magnitude = if atom == 10 { 1.0 } else { 0.2 };
            owner_seed[channel * atoms + atom] =
                Complex64::from_polar(magnitude, channel as f64 + atom as f64 * 0.1);
        }
    }
    tick(
        &mut prepared,
        &owner_seed,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let mut owner_current = vec![Complex64::default(); 2 * atoms];
    owner_current[8] = Complex64::from_polar(1.0, 0.3);
    owner_current[9] = Complex64::from_polar(0.2, 0.5);
    for (atom, magnitude) in [(8, 0.5), (9, 0.6), (10, 0.8), (11, 1.2)] {
        owner_current[atoms + atom] = Complex64::from_polar(magnitude, -0.7 + atom as f64 * 0.1);
    }
    tick(
        &mut prepared,
        &owner_current,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    assert_eq!(current_record(&prepared, atoms, 1, 8).peak, 11);
    assert_eq!(current_record(&prepared, atoms, 1, 8).trajectory_channel, 1);

    let mut prepared = prepare(48_000, 2, 1.5, false).expect("unsupported geometry");
    let mut seed = vec![Complex64::default(); 2 * atoms];
    seed[10] = Complex64::from_polar(1.0, 0.2);
    seed[atoms + 10] = Complex64::from_polar(1.0, -0.4);
    tick(
        &mut prepared,
        &seed,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let previous_base = prepared.region_slot * 2 * atoms;
    prepared.regions[previous_base + 10].supported = false;
    let mut current = vec![Complex64::default(); 2 * atoms];
    current[10] = Complex64::from_polar(1.0, 0.8);
    current[atoms + 10] = Complex64::from_polar(0.5, -0.1);
    tick(
        &mut prepared,
        &current,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    assert_eq!(
        current_record(&prepared, atoms, 1, 10).trajectory_channel,
        1
    );

    let mut prepared = prepare(48_000, 2, 1.5, false).expect("tie geometry");
    tick(
        &mut prepared,
        &seed,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let mut tie = vec![Complex64::default(); 2 * atoms];
    tie[10] = Complex64::from_polar(1.0, 0.8);
    tie[atoms + 10] = Complex64::from_polar(1.0, -0.1);
    tick(
        &mut prepared,
        &tie,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    assert_eq!(
        current_record(&prepared, atoms, 1, 10).trajectory_channel,
        0
    );

    let mut prepared = prepare(48_000, 2, 1.5, false).expect("peak-tie geometry");
    let mut peak_tie = vec![Complex64::default(); 2 * atoms];
    for channel in 0..2 {
        peak_tie[channel * atoms + 10] = Complex64::from_polar(1.0, 0.2);
        peak_tie[channel * atoms + 11] = Complex64::from_polar(1.0, 0.4);
    }
    tick(
        &mut prepared,
        &peak_tie,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    assert_eq!(current_record(&prepared, atoms, 0, 11).peak, 10);
    assert_eq!(current_record(&prepared, atoms, 1, 11).peak, 10);

    let mut prepared = prepare(48_000, 2, 1.5, false).expect("boundary geometry");
    let boundary = prepared.owned_bins[0] + prepared.owned_bins[1];
    let mut boundary_seed = vec![Complex64::default(); 2 * atoms];
    boundary_seed[boundary] = Complex64::from_polar(1.0, 0.2);
    boundary_seed[atoms + boundary] = Complex64::from_polar(1.0, -0.4);
    tick(
        &mut prepared,
        &boundary_seed,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let mut boundary_current = boundary_seed.clone();
    boundary_current[boundary] = Complex64::from_polar(1.0, 0.8);
    boundary_current[atoms + boundary] = Complex64::from_polar(0.5, -0.1);
    tick(
        &mut prepared,
        &boundary_current,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    assert_eq!(prepared.atom_frequency(Scale::Short, 0), LINK_LIMIT_HZ);
    assert_eq!(
        current_record(&prepared, atoms, 1, boundary).trajectory_channel,
        1
    );
}

#[test]
fn direct_scale_timeline_rule_31ac_swap_rates_storage_and_repeat_pass() {
    let normal = staggered_run(false);
    let swapped = staggered_run(true);
    for atom in 0..normal.atoms {
        assert_eq!(normal.output[atom], swapped.output[swapped.atoms + atom]);
        assert_eq!(normal.output[normal.atoms + atom], swapped.output[atom]);
    }
    assert_eq!(normal.report.states, swapped.report.states);
    assert_eq!(
        normal.report.borrowed_locked_atoms,
        swapped.report.borrowed_locked_atoms
    );
    assert_eq!(
        normal.report.local_locked_atoms,
        swapped.report.local_locked_atoms
    );
    assert_eq!(
        normal.report.channel_peak_disagreements,
        swapped.report.channel_peak_disagreements
    );

    for sample_rate in PROOF_RATES {
        let run = || {
            let mut prepared = prepare(sample_rate, 2, 1.5, false).expect("proof-rate geometry");
            let atoms = prepared.owned_bins.iter().sum::<usize>();
            assert_eq!(prepared.phase.len(), 4 * atoms);
            assert_eq!(prepared.regions.len(), 4 * atoms);
            let first = dense_frame(&prepared, 0);
            let second = dense_frame(&prepared, 1);
            tick(
                &mut prepared,
                &first,
                &guidance(atoms, TerminalState::Locked),
                false,
                false,
            );
            tick(
                &mut prepared,
                &second,
                &guidance(atoms, TerminalState::Locked),
                false,
                false,
            )
        };
        let first = run();
        let second = run();
        assert_eq!(first, second);
        assert_eq!(first.2.non_finite_values, 0);
    }
}

fn staggered_run(swap: bool) -> StaggeredRun {
    let mut prepared = prepare(48_000, 2, 1.5, false).expect("staggered geometry");
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let mut seed = vec![Complex64::default(); 2 * atoms];
    let mut current = seed.clone();
    let seed_magnitudes = [0.2, 0.4, 1.0, 0.4, 0.2];
    let current_magnitudes = [[0.3, 1.2, 0.45, 0.25, 0.15], [0.15, 0.25, 0.4, 1.3, 0.3]];
    for logical_channel in 0..2 {
        let channel = if swap {
            1 - logical_channel
        } else {
            logical_channel
        };
        for (offset, atom) in (8..13).enumerate() {
            seed[channel * atoms + atom] = Complex64::from_polar(
                seed_magnitudes[offset],
                logical_channel as f64 * 0.7 + offset as f64 * 0.13,
            );
            current[channel * atoms + atom] = Complex64::from_polar(
                current_magnitudes[logical_channel][offset],
                logical_channel as f64 * -0.5 + offset as f64 * 0.21 + 0.4,
            );
        }
    }
    tick(
        &mut prepared,
        &seed,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let (output, _, report) = tick(
        &mut prepared,
        &current,
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    let base = prepared.region_slot * 2 * atoms;
    StaggeredRun {
        atoms,
        current,
        output,
        records: prepared.regions[base..base + 2 * atoms].to_vec(),
        report,
    }
}

fn current_record(prepared: &Prepared, atoms: usize, channel: usize, atom: usize) -> RegionRecord {
    prepared.regions[prepared.region_slot * 2 * atoms + channel * atoms + atom]
}

fn joint_fixture_peaks(
    current: &[Complex64],
    atoms: usize,
    range: std::ops::Range<usize>,
) -> Vec<usize> {
    range
        .clone()
        .filter(|atom| {
            let candidate = current[*atom]
                .norm_sqr()
                .max(current[atoms + *atom].norm_sqr());
            candidate > SUPPORT_FLOOR
                && !((*atom).saturating_sub(2).max(range.start)..(*atom + 3).min(range.end)).any(
                    |other| {
                        other != *atom
                            && (current[other]
                                .norm_sqr()
                                .max(current[atoms + other].norm_sqr())
                                > candidate
                                || (other < *atom
                                    && current[other]
                                        .norm_sqr()
                                        .max(current[atoms + other].norm_sqr())
                                        == candidate))
                    },
                )
        })
        .collect()
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
    let snapshot = state_snapshot(&prepared);
    let mut output = vec![Complex64::default(); 2 * atoms];
    let mut states = vec![TerminalState::Reset; atoms];
    let current = vec![Complex64::default(); 2 * atoms];
    let materials = guidance(atoms, TerminalState::Locked);
    let control = StateTickControl {
        transient_center: false,
        ordinary_bypass: false,
        analysis_advance: prepared.hop as f64,
    };
    assert_eq!(
        prepared.process_state_tick(
            &vec![Complex64::default(); 2 * atoms - 1],
            &materials,
            control,
            &mut output,
            &mut states,
        ),
        Err(StateError::CurrentShape)
    );
    assert_eq!(
        prepared.process_state_tick(
            &current,
            &materials[..atoms - 1],
            control,
            &mut output,
            &mut states,
        ),
        Err(StateError::GuidanceShape)
    );
    assert_eq!(
        prepared.process_state_tick(
            &current,
            &materials,
            control,
            &mut output[..2 * atoms - 1],
            &mut states,
        ),
        Err(StateError::OutputShape)
    );
    assert_eq!(
        prepared.process_state_tick(
            &current,
            &materials,
            control,
            &mut output,
            &mut states[..atoms - 1],
        ),
        Err(StateError::StateShape)
    );
    assert_eq!(
        prepared.process_state_tick(
            &current,
            &materials,
            StateTickControl {
                analysis_advance: 0.0,
                ..control
            },
            &mut output,
            &mut states,
        ),
        Err(StateError::AnalysisAdvance)
    );
    assert_eq!(snapshot, state_snapshot(&prepared));
    assert!(output.iter().all(|value| *value == Complex64::default()));
    assert!(states.iter().all(|state| *state == TerminalState::Reset));
}

fn state_snapshot(prepared: &Prepared) -> (Vec<f64>, Vec<RegionRecord>, bool, usize) {
    (
        prepared.phase.clone(),
        prepared.regions.clone(),
        prepared.has_state,
        prepared.region_slot,
    )
}
