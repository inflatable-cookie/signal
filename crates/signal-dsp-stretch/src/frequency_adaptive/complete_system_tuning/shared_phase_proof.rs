use std::{fs, path::PathBuf};

use super::{
    configurations,
    objective_grid::{
        audio::{development_cases, synthetic_control},
        metrics::{event_error, identity_error, tone_error},
    },
    smear_attribution::metrics::{
        accumulate, event_disagreement, finish, pairwise_correlation, replica_count, Accumulator,
        ModeEvidence,
    },
    Configuration, Sensitivity,
};
use crate::frequency_adaptive::{
    complete_phase_synthesis::render::{
        render_configured, render_configured_with_layers, Mode, Render,
    },
    study_local_schedule::{
        schedule::{build_schedule_with_strength, Schedule},
        study::{analyze_with_geometry, select},
    },
    HASH_OFFSET,
};

const CANDIDATES: [&str; 3] = ["g512-sr-u0-rc-v1", "g512-sr-u1-rc-v1", "g512-sc-u1-rc-v0"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Direction {
    DevelopmentListeningExport,
    NonDuplicatingCoefficientOwnership,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Review {
    pub configurations: usize,
    pub development_rows: usize,
    pub renders: usize,
    pub holdout_reads: usize,
    pub structural_failures: [usize; 9],
    pub evidence: ModeEvidence,
    pub event_resets: usize,
    pub shared_phase_assignments: usize,
    pub maximum_layer_sum_error: f64,
    pub hashes: [u64; 2],
    pub direction: Direction,
}

pub(crate) fn shared_phase_proof_review() -> Review {
    let configurations = configurations()
        .into_iter()
        .filter(|configuration| CANDIDATES.contains(&configuration.stable_id().as_str()))
        .collect::<Vec<_>>();
    assert_eq!(configurations.len(), CANDIDATES.len());
    let cases = development_cases();
    let synthetic = synthetic_control();
    let mut accumulator = Accumulator::default();
    let mut structural_failures = [0; 9];
    let mut event_resets = 0;
    let mut shared_phase_assignments = 0;
    let mut maximum_layer_sum_error = 0.0_f64;
    let mut hashes = [HASH_OFFSET; 2];
    let mut report = String::from(
        "configuration\trow\tevent_disagreement\tcorrelation\tlayer_replicas\tcombined_replicas\tlayer_sum_error\n",
    );

    for configuration in configurations.iter().copied() {
        let (points, _, stretched) = execute(&synthetic, 1.5, configuration, false);
        let (_, _, identity) = execute(&synthetic, 1.0, configuration, false);
        structural_failures[0] +=
            usize::from(identity_error(&synthetic[0], &identity.samples[0]) > 5.0e-10);
        accumulate_structure(&identity, synthetic[0].len(), 1.0, &mut structural_failures);
        accumulate_structure(
            &stretched,
            synthetic[0].len(),
            1.5,
            &mut structural_failures,
        );
        structural_failures[6] += usize::from(tone_error(&stretched.samples[0]) > 2.0);
        structural_failures[7] += usize::from(event_error(&stretched.samples[0], 1.5) > 256);
        event_resets += stretched.event_resets;
        shared_phase_assignments += stretched.vertical_alignments;
        mix(&mut hashes[0], stretched.output_hash);
        mix(&mut hashes[1], points.len() as u64);

        for (row, case) in cases.iter().enumerate() {
            let (points, schedule, render) =
                execute(&case.channels, case.ratio, configuration, true);
            accumulate_structure(
                &render,
                case.channels[0].len(),
                case.ratio,
                &mut structural_failures,
            );
            let layers = render.layer_samples.as_ref().expect("shared layer samples");
            let combined = &render.samples[0];
            let stems = [&layers[0][0], &layers[1][0], &layers[2][0]];
            let layer_sum_error = combined
                .iter()
                .enumerate()
                .map(|(index, sample)| {
                    (sample - stems.iter().map(|stem| stem[index]).sum::<f64>()).abs()
                })
                .fold(0.0_f64, f64::max);
            maximum_layer_sum_error = maximum_layer_sum_error.max(layer_sum_error);
            structural_failures[8] += usize::from(layer_sum_error > 1.0e-12);
            let projected = projected_events(&points, &schedule, combined.len());
            let disagreement = event_disagreement(stems, &projected);
            let correlation = pairwise_correlation(stems);
            let layer_replicas = stems
                .iter()
                .map(|stem| replica_count(stem, &projected))
                .sum::<usize>();
            let combined_replicas = replica_count(combined, &projected);
            accumulate(
                &mut accumulator,
                disagreement,
                correlation,
                layer_replicas,
                combined_replicas,
                projected.len(),
            );
            event_resets += render.event_resets;
            shared_phase_assignments += render.vertical_alignments;
            mix(&mut hashes[0], render.output_hash);
            mix(&mut hashes[1], layer_sum_error.to_bits());
            report.push_str(&format!(
                "{}\t{}\t{:.6}\t{:.6}\t{}\t{}\t{:.12e}\n",
                configuration.stable_id(),
                row,
                disagreement.0,
                correlation,
                layer_replicas,
                combined_replicas,
                layer_sum_error,
            ));
        }
    }
    fs::write(report_path(), report).expect("write shared phase report");
    let evidence = finish(accumulator);
    let replica_growth = evidence.mean_combined_replica_count - evidence.mean_layer_replica_count;
    let pass = structural_failures == [0; 9]
        && evidence.mean_pairwise_event_disagreement < 8.0
        && evidence.mean_pairwise_correlation > 0.8
        && replica_growth <= 0.0
        && event_resets > 0
        && shared_phase_assignments > 0;
    Review {
        configurations: configurations.len(),
        development_rows: cases.len(),
        renders: configurations.len() * (cases.len() + 2),
        holdout_reads: 0,
        structural_failures,
        evidence,
        event_resets,
        shared_phase_assignments,
        maximum_layer_sum_error,
        hashes,
        direction: if pass {
            Direction::DevelopmentListeningExport
        } else {
            Direction::NonDuplicatingCoefficientOwnership
        },
    }
}

fn execute(
    channels: &[Vec<f64>],
    ratio: f64,
    configuration: Configuration,
    layers: bool,
) -> (Vec<usize>, Schedule, Render) {
    let study = analyze_with_geometry(channels, channels[0].len(), configuration.geometry);
    let (threshold, agreement) = match configuration.sensitivity {
        Sensitivity::Responsive => (3.0, 2),
        Sensitivity::Conservative => (6.0, 3),
    };
    let points = select(&study, threshold, agreement);
    let schedule = build_schedule_with_strength(
        channels[0].len(),
        128,
        ratio,
        &points,
        configuration.unity_strength(),
    );
    let render = if layers {
        render_configured_with_layers(
            channels,
            ratio,
            &points,
            &schedule,
            Mode::Shared,
            configuration,
        )
    } else {
        render_configured(
            channels,
            ratio,
            &points,
            &schedule,
            Mode::Shared,
            configuration,
        )
    };
    (points, schedule, render)
}

fn projected_events(points: &[usize], schedule: &Schedule, output_len: usize) -> Vec<usize> {
    points
        .iter()
        .filter_map(|point| schedule.positions.get(*point / 128).copied())
        .filter(|point| *point >= 256 && *point + 257 < output_len)
        .collect()
}

fn accumulate_structure(render: &Render, source_len: usize, ratio: f64, failures: &mut [usize; 9]) {
    failures[1] +=
        usize::from(render.samples[0].len() != (source_len as f64 * ratio).round() as usize);
    failures[2] += render.uncovered;
    failures[3] += render.non_finite;
    failures[4] += render.boundary_failures;
    failures[5] += render.event_order_failures;
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-bn-shared-phase-proof.tsv")
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
