use super::*;

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

fn current_record(prepared: &Prepared, atoms: usize, channel: usize, atom: usize) -> RegionRecord {
    prepared.regions[prepared.region_slot * 2 * atoms + channel * atoms + atom]
}
