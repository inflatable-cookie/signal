//! `g10.041` guard for finding `A18`: low-mid pops on transients.
//!
//! The artifact is a phase discontinuity in the sustained low-frequency
//! component at each transient. A worst-step measure cannot see it — the
//! percussive attack at the same instant is a larger step and masks it — which
//! is why `g10.041` Batch 41.1 wrongly eliminated the phase-reset hypothesis.
//! This file measures the carrier's phase directly.

use signal_dsp_stretch::{OfflineHighQualityStretcher, TimeStretcher};

const RATE: f32 = 48_000.0;
const TONE: f32 = 80.0;

/// Sustained `80 Hz` plus a `25 ms` percussive attack every `250 ms`, with an
/// optional deliberately injected phase pop at each attack.
fn material(seconds: f32, pop_radians: f32) -> Vec<f32> {
    let frames = (RATE * seconds) as usize;
    let period = RATE as usize / 4;
    let mut seed = 0x12345678u32;
    let mut noise = move || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / 8_388_608.0 - 1.0
    };
    let mut phase = 0.0f32;
    let step = std::f32::consts::TAU * TONE / RATE;
    (0..frames)
        .map(|index| {
            let since = index % period;
            if since == 0 && index > 0 {
                phase += pop_radians;
            }
            phase += step;
            let tone = 0.35 * phase.sin();
            let attack = if since < (RATE * 0.025) as usize {
                0.9 * (-(since as f32) / (RATE * 0.004)).exp() * noise()
            } else {
                0.0
            };
            tone + attack
        })
        .collect()
}

fn moving_average(samples: &[f32], taps: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(samples.len());
    let mut acc = 0.0f32;
    for (index, sample) in samples.iter().enumerate() {
        acc += sample;
        if index >= taps {
            acc -= samples[index - taps];
        }
        out.push(acc / taps as f32);
    }
    out
}

/// Largest phase discontinuity of the `hz` carrier, in radians.
fn carrier_phase_jump(samples: &[f32], hz: f32) -> f32 {
    let taps = 512;
    let mut in_phase = Vec::with_capacity(samples.len());
    let mut quadrature = Vec::with_capacity(samples.len());
    for (index, sample) in samples.iter().enumerate() {
        let angle = std::f32::consts::TAU * hz * index as f32 / RATE;
        in_phase.push(sample * angle.cos());
        quadrature.push(sample * angle.sin());
    }
    let in_phase = moving_average(&in_phase, taps);
    let quadrature = moving_average(&quadrature, taps);

    let from = taps * 2;
    let to = samples.len().saturating_sub(taps * 2);
    let mut worst = 0.0f32;
    let mut previous: Option<f32> = None;
    let mut index = from;
    while index < to {
        let magnitude =
            (in_phase[index] * in_phase[index] + quadrature[index] * quadrature[index]).sqrt();
        if magnitude > 0.02 {
            let phase = quadrature[index].atan2(in_phase[index]);
            if let Some(prev) = previous {
                let mut delta = phase - prev;
                while delta > std::f32::consts::PI {
                    delta -= std::f32::consts::TAU;
                }
                while delta < -std::f32::consts::PI {
                    delta += std::f32::consts::TAU;
                }
                worst = worst.max(delta.abs());
            }
            previous = Some(phase);
        }
        index += 32;
    }
    worst
}

/// The metric must fire on an artifact it claims to detect, or a null from it
/// means nothing. Batch 41.1's worst-step measure did not, and produced a
/// confident wrong answer.
#[test]
fn a18_metric_detects_an_injected_pop() {
    let clean = carrier_phase_jump(&material(4.0, 0.0), TONE);
    let popped = carrier_phase_jump(&material(4.0, std::f32::consts::PI), TONE);
    assert!(
        clean < 0.3,
        "unprocessed clean material should sit near the noise floor, saw {clean:.3}rad"
    );
    assert!(
        popped > 2.5,
        "an injected pi-radian pop must register, saw {popped:.3}rad"
    );
}

/// `A18` itself. Ignored while the defect is open, with the measured value in
/// the reason, following the `g10.039` `G5` precedent.
#[test]
#[ignore = "A18 open: the offline path measures 2.752rad of carrier phase jump at ratio 2.0 against a 0.142rad floor"]
fn a18_offline_stretch_does_not_break_low_carrier_phase() {
    let source = material(4.0, 0.0);
    let floor = carrier_phase_jump(&source, TONE);

    for ratio in [1.5f64, 2.0, 3.0] {
        let rendered = OfflineHighQualityStretcher::new(ratio)
            .stretch_mono(&source)
            .expect("offline stretch should render");
        let jump = carrier_phase_jump(&rendered, TONE);
        assert!(
            jump < floor * 3.0,
            "ratio {ratio}: carrier phase jumped {jump:.3}rad against a {floor:.3}rad floor"
        );
    }
}
