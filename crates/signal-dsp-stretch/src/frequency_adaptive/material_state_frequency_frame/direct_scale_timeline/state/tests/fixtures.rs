use super::*;

pub(super) fn dense_frame(prepared: &Prepared, tick: usize) -> Vec<Complex64> {
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    (0..prepared.channels)
        .flat_map(|channel| {
            (0..atoms).map(move |atom| {
                let magnitude = 0.18 + ((atom * 17 + channel * 11) % 29) as f64 / 100.0;
                let phase = atom as f64 * 0.013
                    + channel as f64 * 0.37
                    + tick as f64 * (0.19 + channel as f64 * 0.07);
                Complex64::from_polar(magnitude, phase)
            })
        })
        .collect()
}

pub(super) fn guidance(atoms: usize, state: TerminalState) -> Vec<MaterialGuidance> {
    let value = match state {
        TerminalState::Attack => MaterialGuidance {
            tonalness: 0.2,
            noisiness: 0.4,
            transientness: 0.8,
        },
        TerminalState::Unlocked => MaterialGuidance {
            tonalness: 0.2,
            noisiness: 0.8,
            transientness: 0.7,
        },
        _ => MaterialGuidance {
            tonalness: 0.8,
            noisiness: 0.2,
            transientness: 0.2,
        },
    };
    vec![value; atoms]
}

pub(super) fn tick(
    prepared: &mut Prepared,
    current: &[Complex64],
    guidance: &[MaterialGuidance],
    transient_center: bool,
    ordinary_bypass: bool,
) -> (Vec<Complex64>, Vec<TerminalState>, StateTickReport) {
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let mut output = vec![Complex64::default(); current.len()];
    let mut states = vec![TerminalState::Reset; atoms];
    let report = prepared
        .process_state_tick(
            current,
            guidance,
            StateTickControl {
                transient_center,
                ordinary_bypass,
                analysis_advance: prepared.hop as f64 * 0.75,
            },
            &mut output,
            &mut states,
        )
        .expect("Rule 31Z direct state tick");
    assert!(output
        .iter()
        .zip(current)
        .all(|(output, current)| (output.norm() - current.norm()).abs() <= 1.0e-12));
    (output, states, report)
}

pub(super) fn terminal_sequence() -> (u64, [StateTickReport; 5]) {
    let mut prepared = prepare(48_000, 2, 1.5, false).expect("state proof geometry");
    let atoms = prepared.owned_bins.iter().sum::<usize>();
    let frames = std::array::from_fn::<_, 5, _>(|tick| dense_frame(&prepared, tick));
    let (reset_output, _, reset) = tick(
        &mut prepared,
        &frames[0],
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    );
    assert!(reset_output
        .iter()
        .zip(&frames[0])
        .all(|(output, current)| {
            current.norm_sqr() == 0.0 || wrap(output.arg() - current.arg()).abs() <= 1.0e-12
        }));
    let (attack_output, attack_states, attack) = tick(
        &mut prepared,
        &frames[1],
        &guidance(atoms, TerminalState::Attack),
        true,
        false,
    );
    for (atom, state) in attack_states.iter().enumerate() {
        if *state == TerminalState::Attack {
            for channel in 0..prepared.channels {
                let index = channel * atoms + atom;
                assert!(wrap(attack_output[index].arg() - frames[1][index].arg()).abs() <= 1.0e-12);
            }
        }
    }
    let unlocked = tick(
        &mut prepared,
        &frames[2],
        &guidance(atoms, TerminalState::Unlocked),
        false,
        false,
    )
    .2;
    let ordinary = tick(
        &mut prepared,
        &frames[3],
        &guidance(atoms, TerminalState::Locked),
        false,
        true,
    )
    .2;
    let locked = tick(
        &mut prepared,
        &frames[4],
        &guidance(atoms, TerminalState::Locked),
        false,
        false,
    )
    .2;
    let reports = [reset, attack, unlocked, ordinary, locked];
    let mut hash = HASH_OFFSET;
    for report in reports {
        hash_u64(&mut hash, report.hash);
    }
    (hash, reports)
}
