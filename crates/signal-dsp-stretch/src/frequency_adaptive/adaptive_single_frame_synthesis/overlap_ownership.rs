use super::super::study_local_schedule::{
    schedule::build_schedule,
    study::{analyze, select},
    BASE_HOP, SOURCE_FRAMES,
};
use super::super::HASH_OFFSET;
use super::anchors::{detect, projected};
use super::dense_attribution::{matched_peaks, sample_contributions, SampleContribution};
use super::quality::{control::controls, Control};
use super::render::{
    render_successor, render_successor_owned, render_successor_owned_traced,
    render_successor_traced,
};

const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];
const SOURCES: [usize; 2] = [SOURCE_FRAMES / 2 - 128, SOURCE_FRAMES / 2 + 128];

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct OverlapOwnershipReview {
    pub pre_errors: [[usize; 2]; 3],
    pub post_errors: [[usize; 2]; 3],
    pub maximum_target_delta: f64,
    pub replica_output: usize,
    pub replica_values: [f64; 2],
    pub replica_contributors: [Vec<SampleContribution>; 2],
    pub event_owned_samples: [usize; 3],
    pub ownership_hashes: [u64; 3],
    pub output_hashes: [[u64; 2]; 3],
    pub contribution_hashes: [u64; 2],
    pub evidence_hash: u64,
}

pub(in crate::frequency_adaptive) fn overlap_ownership_review() -> OverlapOwnershipReview {
    let input = &controls()
        .into_iter()
        .find(|(control, _)| *control == Control::DenseEvent)
        .expect("dense overlap control")
        .1;
    let mut pre_errors = [[0; 2]; 3];
    let mut post_errors = [[0; 2]; 3];
    let mut event_owned_samples = [0; 3];
    let mut ownership_hashes = [0; 3];
    let mut output_hashes = [[0; 2]; 3];
    let mut maximum_target_delta = 0.0_f64;
    let mut replica_output = 0;
    let mut replica_values = [0.0_f64; 2];
    let mut replica_contributors = std::array::from_fn(|_| Vec::new());

    for (index, ratio) in RATIOS.into_iter().enumerate() {
        let channels = [input.clone()];
        let study = analyze(&channels, SOURCE_FRAMES);
        let points = select(&study, 3.0, 2);
        let anchors = detect(&channels, SOURCE_FRAMES);
        let schedule = build_schedule(SOURCE_FRAMES, BASE_HOP, ratio, &points);
        let targets = SOURCES.map(|source| projected(&schedule, source) as usize);
        let pre = render_successor(&channels, ratio, &points, &anchors.positions, &schedule);
        let post = render_successor_owned(&channels, ratio, &points, &anchors.positions, &schedule);
        let pre_peaks = matched_peaks(&pre.samples[0], targets).0;
        let post_peaks = matched_peaks(&post.samples[0], targets).0;
        pre_errors[index] = std::array::from_fn(|event| pre_peaks[event].abs_diff(targets[event]));
        post_errors[index] =
            std::array::from_fn(|event| post_peaks[event].abs_diff(targets[event]));
        for target in targets {
            maximum_target_delta =
                maximum_target_delta.max((pre.samples[0][target] - post.samples[0][target]).abs());
        }
        event_owned_samples[index] = post.event_owned_samples;
        ownership_hashes[index] = post.event_ownership_hash;
        output_hashes[index] = [pre.output_hash, post.output_hash];

        if ratio == 2.0 {
            replica_output = pre_peaks
                .into_iter()
                .find(|peak| !targets.contains(peak))
                .expect("dense replica");
            let trace_outputs = [
                targets[0] as isize,
                targets[1] as isize,
                replica_output as isize,
            ];
            let pre = render_successor_traced(
                &channels,
                ratio,
                &points,
                &anchors.positions,
                &trace_outputs,
                &schedule,
            );
            let post = render_successor_owned_traced(
                &channels,
                ratio,
                &points,
                &anchors.positions,
                &trace_outputs,
                &schedule,
            );
            replica_values = [
                pre.samples[0][replica_output],
                post.samples[0][replica_output],
            ];
            replica_contributors = [
                sample_contributions(&pre, replica_output),
                sample_contributions(&post, replica_output),
            ];
        }
    }

    let contribution_hashes = replica_contributors.each_ref().map(|values| {
        let mut state = HASH_OFFSET;
        for value in values {
            hash_contribution(&mut state, value);
        }
        state
    });
    let mut evidence_hash = HASH_OFFSET;
    for value in pre_errors
        .into_iter()
        .flatten()
        .chain(post_errors.into_iter().flatten())
        .chain(event_owned_samples)
    {
        hash(&mut evidence_hash, value as u64);
    }
    for value in ownership_hashes
        .into_iter()
        .chain(output_hashes.into_iter().flatten())
        .chain(contribution_hashes)
    {
        hash(&mut evidence_hash, value);
    }
    for value in [
        maximum_target_delta.to_bits(),
        replica_output as u64,
        replica_values[0].to_bits(),
        replica_values[1].to_bits(),
    ] {
        hash(&mut evidence_hash, value);
    }
    OverlapOwnershipReview {
        pre_errors,
        post_errors,
        maximum_target_delta,
        replica_output,
        replica_values,
        replica_contributors,
        event_owned_samples,
        ownership_hashes,
        output_hashes,
        contribution_hashes,
        evidence_hash,
    }
}

fn hash_contribution(state: &mut u64, value: &SampleContribution) {
    for word in [
        value.frame_source as i64 as u64,
        value.frame_output as i64 as u64,
        value.frame_length as u64,
        value.dual_weight.to_bits(),
        value.value[0].to_bits(),
        value.value[1].to_bits(),
        value.frame_peak_output as i64 as u64,
        value.frame_peak_magnitude.to_bits(),
    ] {
        hash(state, word);
    }
}

fn hash(state: &mut u64, value: u64) {
    *state = (*state ^ value).wrapping_mul(0x100_0000_01b3);
}
