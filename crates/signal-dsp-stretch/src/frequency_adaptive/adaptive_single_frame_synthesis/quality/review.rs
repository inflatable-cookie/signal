use super::super::super::study_local_schedule::{
    schedule::build_schedule,
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::super::super::HASH_OFFSET;
use super::super::anchors::detect;
use super::super::render::{
    render, render_native_successor_owned, render_successor, render_successor_owned, Mode,
};
use super::control::{controls, RATIOS};
use super::evidence::{case_hash, QualityReview};
use super::{measure, CaseEvidence, Control, QualityDirection};

#[derive(Clone, Copy)]
enum ReviewPath {
    LegacyCombined,
    Successor,
    SuccessorOwned,
    NativeSuccessorOwned,
}

pub(in crate::frequency_adaptive) fn quality_review() -> QualityReview {
    review(ReviewPath::LegacyCombined)
}

pub(in crate::frequency_adaptive) fn successor_quality_review() -> QualityReview {
    review(ReviewPath::Successor)
}

pub(in crate::frequency_adaptive) fn owned_successor_quality_review() -> QualityReview {
    review(ReviewPath::SuccessorOwned)
}

pub(in crate::frequency_adaptive) fn native_successor_quality_review() -> QualityReview {
    review(ReviewPath::NativeSuccessorOwned)
}

fn review(path: ReviewPath) -> QualityReview {
    let mut cases = Vec::with_capacity(controls().len() * RATIOS.len());
    for (control, input) in controls() {
        for ratio in RATIOS {
            cases.push(review_case(control, &input, ratio, path));
        }
    }
    let hard_failures = cases
        .iter()
        .map(|case| {
            let mode_failures = match path {
                ReviewPath::LegacyCombined => case
                    .modes
                    .iter()
                    .flat_map(|mode| mode.hard_failures)
                    .sum::<usize>(),
                ReviewPath::Successor
                | ReviewPath::SuccessorOwned
                | ReviewPath::NativeSuccessorOwned => case.modes[1].hard_failures.into_iter().sum(),
            };
            case.ownership_failures + mode_failures
        })
        .sum();
    let combined_regressions = cases.iter().map(|case| case.combined_regressions).sum();
    let pass = hard_failures == 0 && combined_regressions == 0;
    let direction = match (path, pass) {
        (ReviewPath::LegacyCombined, true) => QualityDirection::FrozenMonoDevelopmentObjective,
        (ReviewPath::LegacyCombined, false) => {
            QualityDirection::MeasuredPhaseEventVerticalOrSynthesisStage
        }
        (ReviewPath::Successor, true) => QualityDirection::SuccessorFrozenMonoDevelopmentObjective,
        (ReviewPath::Successor, false) => QualityDirection::SuccessorOwningMechanism,
        (ReviewPath::SuccessorOwned, true) => {
            QualityDirection::SuccessorFrozenMonoDevelopmentObjective
        }
        (ReviewPath::SuccessorOwned, false) => QualityDirection::SuccessorOwningMechanism,
        (ReviewPath::NativeSuccessorOwned, true) => {
            QualityDirection::SuccessorFrozenMonoDevelopmentObjective
        }
        (ReviewPath::NativeSuccessorOwned, false) => QualityDirection::SuccessorOwningMechanism,
    };
    let mut evidence_hash = HASH_OFFSET;
    for case in &cases {
        hash(&mut evidence_hash, case.hash);
    }
    QualityReview {
        cases,
        hard_failures,
        combined_regressions,
        evidence_hash,
        direction,
    }
}

fn review_case(control: Control, input: &[f64], ratio: f64, path: ReviewPath) -> CaseEvidence {
    let channels = [input.to_vec()];
    let study = analyze(&channels, SOURCE_FRAMES);
    let points = select(&study, 3.0, 2);
    let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
    let ordinary = render(&channels, ratio, &points, &schedule, Mode::Ordinary);
    let candidate = match path {
        ReviewPath::LegacyCombined => render(&channels, ratio, &points, &schedule, Mode::Both),
        ReviewPath::Successor => {
            let anchors = detect(&channels, SOURCE_FRAMES);
            render_successor(&channels, ratio, &points, &anchors.positions, &schedule)
        }
        ReviewPath::SuccessorOwned => {
            let anchors = detect(&channels, SOURCE_FRAMES);
            render_successor_owned(&channels, ratio, &points, &anchors.positions, &schedule)
        }
        ReviewPath::NativeSuccessorOwned => {
            let anchors = detect(&channels, SOURCE_FRAMES);
            render_native_successor_owned(&channels, ratio, &points, &anchors.positions, &schedule)
        }
    };
    let renders = [ordinary, candidate];
    let modes = [
        measure(control, input, ratio, &schedule, &renders[0]),
        measure(control, input, ratio, &schedule, &renders[1]),
    ];
    let combined_regressions = modes[0]
        .hard_failures
        .iter()
        .zip(modes[1].hard_failures)
        .filter(|(ordinary, candidate)| **ordinary == 0 && *candidate != 0)
        .count();
    let ownership_failures = usize::from(match path {
        ReviewPath::LegacyCombined => {
            renders[0].schedule_hash != renders[1].schedule_hash
                || renders[0].frame_hash != renders[1].frame_hash
                || renders[0].coefficient_hash != renders[1].coefficient_hash
                || renders[0].magnitude_hash != renders[1].magnitude_hash
        }
        ReviewPath::Successor | ReviewPath::SuccessorOwned | ReviewPath::NativeSuccessorOwned => {
            renders[0].schedule_hash != renders[1].schedule_hash
        }
    });
    let mode_deltas = [
        modes[1].impulse_crest_db - modes[0].impulse_crest_db,
        modes[1].replica_ratio - modes[0].replica_ratio,
        modes[1].texture[0] - modes[0].texture[0],
        modes[1].texture[1] - modes[0].texture[1],
        modes[1].texture[4] - modes[0].texture[4],
        modes[1].texture[5] - modes[0].texture[5],
    ];
    let mut result = CaseEvidence {
        control,
        ratio,
        selected_points: points.len(),
        modes,
        mode_deltas,
        ownership_failures,
        combined_regressions,
        hash: 0,
    };
    result.hash = case_hash(&result);
    result
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
