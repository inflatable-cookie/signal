pub(super) mod audio;
mod metrics;

use std::{fs, path::PathBuf};

use audio::{development_cases, synthetic_control};
use metrics::{dominates, event_error, identity_error, quality, tone_error};

use super::{configurations, Configuration, Sensitivity, HASH_OFFSET};
use crate::frequency_adaptive::{
    complete_phase_synthesis::render::{render_configured, Mode, Render},
    study_local_schedule::{
        schedule::build_schedule_with_strength,
        study::{analyze_with_geometry, select},
    },
};

#[derive(Clone, Debug)]
pub(crate) struct ObjectiveGridReview {
    pub configuration_count: usize,
    pub development_render_count: usize,
    pub passing_count: usize,
    pub frontier_count: usize,
    pub candidates: Vec<String>,
    pub holdout_reads: usize,
    pub hashes: [u64; 3],
}

#[derive(Clone)]
struct Evidence {
    configuration: Configuration,
    failures: [usize; 9],
    quality: [f64; 5],
    hash: u64,
}

pub(crate) fn objective_grid_review() -> ObjectiveGridReview {
    let configurations = configurations();
    let development = development_cases();
    let synthetic = synthetic_control();
    let mut evidence = Vec::with_capacity(configurations.len());
    for configuration in configurations.iter().copied() {
        evidence.push(evaluate(configuration, &synthetic, &development));
    }
    let passing = evidence
        .iter()
        .filter(|item| item.failures == [0; 9])
        .collect::<Vec<_>>();
    let frontier = passing
        .iter()
        .copied()
        .filter(|candidate| {
            !passing.iter().any(|other| {
                other.configuration != candidate.configuration
                    && dominates(other.quality, candidate.quality)
            })
        })
        .collect::<Vec<_>>();
    let candidates = representatives(&frontier);
    let report_path = report_path();
    fs::create_dir_all(report_path.parent().expect("report parent"))
        .expect("create objective report directory");
    fs::write(&report_path, report(&evidence, &frontier, &candidates))
        .expect("write objective report");
    let mut hashes = [HASH_OFFSET; 3];
    for item in &evidence {
        mix(&mut hashes[0], item.hash);
    }
    for item in &frontier {
        mix(&mut hashes[1], item.hash);
    }
    for candidate in &candidates {
        for byte in candidate.as_bytes() {
            mix(&mut hashes[2], u64::from(*byte));
        }
    }
    ObjectiveGridReview {
        configuration_count: configurations.len(),
        development_render_count: configurations.len() * development.len(),
        passing_count: passing.len(),
        frontier_count: frontier.len(),
        candidates,
        holdout_reads: 0,
        hashes,
    }
}

fn evaluate(
    configuration: Configuration,
    synthetic: &[Vec<f64>],
    development: &[audio::DevelopmentCase],
) -> Evidence {
    let ratio = 1.5;
    let (points, schedule, render) = execute(synthetic, ratio, configuration);
    let repeated = execute(synthetic, ratio, configuration).2;
    let identity = execute(synthetic, 1.0, configuration).2;
    let mut failures = hard_failures(
        &render, &repeated, &identity, synthetic, ratio, &points, &schedule,
    );
    let mut qualities = Vec::with_capacity(development.len());
    let mut aggregate_hash = render.output_hash;
    for case in development {
        let (_, _, output) = execute(&case.channels, case.ratio, configuration);
        failures[1] += usize::from(
            output.samples[0].len()
                != (case.channels[0].len() as f64 * case.ratio).round() as usize,
        );
        failures[2] += output.uncovered;
        failures[3] += output.non_finite;
        failures[4] += output.boundary_failures;
        failures[5] += output.event_order_failures;
        qualities.push(quality(&case.channels[0], &output.samples[0], case.ratio));
        mix(&mut aggregate_hash, output.output_hash);
    }
    let quality = std::array::from_fn(|index| {
        qualities.iter().map(|values| values[index]).sum::<f64>() / qualities.len() as f64
    });
    Evidence {
        configuration,
        failures,
        quality,
        hash: aggregate_hash,
    }
}

fn execute(
    channels: &[Vec<f64>],
    ratio: f64,
    configuration: Configuration,
) -> (
    Vec<usize>,
    crate::frequency_adaptive::study_local_schedule::schedule::Schedule,
    Render,
) {
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
    let render = render_configured(
        channels,
        ratio,
        &points,
        &schedule,
        Mode::Both,
        configuration,
    );
    (points, schedule, render)
}

fn hard_failures(
    render: &Render,
    repeated: &Render,
    identity: &Render,
    input: &[Vec<f64>],
    ratio: f64,
    points: &[usize],
    schedule: &crate::frequency_adaptive::study_local_schedule::schedule::Schedule,
) -> [usize; 9] {
    let target = (input[0].len() as f64 * ratio).round() as usize;
    let movement = points
        .iter()
        .filter(|point| {
            schedule.positions[**point / 128].abs_diff((ratio * **point as f64).round() as usize)
                > 256
        })
        .count();
    [
        usize::from(
            identity_error(&input[0], &identity.samples[0]) > 5.0e-10
                || tone_error(&render.samples[0]) > 2.0
                || event_error(&render.samples[0], ratio) > 256,
        ),
        usize::from(render.samples[0].len() != target),
        render.uncovered,
        render.non_finite,
        render.boundary_failures,
        render.event_order_failures,
        movement,
        usize::from(render.output_hash != repeated.output_hash),
        usize::from(render.channel_decision_hash == 0),
    ]
}

fn representatives(frontier: &[&Evidence]) -> Vec<String> {
    let mut selected = Vec::new();
    for metric in 0..5 {
        if let Some(item) = frontier
            .iter()
            .min_by(|left, right| left.quality[metric].total_cmp(&right.quality[metric]))
        {
            let id = item.configuration.stable_id();
            if !selected.contains(&id) {
                selected.push(id);
            }
        }
        if selected.len() == 3 {
            break;
        }
    }
    selected
}

fn report(evidence: &[Evidence], frontier: &[&Evidence], candidates: &[String]) -> String {
    let mut output = String::from("configuration\tpass\tidentity\tlength\tcoverage\tfinite\tboundary\tevent_order\tmovement\trepeat\tlinked\ttransient\ttonal\tevent\tendpoint\tresidual\tfrontier\tcandidate\n");
    for item in evidence {
        let id = item.configuration.stable_id();
        let frontier = frontier
            .iter()
            .any(|entry| entry.configuration == item.configuration);
        let candidate = candidates.contains(&id);
        output.push_str(&format!("{id}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{frontier}\t{candidate}\n", item.failures == [0; 9], item.failures[0], item.failures[1], item.failures[2], item.failures[3], item.failures[4], item.failures[5], item.failures[6], item.failures[7], item.failures[8], item.quality[0], item.quality[1], item.quality[2], item.quality[3], item.quality[4]));
    }
    output
}

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-successor-bk-objective-grid.tsv")
}
fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
