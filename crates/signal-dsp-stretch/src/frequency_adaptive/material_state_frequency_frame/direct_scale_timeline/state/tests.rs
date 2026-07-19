use super::*;

mod fixtures;
mod rule_31aa;
mod rule_31ac;
mod rule_31ac_matrix;
mod rule_31z_recovery;
use fixtures::*;

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
        "input={peer_input_offset} output={peer_output_offset}"
    );
    let high_input_offset = wrap(current[high_peak + 1].arg() - current[high_peak].arg());
    let high_output_offset = wrap(output[high_peak + 1].arg() - output[high_peak].arg());
    assert!(wrap(high_output_offset - high_input_offset).abs() <= 1.0e-12);
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
