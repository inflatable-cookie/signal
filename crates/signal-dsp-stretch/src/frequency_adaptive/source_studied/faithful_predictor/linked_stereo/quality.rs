pub(in crate::frequency_adaptive) mod analytic_overlap;
pub(in crate::frequency_adaptive) mod attribution;
pub(in crate::frequency_adaptive) mod coefficient_contribution;
mod controls;
pub(in crate::frequency_adaptive) mod gate_calibration;
mod measure;
pub(in crate::frequency_adaptive) mod projection_attribution;
pub(in crate::frequency_adaptive) mod synthesis_closure;

use super::{hash_values, mechanics_review, render, LinkedStereoMechanicsDirection, HASH_OFFSET};
use controls::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum LinkedStereoQualityDirection {
    StereoExport,
    QualityAttribution,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct LinkedStereoQualityRatioEvidence {
    pub(in crate::frequency_adaptive) ratio: f64,
    pub(in crate::frequency_adaptive) maximum_ipd_error_radians: f64,
    pub(in crate::frequency_adaptive) ipd_errors_radians: [[f64; 3]; 3],
    pub(in crate::frequency_adaptive) delay_change_frames: usize,
    pub(in crate::frequency_adaptive) maximum_mid_side_ratio_delta_db: f64,
    pub(in crate::frequency_adaptive) maximum_correlation_delta: f64,
    pub(in crate::frequency_adaptive) correlated_image_delta: [f64; 2],
    pub(in crate::frequency_adaptive) decorrelated_image_delta: [f64; 2],
    pub(in crate::frequency_adaptive) input_delay_frames: usize,
    pub(in crate::frequency_adaptive) output_delay_frames: usize,
    pub(in crate::frequency_adaptive) maximum_attack_error_frames: usize,
    pub(in crate::frequency_adaptive) replica_failures: usize,
    pub(in crate::frequency_adaptive) silent_peer_peak: f64,
    pub(in crate::frequency_adaptive) transient_attack_errors: [usize; 2],
    pub(in crate::frequency_adaptive) transient_replica_failures: [usize; 2],
    pub(in crate::frequency_adaptive) transient_silent_peer_peaks: [f64; 2],
    pub(in crate::frequency_adaptive) audio_hash: u64,
    pub(in crate::frequency_adaptive) measurement_hash: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct LinkedStereoQualityReview {
    pub(in crate::frequency_adaptive) mechanism_hash: u64,
    pub(in crate::frequency_adaptive) ratios: Vec<LinkedStereoQualityRatioEvidence>,
    pub(in crate::frequency_adaptive) audio_hash: u64,
    pub(in crate::frequency_adaptive) measurement_hash: u64,
    pub(in crate::frequency_adaptive) repeated: bool,
    pub(in crate::frequency_adaptive) direction: LinkedStereoQualityDirection,
}

pub(in crate::frequency_adaptive) fn quality_review() -> LinkedStereoQualityReview {
    let first = run();
    let second = run();
    let repeated = first == second;
    let passed = repeated
        && first.mechanism_ready
        && first.ratios.iter().all(|row| {
            row.maximum_ipd_error_radians <= 1.0e-9
                && row.delay_change_frames <= 1
                && row.maximum_mid_side_ratio_delta_db <= 0.25
                && row.maximum_correlation_delta <= 0.02
                && row.maximum_attack_error_frames <= 256
                && row.replica_failures == 0
                && row.silent_peer_peak == 0.0
        });
    LinkedStereoQualityReview {
        mechanism_hash: first.mechanism_hash,
        ratios: first.ratios,
        audio_hash: first.audio_hash,
        measurement_hash: first.measurement_hash,
        repeated,
        direction: if passed {
            LinkedStereoQualityDirection::StereoExport
        } else {
            LinkedStereoQualityDirection::QualityAttribution
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    mechanism_hash: u64,
    mechanism_ready: bool,
    ratios: Vec<LinkedStereoQualityRatioEvidence>,
    audio_hash: u64,
    measurement_hash: u64,
}

fn run() -> Run {
    let mechanics = mechanics_review();
    let delay = delay_control();
    let image_controls = [correlated_control(), decorrelated_control()];
    let transient_controls = [isolated_transient_control(), dense_transient_control()];
    let mut audio_hash = HASH_OFFSET;
    let mut measurement_hash = HASH_OFFSET;
    let mut ratios = Vec::with_capacity(RATIOS.len());

    for ratio in RATIOS {
        let mut row_audio_hash = HASH_OFFSET;
        let mut maximum_ipd_error_radians = 0.0_f64;
        let mut ipd_errors_radians = [[0.0; 3]; 3];
        for (phase_index, phase_offset) in PHASE_OFFSETS.into_iter().enumerate() {
            for (frequency_index, frequency) in TONE_FREQUENCIES.into_iter().enumerate() {
                let tones = tone_control(phase_offset, frequency);
                let output = render::linked([&tones[0], &tones[1]], ratio, SAMPLE_RATE);
                hash_values(&mut row_audio_hash, &[output.hash]);
                let error =
                    measure::maximum_ipd_error(&tones, &output.channels, &[frequency], SAMPLE_RATE);
                ipd_errors_radians[phase_index][frequency_index] = error;
                maximum_ipd_error_radians = maximum_ipd_error_radians.max(error);
            }
        }

        let delay_output = render::linked([&delay[0], &delay[1]], ratio, SAMPLE_RATE);
        hash_values(&mut row_audio_hash, &[delay_output.hash]);
        let input_delay = measure::best_delay(&delay[0], &delay[1], 32);
        let output_delay =
            measure::best_delay(&delay_output.channels[0], &delay_output.channels[1], 32);
        let delay_change_frames = input_delay.abs_diff(output_delay);

        let mut maximum_mid_side_ratio_delta_db = 0.0_f64;
        let mut maximum_correlation_delta = 0.0_f64;
        let mut image_deltas = [[0.0; 2]; 2];
        for (index, control) in image_controls.iter().enumerate() {
            let output = render::linked([&control[0], &control[1]], ratio, SAMPLE_RATE);
            hash_values(&mut row_audio_hash, &[output.hash]);
            let delta = measure::image_delta(control, &output.channels);
            image_deltas[index] = [delta.mid_side_ratio_db, delta.correlation];
            maximum_mid_side_ratio_delta_db =
                maximum_mid_side_ratio_delta_db.max(delta.mid_side_ratio_db);
            maximum_correlation_delta = maximum_correlation_delta.max(delta.correlation);
        }

        let mut maximum_attack_error_frames = 0;
        let mut replica_failures = 0;
        let mut silent_peer_peak = 0.0_f64;
        let mut transient_attack_errors = [0; 2];
        let mut transient_replica_failures = [0; 2];
        let mut transient_silent_peer_peaks = [0.0; 2];
        for (index, control) in transient_controls.iter().enumerate() {
            let silence = vec![0.0; control.samples.len()];
            let output = render::linked([&control.samples, &silence], ratio, SAMPLE_RATE);
            hash_values(&mut row_audio_hash, &[output.hash]);
            let transient = measure::transient_quality(&output.channels, &control.events, ratio);
            transient_attack_errors[index] = transient.maximum_error;
            transient_replica_failures[index] = transient.replica_failures;
            transient_silent_peer_peaks[index] = transient.silent_peer_peak;
            maximum_attack_error_frames = maximum_attack_error_frames.max(transient.maximum_error);
            replica_failures += transient.replica_failures;
            silent_peer_peak = silent_peer_peak.max(transient.silent_peer_peak);
        }

        let row_measurement_hash = measurement_row_hash(
            ratio,
            maximum_ipd_error_radians,
            &ipd_errors_radians,
            delay_change_frames,
            [input_delay, output_delay],
            maximum_mid_side_ratio_delta_db,
            maximum_correlation_delta,
            &image_deltas,
            maximum_attack_error_frames,
            replica_failures,
            silent_peer_peak,
            &transient_attack_errors,
            &transient_replica_failures,
            &transient_silent_peer_peaks,
            row_audio_hash,
        );
        hash_values(&mut audio_hash, &[row_audio_hash]);
        hash_values(&mut measurement_hash, &[row_measurement_hash]);
        ratios.push(LinkedStereoQualityRatioEvidence {
            ratio,
            maximum_ipd_error_radians,
            ipd_errors_radians,
            delay_change_frames,
            maximum_mid_side_ratio_delta_db,
            maximum_correlation_delta,
            correlated_image_delta: image_deltas[0],
            decorrelated_image_delta: image_deltas[1],
            input_delay_frames: input_delay,
            output_delay_frames: output_delay,
            maximum_attack_error_frames,
            replica_failures,
            silent_peer_peak,
            transient_attack_errors,
            transient_replica_failures,
            transient_silent_peer_peaks,
            audio_hash: row_audio_hash,
            measurement_hash: row_measurement_hash,
        });
    }

    hash_values(
        &mut measurement_hash,
        &[mechanics.evidence_hash, audio_hash],
    );
    Run {
        mechanism_hash: mechanics.evidence_hash,
        mechanism_ready: mechanics.direction == LinkedStereoMechanicsDirection::QualityGate,
        ratios,
        audio_hash,
        measurement_hash,
    }
}

#[allow(clippy::too_many_arguments)]
fn measurement_row_hash(
    ratio: f64,
    ipd: f64,
    ipd_controls: &[[f64; 3]; 3],
    delay: usize,
    delays: [usize; 2],
    mid_side: f64,
    correlation: f64,
    image_controls: &[[f64; 2]; 2],
    attack: usize,
    replicas: usize,
    crossfeed: f64,
    attack_controls: &[usize; 2],
    replica_controls: &[usize; 2],
    crossfeed_controls: &[f64; 2],
    audio_hash: u64,
) -> u64 {
    let mut hash = HASH_OFFSET;
    hash_values(
        &mut hash,
        &[
            ratio.to_bits(),
            ipd.to_bits(),
            delay as u64,
            mid_side.to_bits(),
            correlation.to_bits(),
            attack as u64,
            replicas as u64,
            crossfeed.to_bits(),
            audio_hash,
        ],
    );
    for controls in ipd_controls {
        hash_values(&mut hash, &controls.map(f64::to_bits));
    }
    hash_values(&mut hash, &[delays[0] as u64, delays[1] as u64]);
    for controls in image_controls {
        hash_values(&mut hash, &controls.map(f64::to_bits));
    }
    hash_values(
        &mut hash,
        &[
            attack_controls[0] as u64,
            attack_controls[1] as u64,
            replica_controls[0] as u64,
            replica_controls[1] as u64,
            crossfeed_controls[0].to_bits(),
            crossfeed_controls[1].to_bits(),
        ],
    );
    hash
}
