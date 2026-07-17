use std::{fs, path::PathBuf};

use super::{
    external::{read_stereo, replace_directory, write_stereo},
    metrics::{control, ControlKind},
    ALIGNMENTS, LENGTHS, PHASES, RATIOS, SAMPLE_RATE,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::coherent_representation;
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::render::{
    self, PhaseFieldClassTrace,
};

mod report;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PhaseFieldClassEvidence {
    pub coefficients: usize,
    pub phase_delta_rms: f64,
    pub maximum_phase_delta: f64,
    pub relation_bins: usize,
    pub relation_before_rms: f64,
    pub relation_after_rms: f64,
    pub maximum_relation_before: f64,
    pub maximum_relation_after: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PhaseFieldGroup {
    pub ratio: f64,
    pub control: &'static str,
    pub classes: [PhaseFieldClassEvidence; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::frequency_adaptive) enum PhaseFieldDirection {
    SeedBeforeIntegration,
    CompletePeakOwnedRegion,
    CloseCurrentKernel,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::frequency_adaptive) struct PhaseFieldReview {
    pub rows: usize,
    pub groups: Vec<PhaseFieldGroup>,
    pub classes: [PhaseFieldClassEvidence; 3],
    pub evidence_hash: u64,
    pub repeated: bool,
    pub direction: PhaseFieldDirection,
}

pub(in crate::frequency_adaptive) fn review() -> PhaseFieldReview {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/stretch-linked-stereo-phase-field-attribution");
    replace_directory(&root);
    let first = run(&root.join("first"));
    let second = run(&root.join("second"));
    let repeated = first == second;
    let classes = evidence(&first.classes);
    let direction = direction(&classes);
    report::write(&root, &first, repeated, classes, direction);
    PhaseFieldReview {
        rows: first.rows,
        groups: first
            .groups
            .iter()
            .map(|group| PhaseFieldGroup {
                ratio: group.ratio,
                control: group.control,
                classes: evidence(&group.classes),
            })
            .collect(),
        classes,
        evidence_hash: first.evidence_hash,
        repeated,
        direction,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct GroupTrace {
    ratio: f64,
    control: &'static str,
    classes: [PhaseFieldClassTrace; 3],
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    rows: usize,
    groups: Vec<GroupTrace>,
    classes: [PhaseFieldClassTrace; 3],
    evidence_hash: u64,
}

fn run(root: &std::path::Path) -> Run {
    fs::create_dir_all(root).unwrap_or_else(|error| panic!("create {}: {error}", root.display()));
    let mut groups = Vec::new();
    for ratio in RATIOS {
        for kind in [ControlKind::Tone, ControlKind::Image] {
            groups.push(GroupTrace {
                ratio,
                control: kind.name(),
                classes: [PhaseFieldClassTrace::default(); 3],
            });
        }
    }
    let mut rows = 0;
    let mut evidence_hash = 0xcbf2_9ce4_8422_2325;
    let spacing =
        SAMPLE_RATE as f64 / coherent_representation::source_geometry(SAMPLE_RATE)[2] as f64;
    for source_frames in LENGTHS {
        for phase in PHASES {
            for bin_aligned in ALIGNMENTS {
                let frequency = (31.5 + if bin_aligned { 0.0 } else { 0.37 }) * spacing;
                for kind in [ControlKind::Tone, ControlKind::Image] {
                    let source = control(kind, source_frames, frequency, phase);
                    for ratio in RATIOS {
                        let path = root.join(format!(
                            "{}-{source_frames}-{phase:.2}-{bin_aligned}-{ratio:.2}.wav",
                            kind.name()
                        ));
                        write_stereo(&path, &source, SAMPLE_RATE as u32);
                        let input = read_stereo(&path, source_frames, SAMPLE_RATE as u32);
                        fs::remove_file(&path)
                            .unwrap_or_else(|error| panic!("remove {}: {error}", path.display()));
                        let rendered = render::linked_tracked_peaks(
                            [&input[0], &input[1]],
                            ratio,
                            SAMPLE_RATE,
                        );
                        let trace = rendered.tracked_peak_phase_trace;
                        merge_classes(
                            &mut groups
                                .iter_mut()
                                .find(|group| group.ratio == ratio && group.control == kind.name())
                                .expect("phase-field group")
                                .classes,
                            &trace.classes,
                        );
                        evidence_hash = hash_row(evidence_hash, rendered.hash, &trace.classes);
                        rows += 1;
                    }
                }
            }
        }
    }
    let mut classes = [PhaseFieldClassTrace::default(); 3];
    for group in &groups {
        merge_classes(&mut classes, &group.classes);
    }
    Run {
        rows,
        groups,
        classes,
        evidence_hash,
    }
}

fn merge_classes(totals: &mut [PhaseFieldClassTrace; 3], values: &[PhaseFieldClassTrace; 3]) {
    for (total, value) in totals.iter_mut().zip(values) {
        total.coefficients += value.coefficients;
        total.phase_delta_squared_sum += value.phase_delta_squared_sum;
        total.maximum_phase_delta = total.maximum_phase_delta.max(value.maximum_phase_delta);
        total.relation_bins += value.relation_bins;
        total.relation_before_squared_sum += value.relation_before_squared_sum;
        total.relation_after_squared_sum += value.relation_after_squared_sum;
        total.maximum_relation_before = total
            .maximum_relation_before
            .max(value.maximum_relation_before);
        total.maximum_relation_after = total
            .maximum_relation_after
            .max(value.maximum_relation_after);
    }
}

fn evidence(classes: &[PhaseFieldClassTrace; 3]) -> [PhaseFieldClassEvidence; 3] {
    classes.map(|class| PhaseFieldClassEvidence {
        coefficients: class.coefficients,
        phase_delta_rms: rms(class.phase_delta_squared_sum, class.coefficients),
        maximum_phase_delta: class.maximum_phase_delta,
        relation_bins: class.relation_bins,
        relation_before_rms: rms(class.relation_before_squared_sum, class.relation_bins),
        relation_after_rms: rms(class.relation_after_squared_sum, class.relation_bins),
        maximum_relation_before: class.maximum_relation_before,
        maximum_relation_after: class.maximum_relation_after,
    })
}

fn rms(sum: f64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        (sum / count as f64).sqrt()
    }
}

fn direction(classes: &[PhaseFieldClassEvidence; 3]) -> PhaseFieldDirection {
    let relation_growth = |class: PhaseFieldClassEvidence| {
        class.relation_after_rms > class.relation_before_rms + 1.0e-12
    };
    if relation_growth(classes[0]) && relation_growth(classes[1]) {
        PhaseFieldDirection::CompletePeakOwnedRegion
    } else if relation_growth(classes[2]) {
        PhaseFieldDirection::SeedBeforeIntegration
    } else {
        PhaseFieldDirection::CloseCurrentKernel
    }
}

fn hash_row(mut hash: u64, audio_hash: u64, classes: &[PhaseFieldClassTrace; 3]) -> u64 {
    for value in std::iter::once(audio_hash).chain(classes.iter().flat_map(|class| {
        [
            class.coefficients as u64,
            class.phase_delta_squared_sum.to_bits(),
            class.maximum_phase_delta.to_bits(),
            class.relation_bins as u64,
            class.relation_before_squared_sum.to_bits(),
            class.relation_after_squared_sum.to_bits(),
            class.maximum_relation_before.to_bits(),
            class.maximum_relation_after.to_bits(),
        ]
    })) {
        hash = (hash ^ value).wrapping_mul(0x100_0000_01b3);
    }
    hash
}
