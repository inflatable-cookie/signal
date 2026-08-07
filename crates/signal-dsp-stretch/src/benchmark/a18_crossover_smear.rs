//! `g10.041` Batch 41.3: the `A18` fix must not trade the pop for smearing.
//!
//! The candidate resets transient phase only above a crossover. This asserts it
//! against the corpus's own transient-smear measurement and production
//! policies, rather than an ad-hoc proxy — the first proxy tried here disagreed
//! with itself across ratios, which is exactly how Batch 41.1 went wrong.

use signal_primitives::Sample;

use crate::phase_vocoder::{
    high_band_transient_reset_phase_vocoder, phase_locked_phase_vocoder,
    transient_reset_phase_vocoder,
};
use crate::transient_smear::{measure_transient_smear, StretchTransientSmearPolicies};

use super::synthetic::synthetic_extreme_ratio;

/// Frozen crossover as a fraction of Nyquist: `240 Hz` at `48 kHz`.
const CROSSOVER: f64 = 0.010;

fn smear(input: &[Sample], output: &[Sample], ratio: f64) -> f64 {
    measure_transient_smear(
        input,
        output,
        ratio,
        1_024,
        256,
        StretchTransientSmearPolicies::production(),
    )
    .metric
    .value
}

#[test]
fn candidate_crossover_does_not_regress_transient_smear() {
    let input = synthetic_extreme_ratio().samples;
    for ratio in [1.5f64, 2.0, 3.0] {
        let target = (input.len() as f64 * ratio).round() as usize;
        let shipped = transient_reset_phase_vocoder(&input, target, ratio, 2_048, 512);
        let candidate =
            high_band_transient_reset_phase_vocoder(&input, target, ratio, 2_048, 512, CROSSOVER);
        assert!(
            smear(&input, &candidate, ratio) <= smear(&input, &shipped, ratio),
            "ratio {ratio}: candidate smeared {} against shipped {}",
            smear(&input, &candidate, ratio),
            smear(&input, &shipped, ratio),
        );
    }
}

/// Removing the reset outright is not the fix. It regresses smear at ratio
/// `3.0`, which is why the reset is kept above the crossover rather than
/// deleted.
#[test]
fn dropping_the_reset_entirely_regresses_smear() {
    let input = synthetic_extreme_ratio().samples;
    let ratio = 3.0;
    let target = (input.len() as f64 * ratio).round() as usize;
    let shipped = transient_reset_phase_vocoder(&input, target, ratio, 2_048, 512);
    let none = phase_locked_phase_vocoder(&input, target, ratio, 2_048, 512);
    assert!(
        smear(&input, &none, ratio) > smear(&input, &shipped, ratio),
        "expected removing the reset to smear more at ratio {ratio}"
    );
}
