use super::{Configuration, ResetScope, Sensitivity};
use crate::frequency_adaptive::{
    complete_phase_synthesis::render::{render_configured, Mode},
    study_local_schedule::{
        schedule::build_schedule_with_strength,
        study::{analyze_with_geometry, select},
    },
    HASH_OFFSET,
};

const SOURCE_FRAMES: usize = 16_384;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ReachabilityReview {
    pub dimension_changes: [usize; 5],
    pub structural_failures: [usize; 6],
    pub event_resets_by_scope: [usize; 3],
    pub hashes: [u64; 5],
}

pub(crate) fn reachability_review() -> ReachabilityReview {
    let channels = controls();
    let baseline = Configuration {
        geometry: [512, 2_048, 8_192],
        sensitivity: Sensitivity::Responsive,
        unity_strength_index: 2,
        reset_scope: ResetScope::ShortOnly,
        vertical_alignment: true,
    };
    let geometries = [[256, 1_024, 4_096], [1_024, 4_096, 16_384]];
    let mut configurations = vec![baseline];
    configurations.extend(geometries.map(|geometry| Configuration {
        geometry,
        ..baseline
    }));
    configurations.push(Configuration {
        sensitivity: Sensitivity::Conservative,
        ..baseline
    });
    configurations.extend([0, 1].map(|unity_strength_index| Configuration {
        unity_strength_index,
        ..baseline
    }));
    configurations.extend(
        [ResetScope::ConfidenceOwned, ResetScope::FrequencyLimited].map(|reset_scope| {
            Configuration {
                reset_scope,
                ..baseline
            }
        }),
    );
    configurations.push(Configuration {
        vertical_alignment: false,
        ..baseline
    });

    let evidence = configurations
        .iter()
        .copied()
        .map(|configuration| execute(&channels, configuration))
        .collect::<Vec<_>>();
    let base = &evidence[0];
    let dimension_changes = [
        evidence[1..3]
            .iter()
            .filter(|item| item.output_hash != base.output_hash)
            .count(),
        usize::from(evidence[3].schedule_hash != base.schedule_hash),
        evidence[4..6]
            .iter()
            .filter(|item| item.schedule_hash != base.schedule_hash)
            .count(),
        evidence[6..8]
            .iter()
            .filter(|item| item.phase_hash != base.phase_hash)
            .count(),
        usize::from(evidence[8].phase_hash != base.phase_hash),
    ];
    let mut structural_failures = [0; 6];
    for item in &evidence {
        structural_failures[0] +=
            usize::from(item.length != (SOURCE_FRAMES as f64 * 1.5).round() as usize);
        structural_failures[1] += item.uncovered;
        structural_failures[2] += item.non_finite;
        structural_failures[3] += item.boundary_failures;
        structural_failures[4] += item.event_order_failures;
        structural_failures[5] += usize::from(item.decision_hash == 0);
    }
    let event_resets_by_scope = [
        evidence[0].event_resets,
        evidence[6].event_resets,
        evidence[7].event_resets,
    ];
    let mut hashes = [HASH_OFFSET; 5];
    for item in &evidence {
        mix(&mut hashes[0], item.study_hash);
        mix(&mut hashes[1], item.schedule_hash);
        mix(&mut hashes[2], item.magnitude_hash);
        mix(&mut hashes[3], item.phase_hash);
        mix(&mut hashes[4], item.output_hash);
    }
    ReachabilityReview {
        dimension_changes,
        structural_failures,
        event_resets_by_scope,
        hashes,
    }
}

struct Evidence {
    study_hash: u64,
    schedule_hash: u64,
    magnitude_hash: u64,
    phase_hash: u64,
    output_hash: u64,
    decision_hash: u64,
    length: usize,
    uncovered: usize,
    non_finite: usize,
    boundary_failures: usize,
    event_order_failures: usize,
    event_resets: usize,
}

fn execute(channels: &[Vec<f64>], configuration: Configuration) -> Evidence {
    let study = analyze_with_geometry(channels, SOURCE_FRAMES, configuration.geometry);
    let (threshold, agreement) = match configuration.sensitivity {
        Sensitivity::Responsive => (3.0, 2),
        Sensitivity::Conservative => (6.0, 3),
    };
    let points = select(&study, threshold, agreement);
    let schedule = build_schedule_with_strength(
        SOURCE_FRAMES,
        128,
        1.5,
        &points,
        configuration.unity_strength(),
    );
    let render = render_configured(channels, 1.5, &points, &schedule, Mode::Both, configuration);
    Evidence {
        study_hash: study.hash,
        schedule_hash: schedule.hash,
        magnitude_hash: render.magnitude_hash,
        phase_hash: render.phase_hash,
        output_hash: render.output_hash,
        decision_hash: render.channel_decision_hash,
        length: render.samples[0].len(),
        uncovered: render.uncovered,
        non_finite: render.non_finite,
        boundary_failures: render.boundary_failures,
        event_order_failures: render.event_order_failures,
        event_resets: render.event_resets,
    }
}

fn controls() -> Vec<Vec<f64>> {
    let mut left = vec![0.0; SOURCE_FRAMES];
    let mut right = vec![0.0; SOURCE_FRAMES];
    for index in 0..SOURCE_FRAMES {
        left[index] = 0.08 * (std::f64::consts::TAU * 997.0 * index as f64 / 48_000.0).sin();
        right[index] = 0.06 * (std::f64::consts::TAU * 1499.0 * index as f64 / 48_000.0).sin();
        let high = (std::f64::consts::TAU * 5_003.0 * index as f64 / 48_000.0).sin();
        left[index] += 0.4 * high;
        right[index] += 0.35 * high;
    }
    for event in [2_048, 4_096, 4_224, 8_192, 12_288] {
        for offset in 0..32 {
            let pulse = 8.0 * (-(offset as f64) / 7.0).exp();
            left[event + offset] += pulse;
            right[event + offset] += pulse * 0.72;
        }
    }
    vec![left, right]
}

fn mix(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
