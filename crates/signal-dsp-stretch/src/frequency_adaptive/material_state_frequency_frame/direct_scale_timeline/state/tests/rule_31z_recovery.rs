use super::*;

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
