use super::*;

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
